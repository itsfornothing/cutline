//! Variable Templating Engine (`{{var}}`)
//!
//! Evaluates server-side variable substitution on raw JSON before schema validation.
//! Does not execute code; performs safe string and typed value replacement.

use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TemplateError {
    #[error("missing required variable '{name}' at JSON pointer '{pointer}'")]
    MissingVariable { name: String, pointer: String },

    #[error("malformed variable placeholder in '{raw}' at JSON pointer '{pointer}'")]
    MalformedPlaceholder { raw: String, pointer: String },
}

/// Substitute variables in `root` JSON value in-place before schema validation.
///
/// Uses variables defined in `root["variables"]` merged with any external overrides.
pub fn substitute_variables(
    root: &mut Value,
    overrides: Option<&HashMap<String, Value>>,
) -> Result<(), TemplateError> {
    // 1. Collect variables from root["variables"]
    let mut vars = HashMap::new();
    if let Some(obj) = root.get("variables").and_then(Value::as_object) {
        for (k, v) in obj {
            vars.insert(k.clone(), v.clone());
        }
    }

    // 2. Merge runtime overrides
    if let Some(ov) = overrides {
        for (k, v) in ov {
            vars.insert(k.clone(), v.clone());
        }
    }

    // 3. Traverse and substitute recursively
    let mut pointer = String::new();
    substitute_recursive(root, &vars, &mut pointer)?;

    Ok(())
}

fn substitute_recursive(
    val: &mut Value,
    vars: &HashMap<String, Value>,
    pointer: &mut String,
) -> Result<(), TemplateError> {
    match val {
        Value::String(s) => {
            // Check for single full-string replacement e.g. "{{var}}"
            let trimmed = s.trim();
            if trimmed.starts_with("{{") && trimmed.ends_with("}}") && !trimmed[2..trimmed.len() - 2].contains("{{") {
                let var_name = trimmed[2..trimmed.len() - 2].trim();
                if var_name.is_empty() {
                    return Err(TemplateError::MalformedPlaceholder {
                        raw: s.clone(),
                        pointer: pointer.clone(),
                    });
                }
                let replacement = vars.get(var_name).ok_or_else(|| TemplateError::MissingVariable {
                    name: var_name.to_string(),
                    pointer: pointer.clone(),
                })?;
                *val = replacement.clone();
                return Ok(());
            }

            // Check for inline substring replacements e.g. "prefix {{var}} suffix"
            if s.contains("{{") {
                let mut result = String::with_capacity(s.len());
                let mut rest = s.as_str();

                while let Some(start_idx) = rest.find("{{") {
                    result.push_str(&rest[..start_idx]);
                    let after_start = &rest[start_idx + 2..];
                    let end_idx = after_start.find("}}").ok_or_else(|| TemplateError::MalformedPlaceholder {
                        raw: s.clone(),
                        pointer: pointer.clone(),
                    })?;
                    let var_name = after_start[..end_idx].trim();
                    if var_name.is_empty() {
                        return Err(TemplateError::MalformedPlaceholder {
                            raw: s.clone(),
                            pointer: pointer.clone(),
                        });
                    }
                    let replacement = vars.get(var_name).ok_or_else(|| TemplateError::MissingVariable {
                        name: var_name.to_string(),
                        pointer: pointer.clone(),
                    })?;

                    match replacement {
                        Value::String(rs) => result.push_str(rs),
                        Value::Number(n) => result.push_str(&n.to_string()),
                        Value::Bool(b) => result.push_str(&b.to_string()),
                        other => result.push_str(&other.to_string()),
                    }

                    rest = &after_start[end_idx + 2..];
                }
                result.push_str(rest);
                *val = Value::String(result);
            }
        }
        Value::Array(arr) => {
            let base_len = pointer.len();
            for (idx, item) in arr.iter_mut().enumerate() {
                pointer.push_str(&format!("/{idx}"));
                substitute_recursive(item, vars, pointer)?;
                pointer.truncate(base_len);
            }
        }
        Value::Object(map) => {
            let base_len = pointer.len();
            for (key, item) in map.iter_mut() {
                // Don't substitute inside the "variables" block itself
                if pointer.is_empty() && key == "variables" {
                    continue;
                }
                pointer.push('/');
                // RFC 6901 pointer escape: ~ -> ~0, / -> ~1
                let escaped_key = key.replace('~', "~0").replace('/', "~1");
                pointer.push_str(&escaped_key);

                substitute_recursive(item, vars, pointer)?;
                pointer.truncate(base_len);
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_variable_substitution() {
        let mut doc = json!({
            "variables": {
                "title": "Why I quit",
                "voice_url": "https://assets.com/vo.mp3",
                "count": 42
            },
            "assets": {
                "vo": { "type": "audio", "src": "{{voice_url}}" }
            },
            "tracks": [
                {
                    "id": "v1",
                    "text": "Header: {{title}} (ep {{count}})"
                }
            ]
        });

        substitute_variables(&mut doc, None).expect("substitution should succeed");

        assert_eq!(doc["assets"]["vo"]["src"], "https://assets.com/vo.mp3");
        assert_eq!(doc["tracks"][0]["text"], "Header: Why I quit (ep 42)");
    }

    #[test]
    fn test_missing_variable_reports_pointer() {
        let mut doc = json!({
            "variables": {
                "title": "Existing"
            },
            "assets": {
                "bg": { "src": "{{missing_bg}}" }
            }
        });

        let err = substitute_variables(&mut doc, None).unwrap_err();
        assert_eq!(
            err,
            TemplateError::MissingVariable {
                name: "missing_bg".to_string(),
                pointer: "/assets/bg/src".to_string(),
            }
        );
    }
}
