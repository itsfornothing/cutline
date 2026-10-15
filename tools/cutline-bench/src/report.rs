//! Benchmark report generator: converts result JSON files into docs/benchmarks.md

use crate::schema::*;
use crate::stats::summarize_series;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub fn generate_report(input_dirs: &[PathBuf], output_path: &Path) -> Result<String, String> {
    let mut results: Vec<BenchmarkRunResult> = Vec::new();

    for dir in input_dirs {
        if !dir.exists() {
            continue;
        }
        collect_results_from_dir(dir, &mut results)?;
    }

    if results.is_empty() {
        return Err("No benchmark result JSON files found in provided input directories".to_string());
    }

    // Sort deterministically by template and run_id
    results.sort_by(|a, b| {
        a.env
            .instance_type
            .cmp(&b.env.instance_type)
            .then_with(|| a.template.cmp(&b.template))
            .then_with(|| a.run_id.cmp(&b.run_id))
    });

    let markdown = render_markdown_report(&results);

    if let Some(parent) = output_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(output_path, &markdown)
        .map_err(|e| format!("Failed to write report to {}: {e}", output_path.display()))?;

    Ok(markdown)
}

fn collect_results_from_dir(dir: &Path, out: &mut Vec<BenchmarkRunResult>) -> Result<(), String> {
    if dir.is_file() {
        if dir.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = fs::read_to_string(dir).map_err(|e| e.to_string())?;
            if let Ok(res) = serde_json::from_str::<BenchmarkRunResult>(&content) {
                out.push(res);
            }
        }
        return Ok(());
    }

    let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_results_from_dir(&path, out)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if filename.starts_with("run-") || filename.starts_with("result-") {
                let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
                if let Ok(res) = serde_json::from_str::<BenchmarkRunResult>(&content) {
                    out.push(res);
                }
            }
        }
    }

    Ok(())
}

pub fn render_markdown_report(results: &[BenchmarkRunResult]) -> String {
    let mut md = String::new();

    md.push_str("# Cutline Benchmark Results & Hardware Reality Verification\n\n");
    md.push_str("> **Audit Rule:** All metrics are measured directly on physical or virtual instances. `null` and `unavailable` indicate unmeasured metrics. Generated automatically by `cutline-bench report`.\n\n");

    // 1. Environment table
    md.push_str("## 1. Environment Fingerprint\n\n");
    md.push_str("| Instance / Host | OS / Kernel | CPU Model | Cores | RAM (GB) | GPU Model | Driver | Power |\n");
    md.push_str("|---|---|---|---|---|---|---|---|\n");

    let mut seen_hosts = BTreeSet::new();
    for r in results {
        let host_key = format!("{}-{}", r.env.instance_type, r.env.cpu_model);
        if seen_hosts.insert(host_key) {
            let gpu_desc = r
                .env
                .gpu
                .as_ref()
                .map(|g| g.model.as_str())
                .unwrap_or("none");
            let driver_desc = r
                .env
                .gpu
                .as_ref()
                .map(|g| g.driver.as_str())
                .unwrap_or("none");
            let power = r.env.power_source.as_deref().unwrap_or("n/a");

            md.push_str(&format!(
                "| `{}` | {} ({}) | {} | {} | {:.1} | {} | {} | {} |\n",
                r.env.instance_type,
                r.env.os,
                r.env.kernel,
                r.env.cpu_model,
                r.env.cores,
                r.env.ram_gb,
                gpu_desc,
                driver_desc,
                power
            ));
        }
    }
    md.push_str("\n");

    // 2. Results table grouped by host
    md.push_str("## 2. Measured Performance by Hardware\n\n");

    // Group by instance_type
    let mut by_host: BTreeMap<String, Vec<&BenchmarkRunResult>> = BTreeMap::new();
    for r in results {
        by_host
            .entry(r.env.instance_type.clone())
            .or_default()
            .push(r);
    }

    for (host, host_runs) in &by_host {
        md.push_str(&format!("### Host: `{}`\n\n", host));
        md.push_str("| Template | Runs | Realtime Multiple (med [min-max]) | CV % | p95 Frame (ms) | Peak RSS (MB) | HW Encode | Aligner | Flags |\n");
        md.push_str("|---|---|---|---|---|---|---|---|---|\n");

        // Group by template
        let mut by_template: BTreeMap<String, Vec<&BenchmarkRunResult>> = BTreeMap::new();
        for r in host_runs {
            by_template.entry(r.template.clone()).or_default().push(r);
        }

        for (tmpl, runs) in &by_template {
            let count = runs.len();
            let multiples: Vec<f64> = runs.iter().map(|r| r.timing.realtime_multiple).collect();
            let summary = summarize_series(multiples).unwrap();

            let rss_values: Vec<f64> = runs.iter().map(|r| r.memory.peak_rss_tree_mb).collect();
            let rss_summary = summarize_series(rss_values).unwrap();

            let p95_frame = runs
                .iter()
                .filter_map(|r| r.frame_time_ms.p95)
                .next()
                .map(|v| format!("{:.1}", v))
                .unwrap_or_else(|| "unavailable".to_string());

            let hw_encode_str = runs
                .first()
                .and_then(|r| r.output.hw_encode)
                .map(|b| if b { "true" } else { "false" })
                .unwrap_or("null");

            let aligner = runs.first().map(|r| r.aligner.as_str()).unwrap_or("unknown");

            let mut flags = Vec::new();
            if summary.unstable {
                flags.push("unstable (CV>10%)");
            }
            if !runs.iter().all(|r| r.validity.duration_ok) {
                flags.push("duration_mismatch");
            }
            let flags_str = if flags.is_empty() {
                "ok".to_string()
            } else {
                flags.join(", ")
            };

            md.push_str(&format!(
                "| **{}** | {} | **{:.2}x** [{:.2}x - {:.2}x] | {:.1}% | {} | {:.1} | `{}` | `{}` | {} |\n",
                tmpl,
                count,
                summary.median,
                summary.min,
                summary.max,
                summary.cv_pct,
                p95_frame,
                rss_summary.max,
                hw_encode_str,
                aligner,
                flags_str
            ));
        }
        md.push_str("\n");
    }

    // 3. Target comparison
    md.push_str("## 3. PRD Acceptance Targets vs. Reality\n\n");
    md.push_str("| PRD Requirement | Target Threshold | Measured Value | Hardware Tested | Status |\n");
    md.push_str("|---|---|---|---|---|\n");

    // Target 1: GPU 1080p30 template A >= 5.0x
    let gpu_tmpl_a_runs: Vec<&BenchmarkRunResult> = results
        .iter()
        .filter(|r| {
            (r.template == "A-story-captions" || r.template.contains("story"))
                && (r.env.instance_type.contains("g4") || r.env.instance_type.contains("g5") || r.env.gpu.is_some())
        })
        .collect();

    if gpu_tmpl_a_runs.is_empty() {
        md.push_str("| GPU 1080p30 (Template A) Throughput | $\\ge 5.0\\times$ realtime | `null` | NVIDIA GPU (T4/A10G) | **NOT MEASURED** |\n");
    } else {
        let mults: Vec<f64> = gpu_tmpl_a_runs.iter().map(|r| r.timing.realtime_multiple).collect();
        let med = summarize_series(mults).unwrap().median;
        let status = if med >= 5.0 { "**PASS**" } else { "**MISS**" };
        let hw = gpu_tmpl_a_runs[0].env.instance_type.as_str();
        md.push_str(&format!(
            "| GPU 1080p30 (Template A) Throughput | $\\ge 5.0\\times$ realtime | **{:.2}x** | `{}` | {} |\n",
            med, hw, status
        ));
    }

    // Target 2: CPU-only 8 vCPU template A >= 0.7x
    let cpu_tmpl_a_runs: Vec<&BenchmarkRunResult> = results
        .iter()
        .filter(|r| {
            (r.template == "A-story-captions" || r.template.contains("story"))
                && (r.env.instance_type.contains("c6i") || (r.env.cores >= 8 && r.env.gpu.is_none()))
        })
        .collect();

    if cpu_tmpl_a_runs.is_empty() {
        md.push_str("| CPU-only 8 vCPU (Template A) Throughput | $\\ge 0.7\\times$ realtime | `null` | c6i.2xlarge | **NOT MEASURED** |\n");
    } else {
        let mults: Vec<f64> = cpu_tmpl_a_runs.iter().map(|r| r.timing.realtime_multiple).collect();
        let med = summarize_series(mults).unwrap().median;
        let status = if med >= 0.7 { "**PASS**" } else { "**MISS**" };
        let hw = cpu_tmpl_a_runs[0].env.instance_type.as_str();
        md.push_str(&format!(
            "| CPU-only 8 vCPU (Template A) Throughput | $\\ge 0.7\\times$ realtime | **{:.2}x** | `{}` | {} |\n",
            med, hw, status
        ));
    }

    // Target 3: Peak RSS at 1080p < 1500 MB
    let runs_1080p: Vec<&BenchmarkRunResult> = results
        .iter()
        .filter(|r| r.output.width <= 1920 && r.output.height <= 1920)
        .collect();

    if runs_1080p.is_empty() {
        md.push_str("| Peak RSS at 1080p | $< 1,500$ MB | `null` | Any | **NOT MEASURED** |\n");
    } else {
        let max_rss = runs_1080p
            .iter()
            .map(|r| r.memory.peak_rss_tree_mb)
            .fold(0.0f64, f64::max);
        let status = if max_rss < 1500.0 { "**PASS**" } else { "**MISS**" };
        md.push_str(&format!(
            "| Peak RSS at 1080p | $< 1,500$ MB | **{:.1} MB** | All 1080p templates | {} |\n",
            max_rss, status
        ));
    }

    // Target 4: Peak RSS at 4K (Template C) < 3000 MB
    let runs_4k: Vec<&BenchmarkRunResult> = results
        .iter()
        .filter(|r| r.template == "C-4k-slideshow" || r.output.width >= 3840 || r.output.height >= 3840)
        .collect();

    if runs_4k.is_empty() {
        md.push_str("| Peak RSS at 4K (Template C) | $< 3,000$ MB | `null` | Any | **NOT MEASURED** |\n");
    } else {
        let max_rss_4k = runs_4k
            .iter()
            .map(|r| r.memory.peak_rss_tree_mb)
            .fold(0.0f64, f64::max);
        let status = if max_rss_4k < 3000.0 { "**PASS**" } else { "**MISS**" };
        md.push_str(&format!(
            "| Peak RSS at 4K (Template C) | $< 3,000$ MB | **{:.1} MB** | Template C | {} |\n",
            max_rss_4k, status
        ));
    }
    md.push_str("\n");

    // 4. Unit Economics
    md.push_str("## 4. Unit Economics & Pricing Model\n\n");
    md.push_str("### Compute Cost per Rendered Video Minute\n\n");
    md.push_str("Formula: `compute_cost_per_video_min = (instance_usd_per_hr / 60) / (realtime_multiple * utilization)`\n\n");
    md.push_str("| Instance Type | Pricing Mode | Hourly Rate | Source & Date | Measured Realtime Multiple | Cost/min (100% util) | Cost/min (60% util) | Cost/min (40% util) |\n");
    md.push_str("|---|---|---|---|---|---|---|---|\n");

    // Instances to evaluate
    let instance_catalog = [
        ("g4dn.xlarge (T4)", "On-Demand", 0.526, "AWS us-east-1 (Oct 2026)"),
        ("g4dn.xlarge (T4)", "Spot", 0.158, "AWS us-east-1 avg spot (Oct 2026)"),
        ("g5.xlarge (A10G)", "On-Demand", 1.006, "AWS us-east-1 (Oct 2026)"),
        ("g5.xlarge (A10G)", "Spot", 0.302, "AWS us-east-1 avg spot (Oct 2026)"),
        ("c6i.2xlarge (8 vCPU)", "On-Demand", 0.340, "AWS us-east-1 (Oct 2026)"),
        ("c6i.2xlarge (8 vCPU)", "Spot", 0.136, "AWS us-east-1 avg spot (Oct 2026)"),
    ];

    for (inst_name, mode_name, hourly, source) in instance_catalog {
        // Find measured multiple or state unmeasured
        let matching_runs: Vec<&BenchmarkRunResult> = results
            .iter()
            .filter(|r| inst_name.contains(&r.env.instance_type))
            .collect();

        if matching_runs.is_empty() {
            md.push_str(&format!(
                "| {} | {} | ${:.3}/hr | {} | *Pending Cloud Matrix* | - | - | - |\n",
                inst_name, mode_name, hourly, source
            ));
        } else {
            let mults: Vec<f64> = matching_runs.iter().map(|r| r.timing.realtime_multiple).collect();
            let mult = summarize_series(mults).unwrap().median;
            let c100 = (hourly / 60.0) / (mult * 1.0);
            let c60 = (hourly / 60.0) / (mult * 0.6);
            let c40 = (hourly / 60.0) / (mult * 0.4);

            md.push_str(&format!(
                "| {} | {} | ${:.3}/hr | {} | **{:.2}x** | ${:.4} | ${:.4} | ${:.4} |\n",
                inst_name, mode_name, hourly, source, mult, c100, c60, c40
            ));
        }
    }
    md.push_str("\n");

    // Local baseline comparison
    let local_runs: Vec<&BenchmarkRunResult> = results
        .iter()
        .filter(|r| r.env.instance_type == "local")
        .collect();

    if !local_runs.is_empty() {
        let mults: Vec<f64> = local_runs.iter().map(|r| r.timing.realtime_multiple).collect();
        let med = summarize_series(mults).unwrap().median;
        let chip = &local_runs[0].env.cpu_model;
        md.push_str(&format!(
            "> **Local Baseline Reference ({}):** Median realtime multiple across local runs is **{:.2}x**.\n\n",
            chip, med
        ));
    }

    md.push_str("### Margins vs. Pricing Hypotheses\n\n");
    md.push_str("- **Pricing Hypothesis:** $0.030 to $0.080 per rendered video minute.\n");
    md.push_str("- **Ancillary Costs:**\n");
    md.push_str("  - Egress: Cloudflare R2 = $0.000 / AWS CloudFront = $0.0034/min (at 5 Mbps 1080p).\n");
    md.push_str("  - Storage (24h scratch lifecycle): $0.00005/min.\n");
    md.push_str("- **Target Gross Margin:** $\\ge 75\\%$ on Starter tier ($0.08/min) and $\\ge 60\\%$ on Scale tier ($0.03/min).\n\n");

    // 5. Variance and caveats
    md.push_str("## 5. Variance, Caveats & Anomalies\n\n");
    let mut caveats = Vec::new();

    for r in results {
        if r.frame_time_ms.source == "unavailable" {
            caveats.push(format!(
                "- Run `{}` ({}) had unavailable single-frame latency probe.",
                r.run_id, r.template
            ));
        }
        if r.output.hw_encode.is_none() {
            caveats.push(format!(
                "- Run `{}` ({}) hardware encoder detection indeterminate (`null`).",
                r.run_id, r.template
            ));
        }
    }

    if caveats.is_empty() {
        md.push_str("No operational anomalies or unavailable metrics detected.\n");
    } else {
        // Limit caveats to first 10 for readability
        for c in caveats.iter().take(10) {
            md.push_str(&format!("{}\n", c));
        }
        if caveats.len() > 10 {
            md.push_str(&format!(
                "- ... and {} additional minor caveats.\n",
                caveats.len() - 10
            ));
        }
    }
    md.push_str("\n");

    md
}

pub(crate) fn create_dummy_result(template: &str, multiple: f64, rss: f64) -> BenchmarkRunResult {

        BenchmarkRunResult {
            run_id: "00000000-0000-0000-0000-000000000001".to_string(),
            template: template.to_string(),
            mode: "engine".to_string(),
            git_sha: "testsha".to_string(),
            filmcraft_version: "filmcraft-cli 0.7.2".to_string(),
            build_profile: "release+lto".to_string(),
            env: EnvInfo {
                os: "macos".to_string(),
                kernel: "25.6.0".to_string(),
                cpu_model: "Apple M1".to_string(),
                cores: 8,
                ram_gb: 8.0,
                gpu: None,
                power_source: Some("ac".to_string()),
                instance_type: "local".to_string(),
            },
            output: OutputInfo {
                path: "/tmp/out.mp4".to_string(),
                duration_s: 10.0,
                width: 1920,
                height: 1080,
                fps: 30.0,
                frames: 300,
                codec: "h264".to_string(),
                encoder: "FilmCraft".to_string(),
                hw_encode: Some(false),
                size_bytes: 500000,
                frame_hashes_sha256: "dummyhash".to_string(),
            },
            timing: TimingInfo {
                wall_total_s: 10.0 / multiple,
                render_wall_s: 10.0 / multiple,
                realtime_multiple: multiple,
                stages: StageTimingInfo {
                    fetch_s: 0.0,
                    align_s: 0.0,
                    composite_encode_s: 10.0 / multiple,
                    mux_s: 0.0,
                },
            },
            frame_time_ms: FrameTimeInfo {
                p50: Some(33.0),
                p95: Some(35.0),
                p99: Some(40.0),
                source: "probe".to_string(),
            },
            memory: MemoryInfo {
                peak_rss_tree_mb: rss,
                peak_cgroup_mb: None,
                method: "libproc".to_string(),
            },
            cpu: CpuInfo {
                avg_util_pct: 120.0,
                peak_util_pct: 250.0,
            },
            gpu_util: GpuUtilInfo {
                avg_pct: None,
                peak_pct: None,
                encoder_avg_pct: None,
                decoder_avg_pct: None,
                vram_peak_mb: None,
            },
            aligner: "rule_based".to_string(),
            validity: ValidityInfo {
                ffprobe_ok: true,
                duration_ok: true,
                errors: Vec::new(),
            },
        }
    }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_deterministic_byte_identical() {
        let results = vec![
            create_dummy_result("E-baseline-10s", 1.25, 250.0),
            create_dummy_result("A-story-captions", 0.95, 320.0),
            create_dummy_result("C-4k-slideshow", 0.40, 850.0),
        ];

        let rep1 = render_markdown_report(&results);
        let rep2 = render_markdown_report(&results);

        assert_eq!(rep1, rep2, "Report generation must be byte-identical");
        assert!(rep1.contains("PRD Acceptance Targets vs. Reality"));
        assert!(rep1.contains("Unit Economics & Pricing Model"));
    }
}

