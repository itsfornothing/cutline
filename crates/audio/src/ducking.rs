//! Voice Auto-Ducking Algorithm
//!
//! Applies smooth gain attenuation to background music/sfx during active voice segments
//! using configurable attack and release curves.

use cutline_schema::DuckConfig;

/// State tracker and envelope generator for voice ducking.
#[derive(Debug, Clone)]
pub struct Ducker {
    config: DuckConfig,
    sample_rate: u32,
    target_gain_linear: f32,
    current_gain: f32,
    attack_coeff: f32,
    release_coeff: f32,
}

impl Ducker {
    /// Create a new ducker for a given configuration and audio sample rate.
    pub fn new(config: DuckConfig, sample_rate: u32) -> Self {
        let target_gain_linear = 10.0f32.powf(config.amount_db / 20.0);

        // Exponential smoothing coefficients: coeff = exp(-1 / (time_ms * 0.001 * sample_rate))
        let attack_samples = (config.attack_ms as f32 * 0.001 * sample_rate as f32).max(1.0);
        let release_samples = (config.release_ms as f32 * 0.001 * sample_rate as f32).max(1.0);

        let attack_coeff = (-1.0 / attack_samples).exp();
        let release_coeff = (-1.0 / release_samples).exp();

        Self {
            config,
            sample_rate,
            target_gain_linear,
            current_gain: 1.0,
            attack_coeff,
            release_coeff,
        }
    }

    /// Process a block of background audio samples, applying ducking based on voice presence.
    ///
    /// `voice_active`: true if voice is currently speaking during this block.
    /// `bg_samples`: slice of planar audio samples to scale in-place.
    pub fn process_block(&mut self, voice_active: bool, bg_samples: &mut [f32]) {
        let dest_gain = if voice_active {
            self.target_gain_linear
        } else {
            1.0
        };

        for sample in bg_samples.iter_mut() {
            let coeff = if dest_gain < self.current_gain {
                self.attack_coeff
            } else {
                self.release_coeff
            };

            self.current_gain = dest_gain + coeff * (self.current_gain - dest_gain);
            *sample *= self.current_gain;
        }
    }

    /// Returns the current gain factor.
    pub fn current_gain(&self) -> f32 {
        self.current_gain
    }

    pub fn config(&self) -> &DuckConfig {
        &self.config
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ducking_attenuation() {
        let config = DuckConfig {
            under: "voice".to_string(),
            amount_db: -12.0, // ≈ 0.2512 linear gain
            attack_ms: 10,
            release_ms: 20,
        };

        let mut ducker = Ducker::new(config, 48000);
        assert!((ducker.current_gain() - 1.0).abs() < 1e-4);

        // Feed 4800 samples (100ms) of active voice
        let mut buffer = vec![1.0f32; 4800];
        ducker.process_block(true, &mut buffer);

        // After 100ms with 10ms attack, gain should have settled near -12 dB (≈ 0.25)
        let expected_gain = 10.0f32.powf(-12.0 / 20.0);
        assert!((ducker.current_gain() - expected_gain).abs() < 0.01);
        assert!((buffer[buffer.len() - 1] - expected_gain).abs() < 0.01);

        // Now release voice for 200ms
        let mut silence_buffer = vec![1.0f32; 9600];
        ducker.process_block(false, &mut silence_buffer);

        // Gain should have recovered close to 1.0
        assert!((ducker.current_gain() - 1.0).abs() < 0.01);
        assert!((silence_buffer[silence_buffer.len() - 1] - 1.0).abs() < 0.01);
    }
}
