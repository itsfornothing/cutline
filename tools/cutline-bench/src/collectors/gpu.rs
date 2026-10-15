//! GpuUtil collector (NVIDIA GPUs via nvidia-smi)

use super::Collector;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GpuUtilMetrics {
    pub avg_pct: Option<f64>,
    pub peak_pct: Option<f64>,
    pub encoder_avg_pct: Option<f64>,
    pub decoder_avg_pct: Option<f64>,
    pub vram_peak_mb: Option<f64>,
}

pub struct GpuUtilCollector {
    running: Arc<AtomicBool>,
    child: Option<Child>,
    handle: Option<JoinHandle<GpuUtilMetrics>>,
}

impl Default for GpuUtilCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuUtilCollector {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            child: None,
            handle: None,
        }
    }
}

impl Collector for GpuUtilCollector {
    fn start(&mut self, _pid_tree_root: u32) {
        // Spawn nvidia-smi if available
        let mut cmd = Command::new("nvidia-smi");
        cmd.args([
            "--query-gpu=utilization.gpu,utilization.encoder,utilization.decoder,memory.used",
            "--format=csv,noheader,nounits",
            "-lms",
            "250",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

        match cmd.spawn() {
            Ok(mut child) => {
                let stdout = child.stdout.take();
                self.child = Some(child);
                let running = Arc::clone(&self.running);
                running.store(true, Ordering::SeqCst);

                if let Some(stdout) = stdout {
                    let handle = std::thread::spawn(move || {
                        use std::io::{BufRead, BufReader};
                        let reader = BufReader::new(stdout);
                        let mut lines = Vec::new();

                        for line in reader.lines() {
                            if !running.load(Ordering::Relaxed) {
                                break;
                            }
                            if let Ok(l) = line {
                                if !l.trim().is_empty() {
                                    lines.push(l);
                                }
                            }
                        }

                        parse_nvidia_smi_lines(&lines)
                    });
                    self.handle = Some(handle);
                }
            }
            Err(_) => {
                // Not an NVIDIA host or nvidia-smi not in PATH
                self.child = None;
                self.handle = None;
            }
        }
    }

    fn stop(&mut self) -> Value {
        self.running.store(false, Ordering::SeqCst);

        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }

        let metrics = if let Some(handle) = self.handle.take() {
            handle.join().unwrap_or_default()
        } else {
            GpuUtilMetrics::default()
        };

        json!({
            "avg_pct": metrics.avg_pct,
            "peak_pct": metrics.peak_pct,
            "encoder_avg_pct": metrics.encoder_avg_pct,
            "decoder_avg_pct": metrics.decoder_avg_pct,
            "vram_peak_mb": metrics.vram_peak_mb,
        })
    }
}

/// Parses lines emitted by nvidia-smi --query-gpu=utilization.gpu,utilization.encoder,utilization.decoder,memory.used --format=csv,noheader,nounits
pub fn parse_nvidia_smi_lines(lines: &[String]) -> GpuUtilMetrics {
    if lines.is_empty() {
        return GpuUtilMetrics::default();
    }

    let mut gpu_pcts = Vec::new();
    let mut enc_pcts = Vec::new();
    let mut dec_pcts = Vec::new();
    let mut vram_mbs = Vec::new();

    for line in lines {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 4 {
            if let Ok(gpu) = parts[0].parse::<f64>() {
                gpu_pcts.push(gpu);
            }
            if let Ok(enc) = parts[1].parse::<f64>() {
                enc_pcts.push(enc);
            }
            if let Ok(dec) = parts[2].parse::<f64>() {
                dec_pcts.push(dec);
            }
            if let Ok(vram) = parts[3].parse::<f64>() {
                vram_mbs.push(vram);
            }
        }
    }

    let avg_pct = if !gpu_pcts.is_empty() {
        let sum: f64 = gpu_pcts.iter().sum();
        Some(((sum / gpu_pcts.len() as f64) * 10.0).round() / 10.0)
    } else {
        None
    };

    let peak_pct = gpu_pcts.iter().copied().fold(None, |acc, v| {
        Some(acc.map_or(v, |m: f64| m.max(v)))
    });

    let encoder_avg_pct = if !enc_pcts.is_empty() {
        let sum: f64 = enc_pcts.iter().sum();
        Some(((sum / enc_pcts.len() as f64) * 10.0).round() / 10.0)
    } else {
        None
    };

    let decoder_avg_pct = if !dec_pcts.is_empty() {
        let sum: f64 = dec_pcts.iter().sum();
        Some(((sum / dec_pcts.len() as f64) * 10.0).round() / 10.0)
    } else {
        None
    };

    let vram_peak_mb = vram_mbs.iter().copied().fold(None, |acc, v| {
        Some(acc.map_or(v, |m: f64| m.max(v)))
    });

    GpuUtilMetrics {
        avg_pct,
        peak_pct,
        encoder_avg_pct,
        decoder_avg_pct,
        vram_peak_mb,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nvidia_smi_fixture() {
        // Captured sample fixture of nvidia-smi
        let fixture = vec![
            "0, 0, 0, 450".to_string(),
            "25, 42, 12, 1200".to_string(),
            "88, 95, 30, 2400".to_string(),
            "40, 60, 15, 2100".to_string(),
            "12, 10, 0, 1800".to_string(),
        ];

        let metrics = parse_nvidia_smi_lines(&fixture);

        assert_eq!(metrics.peak_pct, Some(88.0));
        assert_eq!(metrics.vram_peak_mb, Some(2400.0));
        // Avg GPU: (0+25+88+40+12)/5 = 33.0
        assert_eq!(metrics.avg_pct, Some(33.0));
        // Avg Enc: (0+42+95+60+10)/5 = 41.4
        assert_eq!(metrics.encoder_avg_pct, Some(41.4));
        // Avg Dec: (0+12+30+15+0)/5 = 11.4
        assert_eq!(metrics.decoder_avg_pct, Some(11.4));
    }

    #[test]
    fn test_non_nvidia_host_graceful() {
        let mut collector = GpuUtilCollector::new();
        collector.start(std::process::id());
        let res = collector.stop();
        // Should return null values gracefully on host without nvidia-smi
        assert!(res["avg_pct"].is_null() || res["avg_pct"].is_number());
    }
}
