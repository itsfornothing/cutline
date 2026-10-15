//! FrameHasher
//!
//! Computes SHA256 hash of all decoded video frames using ffmpeg.

use std::path::Path;
use std::process::Command;

pub struct FrameHasher;

impl FrameHasher {
    pub fn hash_video_frames(path: &Path) -> Result<String, String> {
        let output = Command::new("ffmpeg")
            .arg("-v")
            .arg("error")
            .arg("-i")
            .arg(path)
            .arg("-f")
            .arg("hash")
            .arg("-hash")
            .arg("sha256")
            .arg("-")
            .output()
            .map_err(|e| format!("failed to execute ffmpeg for frame hashing: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("ffmpeg frame hash failed: {stderr}"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some(stripped) = line.strip_prefix("SHA256=") {
                return Ok(stripped.trim().to_string());
            }
        }

        let trimmed = stdout.trim();
        if !trimmed.is_empty() {
            Ok(trimmed.to_string())
        } else {
            Err("ffmpeg produced empty frame hash output".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_hasher_on_bg_asset() {
        let asset = Path::new("bench/assets/bg_1080p30.mp4");
        if asset.exists() {
            let hash = FrameHasher::hash_video_frames(asset).expect("hash_video_frames failed");
            assert_eq!(hash.len(), 64, "SHA-256 hash should be 64 hex chars");
        }
    }
}
