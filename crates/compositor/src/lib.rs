//! Cutline Compositor Trait and Backends

pub mod adapter;
pub mod traits;

pub use traits::{Compositor, CompositorError, RenderOptions, RenderReport};
