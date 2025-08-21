//! Error handling module following Rust best practices
//!
//! Uses `thiserror` for library errors with detailed error types
//! that consumers can match on and handle appropriately.

use std::io;
use thiserror::Error;

/// Custom error type for process operations
#[derive(Error, Debug)]
pub enum ProcessError {
    /// IO operation failed
    #[error("IO operation failed: {0}")]
    Io(#[from] io::Error),

    /// Failed to spawn process
    #[error("Failed to spawn process: {reason}")]
    SpawnError { reason: String },

    /// Invalid input provided
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Signal handling error
    #[error("Signal handling error: {0}")]
    SignalError(String),

    /// Process timeout
    #[error("Process timed out after {seconds} seconds")]
    TimeoutError { seconds: u64 },

    /// Permission denied
    #[error("Permission denied: {context}")]
    PermissionDenied { context: String },

    /// Fork operation failed
    #[cfg(unix)]
    #[error("Fork failed: {0}")]
    ForkError(#[from] nix::Error),

    /// Resource limit error
    #[error("Resource limit error: {message}")]
    ResourceLimitError { message: String },

    /// Process already terminated
    #[error("Process {pid} already terminated")]
    ProcessTerminated { pid: u32 },
}

/// Result type alias for process operations
pub type ProcessResult<T> = Result<T, ProcessError>;

/// Error context extension trait for adding context to errors
pub trait ErrorContext<T> {
    /// Add context to an error
    fn context<C>(self, context: C) -> ProcessResult<T>
    where
        C: Into<String>;

    /// Add context lazily (only evaluated on error)
    fn with_context<C, F>(self, f: F) -> ProcessResult<T>
    where
        F: FnOnce() -> C,
        C: Into<String>;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: Into<ProcessError>,
{
    fn context<C>(self, context: C) -> ProcessResult<T>
    where
        C: Into<String>,
    {
        self.map_err(|e| {
            let base_error = e.into();
            ProcessError::InvalidInput(format!("{}: {}", context.into(), base_error))
        })
    }

    fn with_context<C, F>(self, f: F) -> ProcessResult<T>
    where
        F: FnOnce() -> C,
        C: Into<String>,
    {
        self.map_err(|e| {
            let base_error = e.into();
            ProcessError::InvalidInput(format!("{}: {}", f().into(), base_error))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let process_error: ProcessError = io_error.into();
        assert!(matches!(process_error, ProcessError::Io(_)));
    }

    #[test]
    fn test_error_context() {
        let result: Result<(), io::Error> = Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "access denied",
        ));

        let with_context = result.context("Failed to open file");
        assert!(with_context.is_err());
        let error = with_context.unwrap_err();
        assert!(error.to_string().contains("Failed to open file"));
    }
}
