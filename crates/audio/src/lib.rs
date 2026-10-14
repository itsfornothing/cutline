//! Cutline Audio Mixing, LUFS Normalization, and Voice Ducking

pub mod ducking;
pub mod mixer;

pub use ducking::Ducker;
pub use mixer::{LoudnessMeter, Mixer, constant_power_pan, db_to_linear, linear_to_db};
