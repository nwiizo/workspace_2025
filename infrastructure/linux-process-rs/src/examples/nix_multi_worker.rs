use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{fork, ForkResult, Pid};
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

/// Multiple worker processes example using nix
///
/// Creates multiple child processes (workers) and manages them
/// properly, demonstrating:
/// - Creating multiple children
/// - Tracking child PIDs
/// - Reaping all children
/// - Handling different exit statuses
///
/// # Example Output
///
/// ```
/// Parent: Created worker 0 with PID: 12345
/// Parent: Created worker 1 with PID: 12346
/// Worker 0: Starting work...
/// Worker 1: Starting work...
/// Worker 0: Completed!
/// Parent: Worker 12345 exited with code 0
/// ```
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Multiple Worker Processes with Nix ===\n");
    
    const NUM_WORKERS: usize = 5;
    let mut workers: HashMap<Pid, usize> = HashMap::new();
    
    // Create worker processes
    for worker_id in 0..NUM_WORKERS {
        match unsafe { fork()? } {
            ForkResult::Parent { child } => {
                println!("Parent: Created worker {} with PID: {}", worker_id, child);
                workers.insert(child, worker_id);
                
                // Small delay between creating workers (optional)
                sleep(Duration::from_millis(100));
            }
            ForkResult::Child => {
                // Worker process logic
                worker_task(worker_id);
                // Exit with worker_id as exit code for demonstration
                std::process::exit(worker_id as i32);
            }
        }
    }
    
    println!("\nParent: All workers created. Waiting for completion...\n");
    
    // Wait for all workers to complete
    let mut completed = 0;
    while completed < NUM_WORKERS {
        match waitpid(None, None) {
            Ok(status) => {
                completed += 1;
                handle_worker_exit(status, &workers);
            }
            Err(nix::errno::Errno::ECHILD) => {
                // No more children to wait for
                println!("Parent: No more children to wait for");
                break;
            }
            Err(e) => {
                eprintln!("Parent: Error waiting for child: {}", e);
                break;
            }
        }
    }
    
    println!("\nParent: All {} workers have been reaped", completed);
    Ok(())
}

/// Worker task - simulates work being done
///
/// # Arguments
///
/// * `worker_id` - The unique identifier for this worker
fn worker_task(worker_id: usize) {
    println!("Worker {}: Starting work (PID: {})", worker_id, std::process::id());
    
    // Simulate different work durations for each worker
    let work_duration = Duration::from_secs((worker_id + 1) as u64);
    sleep(work_duration);
    
    println!("Worker {}: Completed after {:?}!", worker_id, work_duration);
}

/// Handles the exit status of a worker process
///
/// # Arguments
///
/// * `status` - The wait status returned by waitpid
/// * `workers` - Map of PIDs to worker IDs
fn handle_worker_exit(status: WaitStatus, workers: &HashMap<Pid, usize>) {
    match status {
        WaitStatus::Exited(pid, code) => {
            if let Some(&worker_id) = workers.get(&pid) {
                println!("Parent: Worker {} (PID: {}) exited with code {}", 
                         worker_id, pid, code);
            } else {
                println!("Parent: Unknown child {} exited with code {}", pid, code);
            }
        }
        WaitStatus::Signaled(pid, sig, core_dumped) => {
            if let Some(&worker_id) = workers.get(&pid) {
                println!("Parent: Worker {} (PID: {}) killed by signal {:?} (core dumped: {})", 
                         worker_id, pid, sig, core_dumped);
            } else {
                println!("Parent: Unknown child {} killed by signal {:?}", pid, sig);
            }
        }
        WaitStatus::Stopped(pid, sig) => {
            println!("Parent: Child {} stopped by signal {:?}", pid, sig);
        }
        _ => {
            println!("Parent: Received other wait status: {:?}", status);
        }
    }
}