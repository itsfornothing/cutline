//! Statistical calculations for benchmark measurements

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SeriesSummary {
    pub count: usize,
    pub median: f64,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub stddev: f64,
    pub cv_pct: f64,
    pub unstable: bool,
}

pub fn summarize_series(mut values: Vec<f64>) -> Option<SeriesSummary> {
    if values.is_empty() {
        return None;
    }

    let count = values.len();
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let min = values[0];
    let max = values[count - 1];

    let median = if count % 2 == 1 {
        values[count / 2]
    } else {
        (values[count / 2 - 1] + values[count / 2]) / 2.0
    };

    let sum: f64 = values.iter().sum();
    let mean = sum / count as f64;

    let variance = if count > 1 {
        let sq_diff_sum: f64 = values.iter().map(|v| (v - mean).powi(2)).sum();
        sq_diff_sum / (count - 1) as f64
    } else {
        0.0
    };

    let stddev = variance.sqrt();
    let cv_pct = if mean.abs() > 1e-9 {
        (stddev / mean) * 100.0
    } else {
        0.0
    };

    let unstable = cv_pct > 10.0;

    Some(SeriesSummary {
        count,
        median,
        min,
        max,
        mean,
        stddev,
        cv_pct,
        unstable,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_single_value() {
        let summary = summarize_series(vec![5.0]).unwrap();
        assert_eq!(summary.count, 1);
        assert_eq!(summary.median, 5.0);
        assert_eq!(summary.min, 5.0);
        assert_eq!(summary.max, 5.0);
        assert_eq!(summary.mean, 5.0);
        assert_eq!(summary.stddev, 0.0);
        assert_eq!(summary.cv_pct, 0.0);
        assert!(!summary.unstable);
    }

    #[test]
    fn test_stats_even_odd_median() {
        let odd = summarize_series(vec![1.0, 3.0, 2.0]).unwrap();
        assert_eq!(odd.median, 2.0);

        let even = summarize_series(vec![1.0, 4.0, 2.0, 3.0]).unwrap();
        assert_eq!(even.median, 2.5);
    }

    #[test]
    fn test_stats_unstable_detection() {
        // CV > 10%
        let values = vec![10.0, 10.0, 15.0, 10.0, 10.0];
        let summary = summarize_series(values).unwrap();
        assert!(summary.cv_pct > 10.0);
        assert!(summary.unstable);

        // CV < 10%
        let stable_values = vec![10.0, 10.1, 10.0, 9.9, 10.0];
        let stable_summary = summarize_series(stable_values).unwrap();
        assert!(stable_summary.cv_pct < 10.0);
        assert!(!stable_summary.unstable);
    }
}
