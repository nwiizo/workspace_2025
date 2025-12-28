use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use indicatif::{ProgressBar, ProgressStyle};

use crate::audio::AudioInfo;
use crate::error::{MosaicError, Result};
use crate::mosaic::{MosaicConfig, MosaicGenerator, PatternMode};

/// Video encoder configuration
#[derive(Debug, Clone)]
pub struct EncoderConfig {
    /// Output video width
    pub width: u32,
    /// Output video height
    pub height: u32,
    /// Frames per second
    pub fps: u32,
    /// Mosaic block size
    pub block_size: u32,
    /// Mosaic change interval in seconds
    pub change_interval: f64,
    /// Pattern mode
    pub pattern_mode: PatternMode,
}

impl EncoderConfig {
    /// Create a new encoder configuration
    pub fn new(
        width: u32,
        height: u32,
        fps: u32,
        block_size: u32,
        change_interval: f64,
    ) -> Self {
        Self {
            width,
            height,
            fps,
            block_size,
            change_interval,
            pattern_mode: PatternMode::Random,
        }
    }

    /// Set pattern mode
    pub fn with_pattern_mode(mut self, mode: PatternMode) -> Self {
        self.pattern_mode = mode;
        self
    }
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            fps: 24,
            block_size: 64,
            change_interval: 0.5,
            pattern_mode: PatternMode::Random,
        }
    }
}

/// Create a video from audio file with mosaic patterns
pub fn create_video<P: AsRef<Path>, Q: AsRef<Path>>(
    input_audio: P,
    output_video: Q,
    config: &EncoderConfig,
    audio_info: &AudioInfo,
) -> Result<()> {
    let output_path = output_video.as_ref();
    let input_path = input_audio.as_ref();

    // Calculate total frames
    let total_frames = (audio_info.duration_secs * config.fps as f64).ceil() as u64;

    // Create mosaic generator
    let mosaic_config = MosaicConfig::new(
        config.width,
        config.height,
        config.block_size,
        config.change_interval,
        config.fps,
    ).with_pattern_mode(config.pattern_mode);
    let mut mosaic_generator = MosaicGenerator::new(mosaic_config);

    // Create temporary video file path
    let temp_video = tempfile::Builder::new()
        .prefix("mosaic_video_")
        .suffix(".mp4")
        .tempfile()
        .map_err(|e| MosaicError::IoError(e))?;
    let temp_video_path = temp_video.path();

    // Start FFmpeg process for raw video input
    let mut ffmpeg = Command::new("ffmpeg")
        .args([
            "-y",
            "-f", "rawvideo",
            "-pix_fmt", "rgb24",
            "-s", &format!("{}x{}", config.width, config.height),
            "-r", &config.fps.to_string(),
            "-i", "-",
            "-c:v", "libx264",
            "-preset", "fast",
            "-crf", "23",
            "-pix_fmt", "yuv420p",
            temp_video_path.to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| MosaicError::EncoderInitError(format!("Failed to start FFmpeg: {}", e)))?;

    let stdin = ffmpeg.stdin.as_mut()
        .ok_or_else(|| MosaicError::EncoderInitError("Failed to open FFmpeg stdin".to_string()))?;

    // Create progress bar
    let progress = ProgressBar::new(total_frames);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} frames ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    // Generate and write frames
    let frame_size = (config.width * config.height * 3) as usize;
    let mut frame_buffer = vec![0u8; frame_size];

    for _ in 0..total_frames {
        mosaic_generator.write_frame_to_buffer(&mut frame_buffer);
        stdin.write_all(&frame_buffer)
            .map_err(|e| MosaicError::FrameEncodeError(format!("Failed to write frame: {}", e)))?;
        progress.inc(1);
    }

    progress.finish_with_message("Video encoding complete");

    // Close stdin to signal end of input (take ownership from ffmpeg)
    drop(ffmpeg.stdin.take());

    // Wait for FFmpeg to finish
    let status = ffmpeg.wait()
        .map_err(|e| MosaicError::VideoFinalizeError(format!("FFmpeg process error: {}", e)))?;

    if !status.success() {
        return Err(MosaicError::VideoFinalizeError("FFmpeg encoding failed".to_string()));
    }

    // Merge audio with video
    merge_audio_video(temp_video_path, input_path, output_path)?;

    Ok(())
}

/// Merge audio and video using FFmpeg command line
fn merge_audio_video(video_path: &Path, audio_path: &Path, output_path: &Path) -> Result<()> {
    println!("Merging audio with video...");

    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-i", video_path.to_str().unwrap(),
            "-i", audio_path.to_str().unwrap(),
            "-c:v", "copy",
            "-c:a", "aac",
            "-b:a", "192k",
            "-shortest",
            output_path.to_str().unwrap(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| MosaicError::VideoFinalizeError(format!("Failed to run FFmpeg: {}", e)))?;

    if !status.success() {
        return Err(MosaicError::VideoFinalizeError(
            "FFmpeg failed to merge audio and video".to_string(),
        ));
    }

    println!("Video created successfully: {}", output_path.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder_config_default() {
        let config = EncoderConfig::default();
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 720);
        assert_eq!(config.fps, 24);
        assert_eq!(config.block_size, 64);
        assert_eq!(config.change_interval, 0.5);
        assert_eq!(config.pattern_mode, PatternMode::Random);
    }

    #[test]
    fn test_encoder_config_new() {
        let config = EncoderConfig::new(1920, 1080, 30, 64, 1.0);
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert_eq!(config.fps, 30);
        assert_eq!(config.block_size, 64);
        assert_eq!(config.change_interval, 1.0);
    }

    #[test]
    fn test_encoder_config_with_pattern() {
        let config = EncoderConfig::new(1280, 720, 24, 64, 0.5)
            .with_pattern_mode(PatternMode::Plasma);
        assert_eq!(config.pattern_mode, PatternMode::Plasma);
    }
}
