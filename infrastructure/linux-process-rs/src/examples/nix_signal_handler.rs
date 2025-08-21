use nix::sys::signal::{self, Signal, SigHandler, SigSet};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{fork, ForkResult, Pid};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::thread::sleep;
use std::time::Duration;

// Global flags for signal handling (must be atomic for signal safety)
static SIGCHLD_RECEIVED: AtomicI32 = AtomicI32::new(0);
static SIGINT_RECEIVED: AtomicBool = AtomicBool::new(false);

/// Signal handling example with nix
///
/// Demonstrates proper signal handling including:
/// - Setting up signal handlers
/// - Handling SIGCHLD to prevent zombie processes
/// - Graceful shutdown on SIGINT
/// - Signal masking
///
/// # Safety
///
/// Signal handlers must only call async-signal-safe functions.
/// Using atomics for communication between signal handlers and main code.
extern "C" fn handle_sigchld(_: i32) {
    // Increment counter - this is async-signal-safe
    SIGCHLD_RECEIVED.fetch_add(1, Ordering::SeqCst);
    
    // Reap all available children without blocking
    // This prevents zombie processes
    loop {
        match waitpid(None, Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::Exited(pid, _)) |
            Ok(WaitStatus::Signaled(pid, _, _)) => {
                // Child reaped successfully
                // We can't use println! here as it's not async-signal-safe
                // Just store the PID if needed
                let _ = pid; // Suppress unused warning
            }
            Ok(WaitStatus::StillAlive) | Err(_) => {
                // No more children to reap or error
                break;
            }
            _ => {
                // Other status, continue
                continue;
            }
        }
    }
}

extern "C" fn handle_sigint(_: i32) {
    SIGINT_RECEIVED.store(true, Ordering::SeqCst);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Signal Handling Example with Nix ===\n");
    println!("Press Ctrl+C to trigger graceful shutdown\n");
    
    // Set up signal handlers
    setup_signal_handlers()?;
    
    // Create some child processes
    create_child_processes(3)?;
    
    // Main loop - monitor signals
    monitor_signals()?;
    
    println!("\nClean shutdown completed");
    Ok(())
}

/// Sets up signal handlers for SIGCHLD and SIGINT
///
/// # Safety
///
/// Signal handlers are set up using unsafe blocks because they
/// can interrupt normal program flow at any time.
fn setup_signal_handlers() -> nix::Result<()> {
    println!("Setting up signal handlers...");
    
    // Handle SIGCHLD to prevent zombie processes
    unsafe {
        signal::signal(Signal::SIGCHLD, SigHandler::Handler(handle_sigchld))?;
    }
    
    // Handle SIGINT for graceful shutdown
    unsafe {
        signal::signal(Signal::SIGINT, SigHandler::Handler(handle_sigint))?;
    }
    
    // Ignore SIGPIPE (common for network programs)
    unsafe {
        signal::signal(Signal::SIGPIPE, SigHandler::SigIgn)?;
    }
    
    println!("Signal handlers installed\n");
    Ok(())
}

/// Creates child processes that run for different durations
fn create_child_processes(count: usize) -> Result<(), Box<dyn std::error::Error>> {
    for i in 0..count {
        match unsafe { fork()? } {
            ForkResult::Parent { child } => {
                println!("Created child process {} with PID: {}", i, child);
            }
            ForkResult::Child => {
                // Child process work
                child_work(i);
                std::process::exit(0);
            }
        }
        
        // Small delay between creating children
        sleep(Duration::from_millis(100));
    }
    
    Ok(())
}

/// Work performed by child processes
fn child_work(id: usize) {
    let duration = Duration::from_secs((id + 1) as u64 * 2);
    println!("  Child {}: Working for {:?}...", id, duration);
    sleep(duration);
    println!("  Child {}: Work completed!", id);
}

/// Monitors signals and handles them appropriately
fn monitor_signals() -> Result<(), Box<dyn std::error::Error>> {
    println!("Monitoring signals (children will exit over time)...\n");
    
    let mut children_reaped = 0;
    
    loop {
        // Check if SIGINT was received
        if SIGINT_RECEIVED.load(Ordering::SeqCst) {
            println!("\n\nSIGINT received! Starting graceful shutdown...");
            graceful_shutdown()?;
            break;
        }
        
        // Check if any children were reaped
        let sigchld_count = SIGCHLD_RECEIVED.swap(0, Ordering::SeqCst);
        if sigchld_count > 0 {
            children_reaped += sigchld_count;
            println!("Main: SIGCHLD received {} time(s). Total children reaped: {}", 
                     sigchld_count, children_reaped);
        }
        
        // Small sleep to avoid busy waiting
        sleep(Duration::from_millis(100));
        
        // Optional: Exit after all children are done
        if children_reaped >= 3 {
            println!("\nAll children completed. Exiting normally.");
            break;
        }
    }
    
    Ok(())
}

/// Performs graceful shutdown when SIGINT is received
fn graceful_shutdown() -> Result<(), Box<dyn std::error::Error>> {
    println!("Performing cleanup...");
    
    // Send SIGTERM to all remaining children
    send_signal_to_children(Signal::SIGTERM)?;
    
    // Wait a bit for children to exit gracefully
    sleep(Duration::from_secs(1));
    
    // Force kill any remaining children
    send_signal_to_children(Signal::SIGKILL)?;
    
    // Final reap of any remaining children
    while let Ok(status) = waitpid(None, Some(WaitPidFlag::WNOHANG)) {
        match status {
            WaitStatus::Exited(pid, _) | WaitStatus::Signaled(pid, _, _) => {
                println!("Reaped child PID: {} during shutdown", pid);
            }
            WaitStatus::StillAlive => break,
            _ => continue,
        }
    }
    
    println!("Graceful shutdown completed");
    Ok(())
}

/// Sends a signal to all child processes
fn send_signal_to_children(sig: Signal) -> nix::Result<()> {
    // In a real application, you would track child PIDs
    // Here we use process group signaling as an example
    
    // Send signal to process group (all children)
    match signal::kill(Pid::from_raw(0), sig) {
        Ok(_) => println!("Sent {:?} to all children", sig),
        Err(e) => eprintln!("Failed to send signal: {}", e),
    }
    
    Ok(())
}

/// Demonstrates signal masking (blocking signals temporarily)
#[allow(dead_code)]
fn demonstrate_signal_masking() -> nix::Result<()> {
    println!("Demonstrating signal masking...");
    
    // Create a signal set with SIGINT
    let mut sigset = SigSet::empty();
    sigset.add(Signal::SIGINT);
    
    // Block SIGINT temporarily
    signal::sigprocmask(signal::SigmaskHow::SIG_BLOCK, Some(&sigset), None)?;
    println!("SIGINT blocked - Ctrl+C won't work for 3 seconds");
    
    sleep(Duration::from_secs(3));
    
    // Unblock SIGINT
    signal::sigprocmask(signal::SigmaskHow::SIG_UNBLOCK, Some(&sigset), None)?;
    println!("SIGINT unblocked - Ctrl+C works again");
    
    Ok(())
}