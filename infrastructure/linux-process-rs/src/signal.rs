//! Signal handling module with safe abstractions

use crate::errors::{ProcessError, ProcessResult};
use signal_hook::{consts::signal::*, iterator::Signals};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

/// Signal types supported by the handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalType {
    /// Interrupt signal (Ctrl+C)
    Interrupt,
    /// Termination signal
    Terminate,
    /// Hangup signal
    Hangup,
    /// Quit signal
    Quit,
    /// User-defined signal 1
    User1,
    /// User-defined signal 2
    User2,
}

impl SignalType {
    /// Convert to signal constant
    fn to_signal(self) -> i32 {
        match self {
            Self::Interrupt => SIGINT,
            Self::Terminate => SIGTERM,
            Self::Hangup => SIGHUP,
            Self::Quit => SIGQUIT,
            Self::User1 => SIGUSR1,
            Self::User2 => SIGUSR2,
        }
    }

    /// Create from signal number
    fn from_signal(sig: i32) -> Option<Self> {
        match sig {
            SIGINT => Some(Self::Interrupt),
            SIGTERM => Some(Self::Terminate),
            SIGHUP => Some(Self::Hangup),
            SIGQUIT => Some(Self::Quit),
            SIGUSR1 => Some(Self::User1),
            SIGUSR2 => Some(Self::User2),
            _ => None,
        }
    }
}

/// Signal handler with graceful shutdown support
pub struct SignalHandler {
    #[allow(dead_code)]
    signals: Vec<SignalType>,
    shutdown: Arc<AtomicBool>,
    stop_flag: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl SignalHandler {
    /// Create a new signal handler
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use linux_process_rs::signal::{SignalHandler, SignalType};
    ///
    /// let handler = SignalHandler::new(&[
    ///     SignalType::Interrupt,
    ///     SignalType::Terminate,
    /// ]).expect("Failed to create signal handler");
    ///
    /// // Check if shutdown was requested
    /// if handler.should_shutdown() {
    ///     println!("Shutdown requested");
    /// }
    /// ```
    pub fn new(signals: &[SignalType]) -> ProcessResult<Self> {
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();

        let signal_nums: Vec<i32> = signals.iter().map(|s| (*s).to_signal()).collect();
        let mut sig_handler =
            Signals::new(&signal_nums).map_err(|e| ProcessError::SignalError(e.to_string()))?;

        let signals_vec = signals.to_vec();
        let handle = thread::spawn(move || {
            loop {
                // Check if we should stop the handler thread
                if stop_flag_clone.load(Ordering::SeqCst) {
                    break;
                }

                // Wait for signals with a timeout to allow checking stop flag
                if let Some(sig) = sig_handler.pending().next() {
                    if let Some(signal_type) = SignalType::from_signal(sig) {
                        eprintln!("Received signal: {:?}", signal_type);
                        shutdown_clone.store(true, Ordering::SeqCst);

                        // Handle specific signals differently if needed
                        match signal_type {
                            SignalType::Interrupt | SignalType::Terminate => {
                                // Graceful shutdown
                                break;
                            }
                            _ => {
                                // Continue handling other signals
                            }
                        }
                    }
                } else {
                    // No signal pending, sleep briefly
                    thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        });

        Ok(Self {
            signals: signals_vec,
            shutdown,
            stop_flag,
            handle: Some(handle),
        })
    }

    /// Check if shutdown has been requested
    pub fn should_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::SeqCst)
    }

    /// Reset the shutdown flag
    pub fn reset(&self) {
        self.shutdown.store(false, Ordering::SeqCst);
    }

    /// Wait for a signal to be received
    pub fn wait_for_signal(&self) {
        while !self.should_shutdown() {
            thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    /// Get a clone of the shutdown flag
    pub fn shutdown_flag(&self) -> Arc<AtomicBool> {
        self.shutdown.clone()
    }
}

impl Drop for SignalHandler {
    fn drop(&mut self) {
        // Signal the handler thread to stop
        self.stop_flag.store(true, Ordering::SeqCst);

        // Wait for the handler thread to finish
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Send a signal to a process (Unix only)
#[cfg(unix)]
pub fn send_signal(pid: u32, signal: SignalType) -> ProcessResult<()> {
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;

    let nix_signal = match signal {
        SignalType::Interrupt => Signal::SIGINT,
        SignalType::Terminate => Signal::SIGTERM,
        SignalType::Hangup => Signal::SIGHUP,
        SignalType::Quit => Signal::SIGQUIT,
        SignalType::User1 => Signal::SIGUSR1,
        SignalType::User2 => Signal::SIGUSR2,
    };

    kill(Pid::from_raw(pid as i32), nix_signal)
        .map_err(|e| ProcessError::SignalError(e.to_string()))?;

    Ok(())
}

/// Send a signal to a process group (Unix only)
#[cfg(unix)]
pub fn send_signal_to_group(pgid: u32, signal: SignalType) -> ProcessResult<()> {
    use nix::sys::signal::{killpg, Signal};
    use nix::unistd::Pid;

    let nix_signal = match signal {
        SignalType::Interrupt => Signal::SIGINT,
        SignalType::Terminate => Signal::SIGTERM,
        SignalType::Hangup => Signal::SIGHUP,
        SignalType::Quit => Signal::SIGQUIT,
        SignalType::User1 => Signal::SIGUSR1,
        SignalType::User2 => Signal::SIGUSR2,
    };

    killpg(Pid::from_raw(pgid as i32), nix_signal)
        .map_err(|e| ProcessError::SignalError(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_type_conversion() {
        assert_eq!(SignalType::Interrupt.to_signal(), SIGINT);
        assert_eq!(SignalType::from_signal(SIGINT), Some(SignalType::Interrupt));
        assert_eq!(SignalType::from_signal(999), None);
    }

    #[test]
    fn test_signal_handler_creation() {
        // シグナルハンドラの作成のみテスト（実際のシグナル待機はしない）
        let handler = SignalHandler::new(&[SignalType::Interrupt]);
        assert!(handler.is_ok());
        if let Ok(h) = handler {
            assert!(!h.should_shutdown());
            // 即座にドロップしてクリーンアップ
        }
    }
}
