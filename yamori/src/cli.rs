// src/cli.rs
use anyhow::{Context, Result};
use crate::test::{self, TestResult};
use std::{path::PathBuf, io::{self, Write}};
use crossterm::{
    execute,
    terminal::{self, Clear, ClearType},
    cursor,
    style::{Color, SetForegroundColor, ResetColor},
    event::{self, Event, KeyCode},
};

/// Run tests in CLI mode and print results to stdout
pub fn run_cli(config_path: PathBuf) -> Result<()> {
    // Load and parse the configuration
    let config = test::load_config(&config_path)
        .with_context(|| format!("failed to load config from `{}`", config_path.display()))?;
    
    println!("Running tests from configuration: {}", config_path.display());
    
    // Run all tests
    let test_results = test::run_tests(&config)?;
    
    // Print results with interactive scrolling
    display_results_interactive(&test_results)?;
    
    // Return success only if all tests passed
    if test_results.iter().all(|r| r.success) {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Some tests failed"))
    }
}

/// Display test results with interactive scrolling
fn display_results_interactive(results: &[TestResult]) -> Result<()> {
    // Get terminal size
    let (_, height) = terminal::size()?;
    let usable_height = height as usize - 5; // Reserve some lines for header and footer
    
    let total = results.len();
    let passed = results.iter().filter(|r| r.success).count();
    let pass_rate = if total > 0 { (passed as f64 / total as f64) * 100.0 } else { 0.0 };
    
    // Prepare detailed results for each test
    let mut detailed_results = Vec::new();
    for (i, result) in results.iter().enumerate() {
        let mut details = Vec::new();
        details.push(format!("Test #{}: {}", i + 1, result.name));
        details.push(format!("Command: {} {}", result.command, result.args.join(" ")));
        details.push(format!("Status: {}", if result.success { "PASSED" } else { "FAILED" }));
        details.push(format!("Execution time: {:?}", result.execution_time));
        
        if !result.success {
            details.push("\nExpected vs Actual:".to_string());
            if let Some(diff) = &result.diff {
                for line in diff {
                    match line.tag {
                        similar::ChangeTag::Delete => details.push(format!("- {}", line.content)),
                        similar::ChangeTag::Insert => details.push(format!("+ {}", line.content)),
                        similar::ChangeTag::Equal => details.push(format!("  {}", line.content)),
                    }
                }
            }
        }
        
        details.push("\n-------------------\n".to_string());
        detailed_results.push(details);
    }
    
    // Initialize scroll position
    let mut scroll_pos = 0;
    
    // Enable raw mode for keyboard input
    terminal::enable_raw_mode()?;
    
    // Main display loop
    loop {
        // Clear screen
        let mut stdout = io::stdout();
        execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
        
        // Print header
        execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            cursor::MoveTo(0, 0)
        )?;
        writeln!(stdout, "=== Test Results ===").unwrap();
        writeln!(stdout, "Passed: {}/{} ({:.1}%)", passed, total, pass_rate).unwrap();
        writeln!(stdout, "====================").unwrap();
        writeln!(stdout, "Use Up/Down arrows or j/k to scroll, q to quit\n").unwrap();
        execute!(stdout, ResetColor)?;
        
        // Calculate which tests to display based on scroll position
        let mut current_line = 0;
        let mut displayed_lines = 0;
        
        // Skip tests until we reach the scroll position
        for test_details in &detailed_results {
            // If we're before the scroll position, skip this test
            if current_line < scroll_pos {
                current_line += test_details.len();
                continue;
            }
            
            // Display test details
            for line in test_details {
                if displayed_lines < usable_height {
                    // Color code the status line
                    if line.starts_with("Status:") {
                        if line.contains("PASSED") {
                            execute!(stdout, SetForegroundColor(Color::Green))?;
                            writeln!(stdout, "{}", line).unwrap();
                            execute!(stdout, ResetColor)?;
                        } else {
                            execute!(stdout, SetForegroundColor(Color::Red))?;
                            writeln!(stdout, "{}", line).unwrap();
                            execute!(stdout, ResetColor)?;
                        }
                    } 
                    // Color code the diff lines
                    else if line.starts_with("-") {
                        execute!(stdout, SetForegroundColor(Color::Red))?;
                        writeln!(stdout, "{}", line).unwrap();
                        execute!(stdout, ResetColor)?;
                    } else if line.starts_with("+") {
                        execute!(stdout, SetForegroundColor(Color::Green))?;
                        writeln!(stdout, "{}", line).unwrap();
                        execute!(stdout, ResetColor)?;
                    } else {
                        writeln!(stdout, "{}", line).unwrap();
                    }
                    
                    displayed_lines += 1;
                    if displayed_lines >= usable_height {
                        break;
                    }
                }
            }
            
            if displayed_lines >= usable_height {
                break;
            }
        }
        
        // Print footer with scroll indicator
        let total_lines = detailed_results.iter().map(|d| d.len()).sum::<usize>();
        let scroll_percentage = if total_lines > 0 {
            (scroll_pos as f64 / total_lines as f64) * 100.0
        } else {
            0.0
        };
        
        execute!(
            stdout,
            cursor::MoveTo(0, height - 1),
            SetForegroundColor(Color::DarkGrey)
        )?;
        writeln!(stdout, "Scroll: {:.0}% (line {} of {})", 
            scroll_percentage, scroll_pos, total_lines).unwrap();
        execute!(stdout, ResetColor)?;
        
        stdout.flush()?;
        
        // Handle keyboard input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') | KeyCode::Down => {
                        if scroll_pos < total_lines - 1 {
                            scroll_pos += 1;
                        }
                    },
                    KeyCode::Char('k') | KeyCode::Up => {
                        if scroll_pos > 0 {
                            scroll_pos -= 1;
                        }
                    },
                    KeyCode::PageDown => {
                        scroll_pos = std::cmp::min(scroll_pos + usable_height, total_lines - 1);
                    },
                    KeyCode::PageUp => {
                        scroll_pos = scroll_pos.saturating_sub(usable_height);
                    },
                    KeyCode::Home => {
                        scroll_pos = 0;
                    },
                    KeyCode::End => {
                        scroll_pos = total_lines.saturating_sub(usable_height);
                    },
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }
    
    // Disable raw mode
    terminal::disable_raw_mode()?;
    
    // Clear screen and reset cursor
    let mut stdout = io::stdout();
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    
    // Print final summary
    println!("\n=== Test Results ===");
    println!("Passed: {}/{} ({:.1}%)", passed, total, pass_rate);
    println!("====================\n");
    
    Ok(())
}

/// Print test results to stdout (non-interactive version)
fn print_results(results: &[TestResult]) {
    let total = results.len();
    let passed = results.iter().filter(|r| r.success).count();
    let pass_rate = if total > 0 { (passed as f64 / total as f64) * 100.0 } else { 0.0 };
    
    println!("\n=== Test Results ===");
    println!("Passed: {}/{} ({:.1}%)", passed, total, pass_rate);
    println!("====================\n");
    
    for (i, result) in results.iter().enumerate() {
        println!("Test #{}: {}", i + 1, result.name);
        println!("Command: {} {}", result.command, result.args.join(" "));
        println!("Status: {}", if result.success { "PASSED" } else { "FAILED" });
        println!("Execution time: {:?}", result.execution_time);
        
        if !result.success {
            println!("\nExpected vs Actual:");
            if let Some(diff) = &result.diff {
                for line in diff {
                    match line.tag {
                        similar::ChangeTag::Delete => println!("- {}", line.content),
                        similar::ChangeTag::Insert => println!("+ {}", line.content),
                        similar::ChangeTag::Equal => println!("  {}", line.content),
                    }
                }
            }
        }
        
        println!("\n-------------------\n");
    }
} 