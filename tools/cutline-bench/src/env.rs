//! Environment and hardware fingerprint collector

use crate::schema::{EnvInfo, GpuHardwareInfo};
use std::process::Command;

pub fn detect_env() -> EnvInfo {
    let os = std::env::consts::OS.to_string();
    let kernel = detect_kernel();
    let cpu_model = detect_cpu_model();
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let ram_gb = detect_ram_gb();
    let gpu = detect_gpu();
    let power_source = detect_power_source();
    let instance_type = detect_instance_type();

    EnvInfo {
        os,
        kernel,
        cpu_model,
        cores,
        ram_gb,
        gpu,
        power_source,
        instance_type,
    }
}

fn detect_kernel() -> String {
    let output = Command::new("uname").arg("-r").output();
    if let Ok(out) = output {
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    } else {
        "unknown".to_string()
    }
}

fn detect_cpu_model() -> String {
    #[cfg(target_os = "macos")]
    {
        if let Ok(out) = Command::new("sysctl")
            .arg("-n")
            .arg("machdep.cpu.brand_string")
            .output()
        {
            let model = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !model.is_empty() {
                return model;
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
            for line in cpuinfo.lines() {
                if line.starts_with("model name") {
                    if let Some((_, model)) = line.split_once(':') {
                        return model.trim().to_string();
                    }
                }
            }
        }
    }

    "unknown".to_string()
}

fn detect_ram_gb() -> f64 {
    #[cfg(target_os = "macos")]
    {
        if let Ok(out) = Command::new("sysctl").arg("-n").arg("hw.memsize").output() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if let Ok(bytes) = s.parse::<u64>() {
                return (bytes as f64 / (1024.0 * 1024.0 * 1024.0) * 10.0).round() / 10.0;
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
            for line in meminfo.lines() {
                if line.starts_with("MemTotal:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<u64>() {
                            return (kb as f64 / (1024.0 * 1024.0) * 10.0).round() / 10.0;
                        }
                    }
                }
            }
        }
    }

    0.0
}

fn detect_gpu() -> Option<GpuHardwareInfo> {
    // 1. Try nvidia-smi
    let nv_output = Command::new("nvidia-smi")
        .arg("--query-gpu=name,driver_version")
        .arg("--format=csv,noheader")
        .output();

    if let Ok(out) = nv_output {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            if let Some(first_line) = s.lines().next() {
                let parts: Vec<&str> = first_line.split(',').map(|p| p.trim()).collect();
                if !parts.is_empty() {
                    let model = parts[0].to_string();
                    let driver = parts.get(1).unwrap_or(&"unknown").to_string();

                    // Query cuda version
                    let cuda_out = Command::new("nvcc").arg("--version").output();
                    let cuda = if let Ok(co) = cuda_out {
                        let cs = String::from_utf8_lossy(&co.stdout);
                        cs.lines()
                            .find(|l| l.contains("release"))
                            .map(|l| l.trim().to_string())
                    } else {
                        None
                    };

                    return Some(GpuHardwareInfo {
                        model,
                        driver,
                        cuda,
                    });
                }
            }
        }
    }

    // 2. On macOS, detect Apple Silicon GPU
    #[cfg(target_os = "macos")]
    {
        let cpu = detect_cpu_model();
        if cpu.starts_with("Apple M") {
            return Some(GpuHardwareInfo {
                model: format!("{} Integrated GPU", cpu),
                driver: "Metal / Apple Silicon".to_string(),
                cuda: None,
            });
        }
    }

    None
}

fn detect_power_source() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        if let Ok(out) = Command::new("pmset").arg("-g").arg("batt").output() {
            let s = String::from_utf8_lossy(&out.stdout);
            if s.contains("AC Power") {
                return Some("ac".to_string());
            } else if s.contains("Battery Power") {
                return Some("battery".to_string());
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("AC") || name.starts_with("Mains") {
                    let online = std::fs::read_to_string(entry.path().join("online")).unwrap_or_default();
                    if online.trim() == "1" {
                        return Some("ac".to_string());
                    }
                }
            }
        }
    }

    Some("n/a".to_string())
}

fn detect_instance_type() -> String {
    use std::io::{Read, Write};
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;

    let addr: Result<SocketAddr, _> = "169.254.169.254:80".parse();
    if let Ok(addr) = addr {
        if let Ok(mut stream) = TcpStream::connect_timeout(&addr, Duration::from_millis(100)) {
            let _ = stream.set_read_timeout(Some(Duration::from_millis(100)));
            let req = "GET /latest/meta-data/instance-type HTTP/1.0\r\nHost: 169.254.169.254\r\n\r\n";
            if stream.write_all(req.as_bytes()).is_ok() {
                let mut buf = Vec::new();
                if stream.read_to_end(&mut buf).is_ok() {
                    let resp = String::from_utf8_lossy(&buf);
                    if let Some(pos) = resp.find("\r\n\r\n") {
                        let body = resp[pos + 4..].trim();
                        if !body.is_empty() && !body.contains("404") && !body.contains("Error") {
                            return body.to_string();
                        }
                    }
                }
            }
        }
    }

    "local".to_string()
}

