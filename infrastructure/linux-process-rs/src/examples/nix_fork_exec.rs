use nix::sys::wait::waitpid;
use nix::unistd::{execvp, fork, ForkResult};
use std::ffi::CString;

/// Fork and exec example using nix
///
/// Demonstrates the common fork-exec pattern where a child
/// process is created and then replaced with a different program.
/// This is the safest pattern for multi-threaded programs.
///
/// # Safety
///
/// The exec family of functions replaces the current process
/// image, so it's safe to use even in multi-threaded contexts
/// after fork.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Fork and Exec Example with Nix ===\n");
    
    // Example 1: Execute a simple command
    execute_command("echo", &["Hello", "from", "exec!"])?;
    
    // Example 2: Execute ls with arguments
    execute_command("ls", &["-la", "/tmp"])?;
    
    // Example 3: Execute with shell
    execute_shell_command("echo $HOME && date")?;
    
    Ok(())
}

/// Executes a command by forking and using exec
///
/// # Arguments
///
/// * `command` - The command to execute
/// * `args` - Arguments to pass to the command
///
/// # Returns
///
/// Returns Ok(()) if the command executed successfully
///
/// # Example
///
/// ```
/// execute_command("ls", &["-l", "/home"])?;
/// ```
fn execute_command(command: &str, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    println!("Executing: {} {}", command, args.join(" "));
    
    match unsafe { fork()? } {
        ForkResult::Parent { child } => {
            // Parent process: wait for child
            let status = waitpid(child, None)?;
            match status {
                nix::sys::wait::WaitStatus::Exited(_, code) => {
                    if code == 0 {
                        println!("Command completed successfully\n");
                    } else {
                        println!("Command exited with code: {}\n", code);
                    }
                }
                nix::sys::wait::WaitStatus::Signaled(_, sig, _) => {
                    println!("Command terminated by signal: {:?}\n", sig);
                }
                _ => {
                    println!("Command ended with status: {:?}\n", status);
                }
            }
        }
        ForkResult::Child => {
            // Child process: execute the command
            
            // Prepare command and arguments as CStrings
            let command_cstr = CString::new(command).expect("CString::new failed");
            
            // Build argument vector including the command name as first arg
            let mut arg_vec: Vec<CString> = vec![command_cstr.clone()];
            for arg in args {
                arg_vec.push(CString::new(*arg).expect("CString::new failed"));
            }
            
            // Execute the command - this replaces the current process
            match execvp(&command_cstr, &arg_vec) {
                Ok(_) => {
                    // This should never be reached if exec succeeds
                    unreachable!("execvp returned Ok, which should not happen");
                }
                Err(err) => {
                    // exec failed - exit child with error
                    eprintln!("Failed to execute {}: {}", command, err);
                    std::process::exit(1);
                }
            }
        }
    }
    
    Ok(())
}

/// Executes a shell command using sh -c
///
/// # Arguments
///
/// * `command` - The shell command string to execute
///
/// # Example
///
/// ```
/// execute_shell_command("ls -l | grep '.rs' | wc -l")?;
/// ```
fn execute_shell_command(command: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Executing shell command: {}", command);
    
    match unsafe { fork()? } {
        ForkResult::Parent { child } => {
            // Parent: wait for child
            let status = waitpid(child, None)?;
            match status {
                nix::sys::wait::WaitStatus::Exited(_, code) => {
                    if code == 0 {
                        println!("Shell command completed successfully\n");
                    } else {
                        println!("Shell command exited with code: {}\n", code);
                    }
                }
                _ => {
                    println!("Shell command ended with status: {:?}\n", status);
                }
            }
        }
        ForkResult::Child => {
            // Child: execute shell command
            let sh = CString::new("sh").unwrap();
            let flag = CString::new("-c").unwrap();
            let cmd = CString::new(command).unwrap();
            
            let args = vec![sh.clone(), flag, cmd];
            
            match execvp(&sh, &args) {
                Ok(_) => unreachable!(),
                Err(err) => {
                    eprintln!("Failed to execute shell: {}", err);
                    std::process::exit(1);
                }
            }
        }
    }
    
    Ok(())
}