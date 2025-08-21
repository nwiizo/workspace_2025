//! Linux Process Management Library for Rust
//!
//! This library provides comprehensive tools and examples for managing Linux processes
//! in Rust, including process spawning, signal handling, zombie prevention, and more.

pub mod errors;
pub mod process;
pub mod process_guard;
pub mod process_pool;
pub mod signal;
pub mod utils;

// Re-export commonly used types
pub use errors::{ProcessError, ProcessResult};
pub use process::ProcessBuilder;
pub use process_guard::ProcessGuard;
pub use process_pool::ProcessPool;
pub use signal::{SignalHandler, SignalType};
