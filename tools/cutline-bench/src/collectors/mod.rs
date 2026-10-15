//! Metric collectors for cutline-bench

pub mod cpu;
pub mod encoder;
pub mod gpu;
pub mod hasher;
pub mod memory;

pub use cpu::CpuUtilCollector;
pub use encoder::EncoderDetector;
pub use gpu::GpuUtilCollector;
pub use hasher::FrameHasher;
pub use memory::ProcessTreeMemoryCollector;

use serde_json::Value;

pub trait Collector: Send + Sync {
    fn start(&mut self, pid_tree_root: u32);
    fn stop(&mut self) -> Value;
}
