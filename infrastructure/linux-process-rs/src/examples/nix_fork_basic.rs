use nix::sys::wait::waitpid;
use nix::unistd::{fork, ForkResult};
use std::thread::sleep;
use std::time::Duration;

/// Basic fork example using nix crate
///
/// Demonstrates the simplest use of fork() with nix,
/// showing parent-child process separation and proper
/// child reaping.
///
/// # Safety
///
/// fork() is unsafe because it creates a new process
/// that shares the parent's memory space initially.
/// The child should only call async-signal-safe functions
/// before exec or exit.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic Fork Example with Nix ===\n");
    println!("Parent PID: {}", std::process::id());

    // Simple fork example
    match unsafe { fork()? } {
        ForkResult::Parent { child } => {
            println!("Parent: Created child with PID: {}", child);
            println!("Parent: Waiting for child to complete...");
            
            // Wait for the specific child process
            let status = waitpid(child, None)?;
            
            println!("Parent: Child process status: {:?}", status);
            
            // Check exit status
            match status {
                nix::sys::wait::WaitStatus::Exited(pid, code) => {
                    println!("Parent: Child {} exited with code {}", pid, code);
                }
                nix::sys::wait::WaitStatus::Signaled(pid, sig, _) => {
                    println!("Parent: Child {} killed by signal {:?}", pid, sig);
                }
                _ => {
                    println!("Parent: Child had other status: {:?}", status);
                }
            }
        }
        ForkResult::Child => {
            // Child process
            println!("Child: I'm a new process with PID: {}", std::process::id());
            println!("Child: My parent PID: {}", nix::unistd::getppid());
            
            // Simulate some work
            println!("Child: Doing some work...");
            sleep(Duration::from_secs(2));
            
            println!("Child: Work completed, exiting.");
            // Exit with success code
            std::process::exit(0);
        }
    }

    println!("\nParent: All done!");
    Ok(())
}