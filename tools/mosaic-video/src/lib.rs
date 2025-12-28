pub mod audio;
pub mod encoder;
pub mod error;
pub mod mosaic;

pub use audio::{get_audio_info, AudioInfo};
pub use encoder::{create_video, EncoderConfig};
pub use error::{MosaicError, Result};
pub use mosaic::{Color, MosaicConfig, MosaicGenerator, PatternMode};
