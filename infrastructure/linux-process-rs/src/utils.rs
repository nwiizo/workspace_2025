//! Utility functions for process management

use crate::errors::ProcessResult;
use std::time::{Duration, Instant};

/// Retry configuration for operations
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of attempts
    pub max_attempts: u32,
    /// Delay between attempts
    pub delay: Duration,
    /// Whether to use exponential backoff
    pub exponential_backoff: bool,
    /// Maximum delay for exponential backoff
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            delay: Duration::from_millis(100),
            exponential_backoff: false,
            max_delay: Duration::from_secs(10),
        }
    }
}

/// Retry an operation with the given configuration
pub fn retry_with_config<T, F, E>(config: &RetryConfig, mut operation: F) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    let mut delay = config.delay;

    for attempt in 1..=config.max_attempts {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) if attempt == config.max_attempts => return Err(e),
            Err(_) => {
                std::thread::sleep(delay);

                if config.exponential_backoff {
                    delay = std::cmp::min(delay * 2, config.max_delay);
                }
            }
        }
    }

    unreachable!()
}

/// Measure the execution time of a closure
pub fn measure_time<T, F>(operation: F) -> (T, Duration)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = operation();
    let duration = start.elapsed();
    (result, duration)
}

/// Rate limiter for operations
pub struct RateLimiter {
    max_per_second: u32,
    window_start: Instant,
    count: u32,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(max_per_second: u32) -> Self {
        Self {
            max_per_second,
            window_start: Instant::now(),
            count: 0,
        }
    }

    /// Check if an operation is allowed
    pub fn allow(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.window_start);

        if elapsed >= Duration::from_secs(1) {
            // Reset window
            self.window_start = now;
            self.count = 1;
            true
        } else if self.count < self.max_per_second {
            self.count += 1;
            true
        } else {
            false
        }
    }

    /// Wait until an operation is allowed
    pub fn wait_if_needed(&mut self) {
        if !self.allow() {
            let elapsed = Instant::now().duration_since(self.window_start);
            let remaining = Duration::from_secs(1).saturating_sub(elapsed);
            std::thread::sleep(remaining);
            self.window_start = Instant::now();
            self.count = 1;
        }
    }
}

/// Convert a string to a C-compatible string
#[cfg(unix)]
pub fn to_cstring(s: &str) -> ProcessResult<std::ffi::CString> {
    std::ffi::CString::new(s)
        .map_err(|e| crate::errors::ProcessError::InvalidInput(format!("Invalid C string: {}", e)))
}

/// Platform-specific process utilities
#[cfg(unix)]
pub mod unix {
    use crate::errors::{ProcessError, ProcessResult};

    /// Get the current process ID
    pub fn get_pid() -> u32 {
        std::process::id()
    }

    /// Get the parent process ID
    pub fn get_ppid() -> ProcessResult<u32> {
        use nix::unistd::getppid;
        Ok(getppid().as_raw() as u32)
    }

    /// Set process priority (nice value)
    #[cfg(target_os = "linux")]
    pub fn set_priority(priority: i32) -> ProcessResult<()> {
        unsafe {
            let result = libc::nice(priority);
            if result == -1 {
                let err = std::io::Error::last_os_error();
                return Err(ProcessError::PermissionDenied {
                    context: format!("Failed to set priority: {}", err),
                });
            }
        }
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    pub fn set_priority(_priority: i32) -> ProcessResult<()> {
        Err(ProcessError::ResourceLimitError {
            message: "Setting process priority is not supported on this platform".into(),
        })
    }

    /// Get process resource usage
    pub fn get_resource_usage() -> ProcessResult<ResourceUsage> {
        use nix::sys::resource::{getrusage, UsageWho};

        let usage =
            getrusage(UsageWho::RUSAGE_SELF).map_err(|e| ProcessError::ResourceLimitError {
                message: e.to_string(),
            })?;

        Ok(ResourceUsage {
            user_time_secs: usage.user_time().tv_sec() as u64,
            system_time_secs: usage.system_time().tv_sec() as u64,
            max_rss_kb: usage.max_rss(),
        })
    }

    /// Resource usage information
    #[derive(Debug, Clone)]
    pub struct ResourceUsage {
        pub user_time_secs: u64,
        pub system_time_secs: u64,
        pub max_rss_kb: i64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert!(!config.exponential_backoff);
    }

    #[test]
    fn test_retry_success() {
        let mut attempt = 0;
        let result = retry_with_config(
            &RetryConfig {
                max_attempts: 3,
                delay: Duration::from_millis(10),
                ..Default::default()
            },
            || {
                attempt += 1;
                if attempt == 2 {
                    Ok(42)
                } else {
                    Err("not yet")
                }
            },
        );

        assert_eq!(result, Ok(42));
        assert_eq!(attempt, 2);
    }

    #[test]
    fn test_measure_time() {
        let (result, duration) = measure_time(|| {
            std::thread::sleep(Duration::from_millis(10));
            42
        });

        assert_eq!(result, 42);
        assert!(duration >= Duration::from_millis(10));
    }

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(2);

        assert!(limiter.allow());
        assert!(limiter.allow());
        assert!(!limiter.allow()); // Should be rate limited
    }
}
