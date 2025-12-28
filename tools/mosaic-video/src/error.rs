use thiserror::Error;

/// Errors that can occur during mosaic-video processing
#[derive(Error, Debug)]
pub enum MosaicError {
    /// Failed to open or read the input audio file
    #[error("Failed to open audio file: {0}")]
    AudioOpenError(String),

    /// Failed to probe audio format
    #[error("Failed to probe audio format: {0}")]
    AudioProbeError(String),

    /// Failed to get audio duration
    #[error("Failed to get audio duration: {0}")]
    AudioDurationError(String),

    /// Failed to initialize video encoder
    #[error("Failed to initialize video encoder: {0}")]
    EncoderInitError(String),

    /// Failed to encode video frame
    #[error("Failed to encode video frame: {0}")]
    FrameEncodeError(String),

    /// Failed to finalize video encoding
    #[error("Failed to finalize video: {0}")]
    VideoFinalizeError(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type alias using MosaicError
pub type Result<T> = std::result::Result<T, MosaicError>;
