// src/cli.rs
use anyhow::{Context, Result};
use crate::test::{self, TestResult};
use std::path::PathBuf;
use crossterm::style::Stylize;

/// Run tests in CLI mode and print results to stdout
pub fn run_cli(config_path: PathBuf) -> Result<()> {
    // Load and parse the configuration
    let config = test::load_config(&config_path)
        .with_context(|| format!("failed to load config from `{}`", config_path.display()))?;
    
    println!("Running tests from configuration: {}", config_path.display());
    
    // Run all tests
    let test_results = test::run_tests(&config)?;
    
    // Print results in a compact format
    print_compact_results(&test_results);
    
    // Return success only if all tests passed
    if test_results.iter().all(|r| r.success) {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Some tests failed"))
    }
}

/// Print test results in a compact format
fn print_compact_results(results: &[TestResult]) {
    let total = results.len();
    let passed = results.iter().filter(|r| r.success).count();
    let pass_rate = if total > 0 { (passed as f64 / total as f64) * 100.0 } else { 0.0 };
    
    println!("\n=== Test Results ===");
    println!("Passed: {}/{} ({:.1}%)", passed, total, pass_rate);
    println!("====================\n");
    
    // Print a compact summary of each test
    for (i, result) in results.iter().enumerate() {
        let status = if result.success {
            "PASS".green()
        } else {
            "FAIL".red()
        };
        
        println!("[{}] Test #{}: {} ({}ms)", 
            status, 
            i + 1, 
            result.name,
            result.execution_time.as_millis());
        
        // Only show details for failed tests
        if !result.success {
            println!("  Command: {} {}", result.command, result.args.join(" "));
            println!("  Expected vs Actual:");
            
            if let Some(diff) = &result.diff {
                for line in diff {
                    match line.tag {
                        similar::ChangeTag::Delete => println!("  - {}", line.content),
                        similar::ChangeTag::Insert => println!("  + {}", line.content),
                        similar::ChangeTag::Equal => {} // Skip equal lines for brevity
                    }
                }
            }
            println!();
        }
    }
} 