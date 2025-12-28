//! Rust Coupling Analysis CLI
//!
//! A command-line tool for analyzing coupling in Rust projects based on
//! Vlad Khononov's "Balancing Coupling in Software Design" framework.

use rust_coupling_analysis::{analyze_project, generate_report, report::generate_summary};
use std::env;
use std::fs::File;
use std::io;
use std::path::Path;
use std::process;

/// CLI usage information
const USAGE: &str = r#"
Rust Coupling Analysis Tool

USAGE:
    rust-coupling-analysis [OPTIONS] <PATH>

ARGS:
    <PATH>    Path to the Rust project to analyze (default: ./src)

OPTIONS:
    -h, --help           Print help information
    -o, --output <FILE>  Write report to file instead of stdout
    -s, --summary        Print condensed summary instead of full report
    -v, --version        Print version information

EXAMPLES:
    # Analyze current project
    rust-coupling-analysis ./src

    # Generate report to file
    rust-coupling-analysis --output report.md ./src

    # Print summary
    rust-coupling-analysis --summary ./src
"#;

/// CLI configuration
#[derive(Debug)]
struct Config {
    /// Path to analyze
    path: String,
    /// Optional output file
    output_file: Option<String>,
    /// Whether to print summary instead of full report
    summary_mode: bool,
}

impl Config {
    /// Parses command-line arguments
    ///
    /// # Returns
    ///
    /// Returns parsed configuration or None if help/version was requested
    ///
    /// # Errors
    ///
    /// Exits the program if arguments are invalid
    fn from_args() -> Option<Self> {
        let args: Vec<String> = env::args().collect();

        // Default configuration
        let mut path = String::from("./src");
        let mut output_file = None;
        let mut summary_mode = false;

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "-h" | "--help" => {
                    println!("{}", USAGE);
                    return None;
                }
                "-v" | "--version" => {
                    println!("rust-coupling-analysis {}", env!("CARGO_PKG_VERSION"));
                    return None;
                }
                "-o" | "--output" => {
                    if i + 1 >= args.len() {
                        eprintln!("Error: --output requires a file path");
                        process::exit(1);
                    }
                    output_file = Some(args[i + 1].clone());
                    i += 2;
                }
                "-s" | "--summary" => {
                    summary_mode = true;
                    i += 1;
                }
                arg if !arg.starts_with('-') => {
                    path = arg.to_string();
                    i += 1;
                }
                unknown => {
                    eprintln!("Error: Unknown option '{}'", unknown);
                    eprintln!("{}", USAGE);
                    process::exit(1);
                }
            }
        }

        Some(Config {
            path,
            output_file,
            summary_mode,
        })
    }
}

/// Main entry point
fn main() {
    // Parse command-line arguments
    let config = match Config::from_args() {
        Some(cfg) => cfg,
        None => return, // Help or version was printed
    };

    // Validate path exists
    if !Path::new(&config.path).exists() {
        eprintln!("Error: Path '{}' does not exist", config.path);
        process::exit(1);
    }

    // Run analysis
    eprintln!("Analyzing project at '{}'...", config.path);
    let metrics = match analyze_project(&config.path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error during analysis: {}", e);
            process::exit(1);
        }
    };
    eprintln!(
        "Analysis complete: {} files, {} modules",
        metrics.file_count, metrics.module_count
    );

    // Generate output
    let result = if let Some(ref output_path) = config.output_file {
        // Write to file
        match File::create(output_path) {
            Ok(mut file) => {
                if config.summary_mode {
                    generate_summary(&metrics, &mut file)
                } else {
                    generate_report(&metrics, &mut file)
                }
            }
            Err(e) => {
                eprintln!("Error creating output file: {}", e);
                process::exit(1);
            }
        }
    } else {
        // Write to stdout
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        if config.summary_mode {
            generate_summary(&metrics, &mut handle)
        } else {
            generate_report(&metrics, &mut handle)
        }
    };

    // Handle output errors
    if let Err(e) = result {
        eprintln!("Error writing output: {}", e);
        process::exit(1);
    }

    // Success
    if let Some(output_path) = config.output_file {
        eprintln!("Report written to '{}'", output_path);
    }
}
