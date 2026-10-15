//! CpuUtil collector
//!
//! Samples per-tree CPU utilization percentage every 250ms.

use super::Collector;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub struct CpuUtilCollector {
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<(f64, f64)>>, // (avg, peak)
}

impl Default for CpuUtilCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuUtilCollector {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }
}

impl Collector for CpuUtilCollector {
    fn start(&mut self, pid_tree_root: u32) {
        let running = Arc::clone(&self.running);
        running.store(true, Ordering::SeqCst);

        let handle = std::thread::spawn(move || {
            let mut samples = Vec::new();
            let mut last_instant = Instant::now();
            let mut last_cpu_seconds = measure_tree_cpu_seconds(pid_tree_root);

            while running.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(250));

                let now = Instant::now();
                let wall_elapsed = now.duration_since(last_instant).as_secs_f64();
                let current_cpu_seconds = measure_tree_cpu_seconds(pid_tree_root);

                if wall_elapsed > 0.05 {
                    let cpu_delta = current_cpu_seconds.saturating_sub_f64(last_cpu_seconds);
                    let util_pct = (cpu_delta / wall_elapsed) * 100.0;
                    samples.push(util_pct);
                }

                last_instant = now;
                last_cpu_seconds = current_cpu_seconds;
            }

            if samples.is_empty() {
                (0.0, 0.0)
            } else {
                let peak = samples.iter().copied().fold(0.0f64, f64::max);
                let sum: f64 = samples.iter().sum();
                let avg = sum / samples.len() as f64;
                (
                    (avg * 10.0).round() / 10.0,
                    (peak * 10.0).round() / 10.0,
                )
            }
        });

        self.handle = Some(handle);
    }

    fn stop(&mut self) -> Value {
        self.running.store(false, Ordering::SeqCst);
        let (avg, peak) = if let Some(handle) = self.handle.take() {
            handle.join().unwrap_or((0.0, 0.0))
        } else {
            (0.0, 0.0)
        };

        json!({
            "avg_util_pct": avg,
            "peak_util_pct": peak,
        })
    }
}

trait SubF64 {
    fn saturating_sub_f64(self, other: f64) -> f64;
}

impl SubF64 for f64 {
    fn saturating_sub_f64(self, other: f64) -> f64 {
        if self > other {
            self - other
        } else {
            0.0
        }
    }
}

#[cfg(target_os = "macos")]
#[repr(C)]
struct MachTimebaseInfo {
    pub numer: u32,
    pub denom: u32,
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn mach_timebase_info(info: *mut MachTimebaseInfo) -> i32;
}

pub fn measure_tree_cpu_seconds(root_pid: u32) -> f64 {
    let pids = super::memory::get_all_tree_pids(root_pid);
    let mut total_seconds = 0.0;

    for pid in pids {
        total_seconds += get_single_pid_cpu_seconds(pid);
    }

    total_seconds
}

#[cfg(target_os = "macos")]
fn get_single_pid_cpu_seconds(pid: u32) -> f64 {
    use super::memory::{ProcTaskInfo, PROC_PIDTASKINFO};

    unsafe {
        let mut info: ProcTaskInfo = std::mem::zeroed();
        let ret = libc_proc_pidinfo(
            pid as i32,
            PROC_PIDTASKINFO,
            0,
            &mut info as *mut _ as *mut std::ffi::c_void,
            std::mem::size_of::<ProcTaskInfo>() as i32,
        );
        if ret > 0 {
            let mut tb = MachTimebaseInfo { numer: 0, denom: 0 };
            mach_timebase_info(&mut tb);
            if tb.denom == 0 {
                tb.denom = 1;
                tb.numer = 1;
            }
            let ticks = info.pti_total_user.saturating_add(info.pti_total_system);
            let ns = (ticks as f64 * tb.numer as f64) / tb.denom as f64;
            ns / 1_000_000_000.0
        } else {
            0.0
        }
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    #[link_name = "proc_pidinfo"]
    fn libc_proc_pidinfo(
        pid: i32,
        flavor: i32,
        arg: u64,
        buffer: *mut std::ffi::c_void,
        buffersize: i32,
    ) -> i32;
}


#[cfg(target_os = "linux")]
fn get_single_pid_cpu_seconds(pid: u32) -> f64 {
    if let Ok(stat) = std::fs::read_to_string(format!("/proc/{}/stat", pid)) {
        if let Some(r_paren) = stat.rfind(')') {
            let fields: Vec<&str> = stat[r_paren + 1..].split_whitespace().collect();
            // fields[11] is utime, fields[12] is stime (0-indexed after ')')
            if fields.len() >= 13 {
                let utime: f64 = fields[11].parse().unwrap_or(0.0);
                let stime: f64 = fields[12].parse().unwrap_or(0.0);
                let clk_tck = 100.0; // standard Linux USER_HZ
                return (utime + stime) / clk_tck;
            }
        }
    }
    0.0
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn get_single_pid_cpu_seconds(_pid: u32) -> f64 {
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_collector() {
        let mut collector = CpuUtilCollector::new();
        collector.start(std::process::id());

        // Burn CPU for 300 ms
        let t0 = Instant::now();
        while t0.elapsed() < Duration::from_millis(300) {
            let _ = 2 + 2;
        }

        let res = collector.stop();
        assert!(res["avg_util_pct"].is_number());
        assert!(res["peak_util_pct"].is_number());
    }
}
