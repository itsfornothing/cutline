//! CLI argument parsing for cutline-bench

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "cutline-bench", about = "Cutline Performance Benchmark Harness")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Allow running in debug mode (disabled by default; release build is strictly enforced)
    #[arg(long, default_value_t = false, global = true)]
    pub allow_debug: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Print the hardware fingerprint as JSON
    Env,

    /// Run benchmark suite on a template
    Run(RunArgs),

    /// Per-frame latency probe
    Frames(FramesArgs),

    /// Generate docs/benchmarks.md report from raw run result JSON files
    Report(ReportArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct RunArgs {
    /// Path to a template JSON file or 'all' to run all benchmark templates
    #[arg(long)]
    pub template: String,

    /// Execution mode: engine (isolated compositor) or service (end-to-end API/worker)
    #[arg(long, default_value = "engine")]
    pub mode: BenchmarkMode,

    /// Number of measured runs (sequential)
    #[arg(long, default_value_t = 5)]
    pub runs: usize,

    /// Number of warmup runs (discarded, but saved as warmup.json)
    #[arg(long, default_value_t = 1)]
    pub warmup: usize,

    /// Cold-cache mode: auto, off, on
    #[arg(long, default_value = "off")]
    pub cold_cache: ColdCacheMode,

    /// Concurrency level (1, 2, 4...)
    #[arg(long, default_value_t = 1)]
    pub concurrency: usize,

    /// GPU rendering setting: auto or off
    #[arg(long, default_value = "auto")]
    pub gpu: GpuSetting,

    /// Run single-frame probe on first run (optional; disabled by default)
    #[arg(long, default_value_t = false)]
    pub probe_frames: bool,

    /// Output directory for per-run JSON results
    #[arg(long)]
    pub out: PathBuf,

}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum BenchmarkMode {
    Engine,
    Service,
}

impl std::fmt::Display for BenchmarkMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Engine => write!(f, "engine"),
            Self::Service => write!(f, "service"),
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColdCacheMode {
    Auto,
    Off,
    On,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuSetting {
    Auto,
    Off,
}

impl std::fmt::Display for GpuSetting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Off => write!(f, "off"),
        }
    }
}

#[derive(Parser, Debug, Clone)]
pub struct FramesArgs {
    /// Path to template JSON file
    #[arg(long)]
    pub template: PathBuf,

    /// Number of single-frame probe samples
    #[arg(long, default_value_t = 100)]
    pub count: usize,

    /// GPU rendering setting: auto or off
    #[arg(long, default_value = "auto")]
    pub gpu: GpuSetting,

    /// Optional output JSON file path
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Parser, Debug, Clone)]
pub struct ReportArgs {
    /// Directories containing benchmark run-*.json files
    #[arg(long = "in", num_args = 1..)]
    pub inputs: Vec<PathBuf>,

    /// Destination markdown file path
    #[arg(long, default_value = "docs/benchmarks.md")]
    pub out: PathBuf,
}
