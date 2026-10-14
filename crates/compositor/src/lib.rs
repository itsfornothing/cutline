//! Cutline Compositor Trait, Adapters and Pipeline

pub mod adapter;
pub mod fcproj;
pub mod traits;

pub use adapter::FilmCraftCompositor;
pub use fcproj::ProjectBuilder;
pub use traits::{Compositor, CompositorError, RenderOptions, RenderReport};
