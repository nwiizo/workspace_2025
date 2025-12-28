use std::fs::File;
use std::path::Path;

use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::error::{MosaicError, Result};

/// Audio metadata extracted from an MP3 file
#[derive(Debug, Clone)]
pub struct AudioInfo {
    /// Duration of the audio in seconds
    pub duration_secs: f64,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u32,
}

/// Get audio information from an MP3 file
///
/// # Arguments
///
/// * `path` - Path to the MP3 file
///
/// # Returns
///
/// AudioInfo containing duration, sample rate, and channel count
///
/// # Errors
///
/// Returns `MosaicError::AudioOpenError` if the file cannot be opened
/// Returns `MosaicError::AudioProbeError` if the format cannot be detected
/// Returns `MosaicError::AudioDurationError` if the duration cannot be determined
pub fn get_audio_info<P: AsRef<Path>>(path: P) -> Result<AudioInfo> {
    let path = path.as_ref();

    // Open the media source
    let file = File::open(path).map_err(|e| {
        MosaicError::AudioOpenError(format!("{}: {}", path.display(), e))
    })?;

    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    // Create a hint to help the format registry guess what format reader is appropriate
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    // Probe the media source for the format
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| MosaicError::AudioProbeError(e.to_string()))?;

    let format = probed.format;

    // Get the default track
    let track = format
        .default_track()
        .ok_or_else(|| MosaicError::AudioDurationError("No audio track found".to_string()))?;

    // Get codec parameters
    let codec_params = &track.codec_params;

    let sample_rate = codec_params
        .sample_rate
        .ok_or_else(|| MosaicError::AudioDurationError("No sample rate found".to_string()))?;

    let channels = codec_params
        .channels
        .map(|c| c.count() as u32)
        .unwrap_or(2);

    // Calculate duration
    let duration_secs = if let Some(n_frames) = codec_params.n_frames {
        n_frames as f64 / sample_rate as f64
    } else {
        // Fallback: decode the entire file to get duration
        get_duration_by_decoding(path)?
    };

    Ok(AudioInfo {
        duration_secs,
        sample_rate,
        channels,
    })
}

/// Get duration by decoding the entire file (fallback method)
fn get_duration_by_decoding<P: AsRef<Path>>(path: P) -> Result<f64> {
    let path = path.as_ref();
    let file = File::open(path).map_err(|e| {
        MosaicError::AudioOpenError(format!("{}: {}", path.display(), e))
    })?;

    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| MosaicError::AudioProbeError(e.to_string()))?;

    let mut format = probed.format;

    let track = format
        .default_track()
        .ok_or_else(|| MosaicError::AudioDurationError("No audio track found".to_string()))?;

    let track_id = track.id;
    let sample_rate = track
        .codec_params
        .sample_rate
        .ok_or_else(|| MosaicError::AudioDurationError("No sample rate found".to_string()))?;

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| MosaicError::AudioDurationError(format!("Failed to create decoder: {}", e)))?;

    let mut total_samples: u64 = 0;

    loop {
        match format.next_packet() {
            Ok(packet) => {
                if packet.track_id() == track_id {
                    if let Ok(decoded) = decoder.decode(&packet) {
                        total_samples += decoded.frames() as u64;
                    }
                }
            }
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(_) => break,
        }
    }

    Ok(total_samples as f64 / sample_rate as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_info_struct() {
        let info = AudioInfo {
            duration_secs: 180.0,
            sample_rate: 44100,
            channels: 2,
        };
        assert_eq!(info.duration_secs, 180.0);
        assert_eq!(info.sample_rate, 44100);
        assert_eq!(info.channels, 2);
    }
}
