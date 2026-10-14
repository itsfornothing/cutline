//! Audio Track Mixing, Constant-Power Panning, and BS.1770 / EBU R128 Loudness Normalization

use std::f32::consts::PI;

/// Compute stereo constant-power panning gains for a pan value between -1.0 (left) and +1.0 (right).
pub fn constant_power_pan(pan: f32) -> (f32, f32) {
    let clamped = pan.clamp(-1.0, 1.0);
    let angle = (clamped + 1.0) * (PI / 4.0);
    (angle.cos(), angle.sin())
}

/// Convert decibels (dB) to linear amplitude gain factor.
pub fn db_to_linear(db: f32) -> f32 {
    10.0f32.powf(db / 20.0)
}

/// Convert linear amplitude gain factor to decibels (dB).
pub fn linear_to_db(linear: f32) -> f32 {
    if linear <= 1e-7 {
        -140.0
    } else {
        20.0 * linear.log10()
    }
}

/// Simplified BS.1770 / EBU R128 loudness meter and normalization calculator.
pub struct LoudnessMeter {
    sample_rate: u32,
}

impl LoudnessMeter {
    pub fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Compute approximate integrated LUFS of a stereo audio buffer.
    pub fn integrated_lufs(&self, left: &[f32], right: &[f32]) -> f32 {
        let n = left.len().min(right.len());
        if n == 0 {
            return -70.0;
        }

        // K-weighting energy estimation
        let mut sum_sq = 0.0f64;
        for i in 0..n {
            let l = left[i] as f64;
            let r = right[i] as f64;
            // Channel weights: 1.0 for left and right
            sum_sq += l * l + r * r;
        }

        let mean_sq = sum_sq / (2.0 * n as f64);
        if mean_sq <= 1e-12 {
            return -70.0;
        }

        // Standard BS.1770 loudness offset
        let lkfs = -0.691 + 10.0 * mean_sq.log10();
        lkfs as f32
    }

    /// Calculate the gain adjustment in dB needed to reach `target_lufs` without exceeding `max_peak_db`.
    pub fn calculate_normalization_gain(
        &self,
        current_lufs: f32,
        target_lufs: f32,
        peak_linear: f32,
        max_peak_db: f32,
    ) -> f32 {
        let desired_gain_db = target_lufs - current_lufs;
        let max_allowed_linear = db_to_linear(max_peak_db);

        let desired_peak_linear = peak_linear * db_to_linear(desired_gain_db);
        if desired_peak_linear > max_allowed_linear && peak_linear > 0.0 {
            // Cap gain so we don't exceed true peak limit
            linear_to_db(max_allowed_linear / peak_linear)
        } else {
            desired_gain_db
        }
    }
}

/// Stereo audio buffer mixer with gain and pan control.
pub struct Mixer {
    sample_rate: u32,
}

impl Mixer {
    pub fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Mix a source stereo/mono track into the destination master buffer.
    pub fn mix_track(
        &self,
        src_left: &[f32],
        src_right: Option<&[f32]>,
        volume_db: f32,
        pan: f32,
        dst_left: &mut [f32],
        dst_right: &mut [f32],
    ) {
        let (pan_l, pan_r) = constant_power_pan(pan);
        let gain = db_to_linear(volume_db);

        let n = src_left.len().min(dst_left.len()).min(dst_right.len());

        if let Some(r_channel) = src_right {
            for i in 0..n {
                dst_left[i] += src_left[i] * gain * pan_l;
                dst_right[i] += r_channel[i] * gain * pan_r;
            }
        } else {
            // Mono input panned into stereo
            for i in 0..n {
                let sample = src_left[i] * gain;
                dst_left[i] += sample * pan_l;
                dst_right[i] += sample * pan_r;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_power_panning() {
        // Center pan (0.0): left and right should both be cos(pi/4) ≈ 0.7071
        let (l, r) = constant_power_pan(0.0);
        assert!((l - 0.7071).abs() < 1e-3);
        assert!((r - 0.7071).abs() < 1e-3);
        // Total power: l^2 + r^2 should equal 1.0
        assert!((l * l + r * r - 1.0).abs() < 1e-4);

        // Hard left (-1.0): left = 1.0, right = 0.0
        let (l_left, r_left) = constant_power_pan(-1.0);
        assert!((l_left - 1.0).abs() < 1e-4);
        assert!(r_left.abs() < 1e-4);

        // Hard right (+1.0): left = 0.0, right = 1.0
        let (l_right, r_right) = constant_power_pan(1.0);
        assert!(l_right.abs() < 1e-4);
        assert!((r_right - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_loudness_normalization_calc() {
        let meter = LoudnessMeter::new(48000);
        let current_lufs = -20.0;
        let target_lufs = -14.0;
        let peak_linear = 0.5; // -6 dB

        let gain_db = meter.calculate_normalization_gain(current_lufs, target_lufs, peak_linear, -1.0);
        // Target -14 from -20 is +6 dB. Peak with +6 dB is 0.5 * 2.0 = 1.0 (0 dB).
        // Since -1.0 dBTP limit is 10^(-1/20) ≈ 0.891, gain should be capped at around +5.0 dB!
        assert!(gain_db > 4.9 && gain_db < 5.1);
    }
}
