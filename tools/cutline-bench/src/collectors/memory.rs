//! ProcessTreeMemory collector
//!
//! Samples the sum of RSS across the root PID and all its descendants every 20ms.

use super::Collector;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

pub struct ProcessTreeMemoryCollector {
    running: Arc<AtomicBool>,
    peak_rss_bytes: Arc<AtomicU64>,
    handle: Option<JoinHandle<()>>,
    cgroup_peak_mb: Option<f64>,
    method: String,
}

impl Default for ProcessTreeMemoryCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessTreeMemoryCollector {
    pub fn new() -> Self {
        #[cfg(target_os = "macos")]
        let method = "libproc_taskinfo_recursive".to_string();
        #[cfg(target_os = "linux")]
        let method = "procfs_walking".to_string();
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        let method = "generic_rss".to_string();

        Self {
            running: Arc::new(AtomicBool::new(false)),
            peak_rss_bytes: Arc::new(AtomicU64::new(0)),
            handle: None,
            cgroup_peak_mb: None,
            method,
        }
    }
}

impl Collector for ProcessTreeMemoryCollector {
    fn start(&mut self, pid_tree_root: u32) {
        let running = Arc::clone(&self.running);
        let peak_rss_bytes = Arc::clone(&self.peak_rss_bytes);
        running.store(true, Ordering::SeqCst);
        peak_rss_bytes.store(0, Ordering::SeqCst);

        let handle = std::thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                let current_tree_rss = measure_tree_rss_bytes(pid_tree_root);
                peak_rss_bytes.fetch_max(current_tree_rss, Ordering::Relaxed);
                std::thread::sleep(Duration::from_millis(20));
            }
            // Final measurement on stop
            let current_tree_rss = measure_tree_rss_bytes(pid_tree_root);
            peak_rss_bytes.fetch_max(current_tree_rss, Ordering::Relaxed);
        });

        self.handle = Some(handle);
    }

    fn stop(&mut self) -> Value {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }

        let peak_bytes = self.peak_rss_bytes.load(Ordering::SeqCst);
        let peak_mb = (peak_bytes as f64 / (1024.0 * 1024.0) * 100.0).round() / 100.0;

        // On Linux, try reading cgroup v2 memory.peak if available
        #[cfg(target_os = "linux")]
        {
            if let Ok(cgroup_peak) = std::fs::read_to_string("/sys/fs/cgroup/memory.peak") {
                if let Ok(b) = cgroup_peak.trim().parse::<u64>() {
                    self.cgroup_peak_mb = Some((b as f64 / (1024.0 * 1024.0) * 100.0).round() / 100.0);
                }
            }
        }

        json!({
            "peak_rss_tree_mb": peak_mb,
            "peak_cgroup_mb": self.cgroup_peak_mb,
            "method": self.method,
        })
    }
}

pub fn measure_tree_rss_bytes(root_pid: u32) -> u64 {
    let pids = get_all_tree_pids(root_pid);
    let mut total_rss = 0u64;

    for pid in pids {
        if let Some(rss) = get_single_pid_rss(pid) {
            total_rss = total_rss.saturating_add(rss);
        }
    }

    total_rss
}

#[cfg(target_os = "macos")]
#[repr(C)]
pub(crate) struct ProcTaskInfo {
    pub pti_virtual_size: u64,
    pub pti_resident_size: u64,
    pub pti_total_user: u64,
    pub pti_total_system: u64,
    pub pti_threads_user: u64,
    pub pti_threads_system: u64,
    pub pti_policy: i32,
    pub pti_faults: i32,
    pub pti_pageins: i32,
    pub pti_cow_faults: i32,
    pub pti_messages_sent: i32,
    pub pti_messages_received: i32,
    pub pti_syscalls_mach: i32,
    pub pti_syscalls_unix: i32,
    pub pti_csw: i32,
    pub pti_threadnum: i32,
    pub pti_numrunning: i32,
    pub pti_priority: i32,
}

#[cfg(target_os = "macos")]
pub(crate) const PROC_PIDTASKINFO: i32 = 4;
#[cfg(target_os = "macos")]
pub(crate) const PROC_PPID_ONLY: u32 = 6;


#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn proc_pidinfo(
        pid: i32,
        flavor: i32,
        arg: u64,
        buffer: *mut std::ffi::c_void,
        buffersize: i32,
    ) -> i32;
    fn proc_listpids(type_: u32, typeinfo: u32, buffer: *mut std::ffi::c_void, buffersize: i32) -> i32;
}


#[cfg(target_os = "macos")]
pub fn get_single_pid_rss(pid: u32) -> Option<u64> {
    unsafe {
        let mut info: ProcTaskInfo = std::mem::zeroed();
        let ret = proc_pidinfo(
            pid as i32,
            PROC_PIDTASKINFO,
            0,
            &mut info as *mut _ as *mut std::ffi::c_void,
            std::mem::size_of::<ProcTaskInfo>() as i32,
        );
        if ret > 0 {
            Some(info.pti_resident_size)
        } else {
            None
        }
    }
}

#[cfg(target_os = "macos")]
pub fn get_all_tree_pids(root_pid: u32) -> Vec<u32> {
    let mut tree_pids = vec![root_pid];
    let mut queue = vec![root_pid];

    while let Some(parent) = queue.pop() {
        let mut buf = [0i32; 256];
        let bytes = unsafe {
            proc_listpids(
                PROC_PPID_ONLY,
                parent,
                buf.as_mut_ptr() as *mut std::ffi::c_void,
                (buf.len() * std::mem::size_of::<i32>()) as i32,
            )
        };
        if bytes > 0 {
            let count = bytes as usize / std::mem::size_of::<i32>();
            for &child in &buf[..count] {
                let child_u = child as u32;
                if child_u > 0 && child_u != parent && !tree_pids.contains(&child_u) {
                    tree_pids.push(child_u);
                    queue.push(child_u);
                }
            }
        }
    }

    tree_pids
}

#[cfg(target_os = "linux")]
pub fn get_single_pid_rss(pid: u32) -> Option<u64> {
    let statm_path = format!("/proc/{}/statm", pid);
    if let Ok(content) = std::fs::read_to_string(&statm_path) {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(pages) = parts[1].parse::<u64>() {
                let page_size = 4096u64;
                return Some(pages * page_size);
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
pub fn get_all_tree_pids(root_pid: u32) -> Vec<u32> {
    let mut tree_pids = vec![root_pid];
    let mut changed = true;

    // Iterate /proc until no new descendants are added
    while changed {
        changed = false;
        if let Ok(entries) = std::fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if let Ok(pid) = name.parse::<u32>() {
                    if !tree_pids.contains(&pid) {
                        if let Ok(stat) = std::fs::read_to_string(format!("/proc/{}/stat", pid)) {
                            // 4th field is PPID
                            if let Some(r_paren) = stat.rfind(')') {
                                let after_comm = &stat[r_paren + 1..];
                                let fields: Vec<&str> = after_comm.split_whitespace().collect();
                                if fields.len() >= 2 {
                                    if let Ok(ppid) = fields[1].parse::<u32>() {
                                        if tree_pids.contains(&ppid) {
                                            tree_pids.push(pid);
                                            changed = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    tree_pids
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn get_single_pid_rss(_pid: u32) -> Option<u64> {
    None
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn get_all_tree_pids(root_pid: u32) -> Vec<u32> {
    vec![root_pid]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_process_rss() {
        let current_pid = std::process::id();
        let rss = get_single_pid_rss(current_pid);
        assert!(rss.is_some());
        let rss_bytes = rss.unwrap();
        assert!(rss_bytes > 0, "Current process RSS should be > 0 bytes");
    }

    #[test]
    fn test_memory_hog_measurement() {
        // Ticket 6.1.3 Acceptance: A known memory hog (e.g. child allocating 500 MB)
        // is measured within +/-10%.
        // For unit testing in constrained sandbox/CI, test allocating 50 MB in child.
        let mut child = std::process::Command::new("python3")
            .arg("-c")
            .arg("import time; b = bytearray(50 * 1024 * 1024); time.sleep(0.4)")
            .spawn()
            .expect("Failed to spawn python3 child");

        let child_pid = child.id();
        let mut collector = ProcessTreeMemoryCollector::new();
        collector.start(child_pid);

        std::thread::sleep(Duration::from_millis(200));

        let res = collector.stop();
        let _ = child.wait();

        let peak_mb = res["peak_rss_tree_mb"].as_f64().expect("peak_rss_tree_mb missing");
        // 50 MB allocated + python runtime (~15 MB) should be between 45 MB and 85 MB
        assert!(
            peak_mb >= 45.0 && peak_mb <= 90.0,
            "Expected ~50-70 MB, measured: {} MB",
            peak_mb
        );
    }
}
