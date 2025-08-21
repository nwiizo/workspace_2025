use nix::sys::stat::{umask, Mode};
use nix::unistd::{chdir, close, dup2, fork, setsid, ForkResult};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

/// Daemon process example using nix
///
/// Demonstrates creating a proper Unix daemon using the
/// double-fork technique. The daemon:
/// - Detaches from the terminal
/// - Runs in the background
/// - Redirects stdio to /dev/null
/// - Creates a PID file
/// - Logs to a file
///
/// # Safety
///
/// Uses unsafe fork() calls but follows proper daemon
/// creation patterns for safety.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Daemon Creation Example with Nix ===\n");
    println!("Starting daemon process...");
    
    // Daemonize the process
    daemonize()?;
    
    // From here on, we're running as a daemon
    daemon_main()?;
    
    Ok(())
}

/// Creates a daemon process using the double-fork technique
///
/// This technique ensures:
/// 1. The daemon is not a session leader (can't acquire a controlling terminal)
/// 2. The daemon is properly detached from the parent
/// 3. No zombie processes are created
fn daemonize() -> nix::Result<()> {
    // First fork - parent exits, child continues
    match unsafe { fork()? } {
        ForkResult::Parent { .. } => {
            // Parent exits immediately
            println!("Parent process exiting, daemon continuing in background...");
            std::process::exit(0);
        }
        ForkResult::Child => {
            // Continue as child
        }
    }
    
    // Create new session - child becomes session leader
    setsid()?;
    
    // Second fork - ensures we can't acquire a controlling terminal
    match unsafe { fork()? } {
        ForkResult::Parent { .. } => {
            // First child exits
            std::process::exit(0);
        }
        ForkResult::Child => {
            // Continue as grandchild (the actual daemon)
        }
    }
    
    // Set working directory to root to avoid blocking unmounts
    chdir("/")?;
    
    // Set file creation mask
    umask(Mode::from_bits_truncate(0o027));
    
    // Redirect standard file descriptors to /dev/null
    redirect_stdio()?;
    
    Ok(())
}

/// Redirects stdin, stdout, and stderr to /dev/null
fn redirect_stdio() -> nix::Result<()> {
    // Open /dev/null
    let dev_null = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/null")
        .expect("Failed to open /dev/null");
    
    let dev_null_fd = dev_null.as_raw_fd();
    
    // Redirect stdin (fd 0)
    dup2(dev_null_fd, 0)?;
    
    // Redirect stdout (fd 1)
    dup2(dev_null_fd, 1)?;
    
    // Redirect stderr (fd 2)
    dup2(dev_null_fd, 2)?;
    
    // Close the original /dev/null fd if it's not one of the standard fds
    if dev_null_fd > 2 {
        close(dev_null_fd)?;
    }
    
    Ok(())
}

/// Main daemon logic
fn daemon_main() -> Result<(), Box<dyn std::error::Error>> {
    // Create PID file
    let pid_file_path = "/tmp/rust_daemon.pid";
    create_pid_file(pid_file_path)?;
    
    // Set up logging
    let log_file_path = "/tmp/rust_daemon.log";
    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)?;
    
    // Log startup
    log_message(&mut log_file, "Daemon started successfully")?;
    log_message(&mut log_file, &format!("PID: {}", std::process::id()))?;
    
    // Main daemon loop
    let mut iteration = 0;
    loop {
        iteration += 1;
        
        // Perform daemon work
        log_message(&mut log_file, &format!("Daemon iteration {}", iteration))?;
        
        // Simulate some work
        perform_daemon_work(&mut log_file)?;
        
        // Sleep before next iteration
        sleep(Duration::from_secs(10));
        
        // Check if we should stop (for demo, stop after 5 iterations)
        if iteration >= 5 {
            log_message(&mut log_file, "Daemon stopping after 5 iterations")?;
            break;
        }
        
        // In a real daemon, you might check for:
        // - Configuration reload signals (SIGHUP)
        // - Graceful shutdown signals (SIGTERM)
        // - Other control signals
    }
    
    // Cleanup
    cleanup(pid_file_path)?;
    log_message(&mut log_file, "Daemon shutdown complete")?;
    
    Ok(())
}

/// Creates a PID file for the daemon
fn create_pid_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut pid_file = File::create(path)?;
    writeln!(pid_file, "{}", std::process::id())?;
    Ok(())
}

/// Logs a message with timestamp
fn log_message(file: &mut File, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    
    writeln!(file, "[{}] {}", timestamp, message)?;
    file.flush()?;
    Ok(())
}

/// Performs the actual daemon work
fn perform_daemon_work(log_file: &mut File) -> Result<(), Box<dyn std::error::Error>> {
    // This is where your daemon would do its actual work
    // For example:
    // - Monitor system resources
    // - Process queued jobs
    // - Handle network connections
    // - Watch for file changes
    
    log_message(log_file, "  Checking system status...")?;
    sleep(Duration::from_secs(1));
    
    log_message(log_file, "  Processing queue...")?;
    sleep(Duration::from_secs(1));
    
    log_message(log_file, "  Work cycle complete")?;
    
    Ok(())
}

/// Cleanup function for daemon shutdown
fn cleanup(pid_file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Remove PID file
    if Path::new(pid_file_path).exists() {
        std::fs::remove_file(pid_file_path)?;
    }
    
    // Close any open resources
    // Flush any buffers
    // Save state if needed
    
    Ok(())
}

/// Alternative: Simple daemon for testing
///
/// This version stays in foreground for easier testing
#[allow(dead_code)]
fn simple_daemon_for_testing() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running in foreground mode for testing...");
    println!("PID: {}", std::process::id());
    println!("Check /tmp/rust_daemon.log for output");
    
    // Run daemon main without forking
    daemon_main()?;
    
    Ok(())
}