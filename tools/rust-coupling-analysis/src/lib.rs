//! Rust Coupling Analysis Tool
//!
//! This library provides tools for analyzing coupling in Rust projects based on
//! Vlad Khononov's "Balancing Coupling in Software Design" framework.
//!
//! # Overview
//!
//! The tool measures coupling across three dimensions:
//! - **Integration Strength**: How much knowledge is shared between components
//! - **Distance**: How far apart the components are
//! - **Volatility**: How frequently components change
//!
//! # Example
//!
//! ```rust,no_run
//! use rust_coupling_analysis::analyze_project;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let metrics = analyze_project("./src")?;
//! println!("Analyzed {} files", metrics.file_count);
//! # Ok(())
//! # }
//! ```

pub mod analyzer;
pub mod balance;
pub mod metrics;
pub mod report;
pub mod volatility;

// Re-export main types for convenience
pub use analyzer::CouplingAnalyzer;
pub use balance::{BalancedCouplingScore, CouplingIssue, IssueSeverity};
pub use metrics::{CouplingMeasurement, CouplingMetrics, ProjectMetrics};
pub use report::generate_report;
pub use volatility::{VolatilityAnalyzer, VolatilityLevel};

use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Analyzes a Rust project for coupling metrics
///
/// # Arguments
///
/// * `project_path` - Path to the project root directory
///
/// # Returns
///
/// Returns project metrics including coupling measurements, or an error
///
/// # Errors
///
/// Returns an error if:
/// - The project path doesn't exist
/// - Git history analysis fails
/// - File parsing fails
///
/// # Example
///
/// ```rust,no_run
/// use rust_coupling_analysis::analyze_project;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let metrics = analyze_project("./src")?;
/// println!("Total files analyzed: {}", metrics.file_count);
/// # Ok(())
/// # }
/// ```
pub fn analyze_project(project_path: &str) -> Result<ProjectMetrics, Box<dyn std::error::Error>> {
    let mut project_metrics = ProjectMetrics::default();

    // 1. Analyze structural dependencies
    for entry in WalkDir::new(project_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Process only Rust files
        if path.extension() == Some(OsStr::new("rs")) {
            analyze_rust_file(path, &mut project_metrics)?;
        }
    }

    // 2. Analyze volatility from Git history
    let mut volatility = VolatilityAnalyzer::new();
    volatility.analyze_git_history(6)?;

    // 3. Calculate balance scores
    calculate_balance_scores(&mut project_metrics, &volatility);

    Ok(project_metrics)
}

/// Analyzes an individual Rust file
///
/// # Arguments
///
/// * `path` - File path to analyze
/// * `project_metrics` - Mutable reference to project metrics
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed
fn analyze_rust_file(
    path: &Path,
    project_metrics: &mut ProjectMetrics,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let syntax = syn::parse_file(&content)?;

    let module_name = path.to_string_lossy().replace('/', "::").replace(".rs", "");

    let mut analyzer = CouplingAnalyzer::new(module_name);
    use syn::visit::Visit;
    analyzer.visit_file(&syntax);

    project_metrics.file_count += 1;
    project_metrics.module_count += 1;

    Ok(())
}

/// Calculates balance scores and adds them to project metrics
///
/// # Arguments
///
/// * `metrics` - Mutable reference to project metrics
/// * `volatility` - Reference to volatility analyzer
fn calculate_balance_scores(metrics: &mut ProjectMetrics, _volatility: &VolatilityAnalyzer) {
    for measurement in &metrics.measurements {
        let score = BalancedCouplingScore::new(
            measurement.strength,
            measurement.distance,
            measurement.volatility,
        );

        let issues = score.identify_issues();
        if !issues.is_empty() {
            eprintln!(
                "Issues found in coupling from {} to {}:",
                measurement.from_module, measurement.to_module
            );
            for issue in issues {
                eprintln!("  [{:?}] {}", issue.severity, issue.description);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_project_with_current_project() {
        // Test analyzing the current project
        let result = analyze_project("./src");
        assert!(result.is_ok(), "Project analysis should succeed");
    }
}
