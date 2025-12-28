use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use mosaic_video::{create_video, get_audio_info, EncoderConfig, PatternMode};

/// Convert MP3 to video with animated mosaic patterns
#[derive(Parser, Debug)]
#[command(name = "mosaic-video")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input MP3 file
    #[arg(value_name = "INPUT")]
    input: PathBuf,

    /// Output MP4 file
    #[arg(short, long, default_value = "output.mp4")]
    output: PathBuf,

    /// Video width in pixels
    #[arg(short = 'W', long, default_value_t = 1280)]
    width: u32,

    /// Video height in pixels
    #[arg(short = 'H', long, default_value_t = 720)]
    height: u32,

    /// Frames per second
    #[arg(short, long, default_value_t = 24)]
    fps: u32,

    /// Mosaic block size in pixels
    #[arg(short, long, default_value_t = 64)]
    block_size: u32,

    /// Interval between mosaic pattern changes (seconds)
    #[arg(short = 'i', long, default_value_t = 0.5)]
    change_interval: f64,

    /// Pattern mode: perlin, plasma, cellular, wave, fbm, warp, lissajous, kaleidoscope, spiral, checker, walker, metaballs, julia, truchet, matrix, fire, moire, or random
    #[arg(short, long, default_value = "random")]
    pattern: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Validate input file exists
    if !args.input.exists() {
        anyhow::bail!("Input file does not exist: {}", args.input.display());
    }

    // Validate input file is MP3
    match args.input.extension().and_then(|e| e.to_str()) {
        Some("mp3") | Some("MP3") => {}
        _ => {
            anyhow::bail!("Input file must be an MP3 file: {}", args.input.display());
        }
    }

    // Parse pattern mode
    let pattern_mode = PatternMode::from_str(&args.pattern)
        .ok_or_else(|| anyhow::anyhow!(
            "Invalid pattern mode '{}'. Available: perlin, plasma, cellular, wave, fbm, warp, lissajous, kaleidoscope, spiral, checker, walker, metaballs, julia, truchet, matrix, fire, moire, random",
            args.pattern
        ))?;

    println!("mosaic-video - MP3 to Video Converter");
    println!("======================================");
    println!();
    println!("Input:  {}", args.input.display());
    println!("Output: {}", args.output.display());
    println!();

    // Get audio information
    println!("Analyzing audio file...");
    let audio_info = get_audio_info(&args.input)
        .context("Failed to get audio information")?;

    println!("  Duration:    {:.2} seconds", audio_info.duration_secs);
    println!("  Sample rate: {} Hz", audio_info.sample_rate);
    println!("  Channels:    {}", audio_info.channels);
    println!();

    // Create encoder configuration
    let config = EncoderConfig::new(
        args.width,
        args.height,
        args.fps,
        args.block_size,
        args.change_interval,
    ).with_pattern_mode(pattern_mode);

    println!("Video settings:");
    println!("  Resolution:  {}x{}", config.width, config.height);
    println!("  FPS:         {}", config.fps);
    println!("  Block size:  {}px", config.block_size);
    println!("  Change interval: {:.2}s", config.change_interval);
    println!("  Pattern:     {}", pattern_mode);
    println!();

    // Create the video
    println!("Generating video...");
    create_video(&args.input, &args.output, &config, &audio_info)
        .context("Failed to create video")?;

    println!();
    println!("Done! Video saved to: {}", args.output.display());

    Ok(())
}
