//! Cutline Reframe Crate
//! Multi-format fan-out, smart reframing, and saliency tracking.

pub mod fanout;
pub mod saliency;

pub use fanout::{plan_fanout, AspectLayoutOverride, FanoutStream};
pub use saliency::{
    compute_smart_crop, BoundingBox, CropWindow, FocusPoint, SaliencyTracker,
};
