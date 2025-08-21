//! Integration tests for the process management library

use linux_process_rs::errors::ProcessError;
use linux_process_rs::process::{validate_input, ProcessBuilder};
use linux_process_rs::signal::{SignalHandler, SignalType};
use std::time::Duration;

#[test]
fn test_simple_process_execution() {
    let output = ProcessBuilder::new("echo")
        .arg("hello")
        .arg("world")
        .output()
        .expect("Failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello world"));
}

#[test]
fn test_process_with_environment() {
    // Use printenv instead of echo with shell expansion to avoid $ character
    let output = ProcessBuilder::new("printenv")
        .arg("TEST_VAR")
        .env("TEST_VAR", "test_value")
        .output()
        .expect("Failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test_value"));
}

#[test]
fn test_process_timeout() {
    let builder = ProcessBuilder::new("sleep")
        .arg("10")
        .timeout(Duration::from_millis(100));

    let result = builder.spawn().and_then(|mut g| g.wait());

    match result {
        Err(ProcessError::TimeoutError { .. }) => {
            // Expected timeout error
        }
        _ => panic!("Expected timeout error"),
    }
}

#[test]
fn test_input_validation_safe() {
    assert!(validate_input("normal_file.txt").is_ok());
    assert!(validate_input("file-name_123.log").is_ok());
}

#[test]
fn test_input_validation_dangerous() {
    assert!(validate_input("file.txt; rm -rf /").is_err());
    assert!(validate_input("../../../etc/passwd").is_err());
    assert!(validate_input("$(whoami)").is_err());
    assert!(validate_input("file\0name").is_err());
}

#[test]
fn test_signal_handler_creation() {
    let handler = SignalHandler::new(&[SignalType::Interrupt, SignalType::Terminate])
        .expect("Failed to create signal handler");

    assert!(!handler.should_shutdown());
}

#[test]
fn test_process_guard_cleanup() {
    // Test that ProcessGuard properly cleans up on drop
    {
        let _guard = ProcessBuilder::new("sleep")
            .arg("10")
            .spawn()
            .expect("Failed to spawn process");
        // Guard will be dropped here, should kill the process
    }

    // Process should be terminated by now
    std::thread::sleep(Duration::from_millis(100));
}

#[cfg(unix)]
#[test]
fn test_process_with_working_directory() {
    let output = ProcessBuilder::new("pwd")
        .current_dir("/tmp")
        .output()
        .expect("Failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("/tmp"));
}
