//! Report generation for coupling analysis
//!
//! This module provides functionality to generate human-readable reports
//! from coupling analysis results in Markdown format.

use crate::balance::BalancedCouplingScore;
use crate::metrics::ProjectMetrics;
use std::io::{self, Write};

/// Generates a comprehensive Markdown report
///
/// Creates a formatted report including:
/// - Project summary statistics
/// - Integration strength distribution
/// - Detected coupling issues
/// - High volatility analysis
///
/// # Arguments
///
/// * `metrics` - Project metrics to report on
/// * `writer` - Output destination (stdout, file, etc.)
///
/// # Returns
///
/// Returns `Ok(())` on success, or an IO error
///
/// # Errors
///
/// Returns an error if writing to the output fails
///
/// # Example
///
/// ```rust,no_run
/// use rust_coupling_analysis::{analyze_project, generate_report};
/// use std::fs::File;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let metrics = analyze_project("./src")?;
///
/// // Write to stdout
/// generate_report(&metrics, &mut std::io::stdout())?;
///
/// // Write to file
/// let mut file = File::create("coupling-report.md")?;
/// generate_report(&metrics, &mut file)?;
/// # Ok(())
/// # }
/// ```
pub fn generate_report<W: Write>(metrics: &ProjectMetrics, writer: &mut W) -> io::Result<()> {
    writeln!(writer, "# Coupling Analysis Report\n")?;

    // Summary section
    write_summary(metrics, writer)?;

    // Integration strength distribution
    write_strength_distribution(metrics, writer)?;

    // Detected issues
    write_issues(metrics, writer)?;

    // Volatility analysis
    write_volatility_analysis(metrics, writer)?;

    Ok(())
}

/// Writes the summary section
///
/// # Arguments
///
/// * `metrics` - Project metrics
/// * `writer` - Output destination
///
/// # Returns
///
/// Returns `Ok(())` on success, or an IO error
fn write_summary<W: Write>(metrics: &ProjectMetrics, writer: &mut W) -> io::Result<()> {
    writeln!(writer, "## Summary\n")?;
    writeln!(writer, "- **Total Files**: {}", metrics.file_count)?;
    writeln!(writer, "- **Total Modules**: {}", metrics.module_count)?;
    writeln!(
        writer,
        "- **Total Couplings**: {}\n",
        metrics.measurements.len()
    )?;

    if !metrics.measurements.is_empty() {
        writeln!(
            writer,
            "- **Average Strength**: {:.2}",
            metrics.average_strength()
        )?;
        writeln!(
            writer,
            "- **Average Distance**: {:.2}",
            metrics.average_distance()
        )?;
        writeln!(
            writer,
            "- **Average Volatility**: {:.2}\n",
            metrics.average_volatility()
        )?;
    }

    Ok(())
}

/// Writes the integration strength distribution section
///
/// Classifies couplings by strength level:
/// - Contract: 0.0-0.25 (weakest)
/// - Model: 0.25-0.50
/// - Functional: 0.50-0.75
/// - Intrusive: 0.75-1.0 (strongest)
///
/// # Arguments
///
/// * `metrics` - Project metrics
/// * `writer` - Output destination
///
/// # Returns
///
/// Returns `Ok(())` on success, or an IO error
fn write_strength_distribution<W: Write>(
    metrics: &ProjectMetrics,
    writer: &mut W,
) -> io::Result<()> {
    writeln!(writer, "## Integration Strength Distribution\n")?;

    let total = metrics.measurements.len() as f64;
    if total == 0.0 {
        writeln!(writer, "No coupling measurements available.\n")?;
        return Ok(());
    }

    let contract = metrics
        .measurements
        .iter()
        .filter(|m| m.strength <= 0.25)
        .count();
    let model = metrics
        .measurements
        .iter()
        .filter(|m| m.strength > 0.25 && m.strength <= 0.50)
        .count();
    let functional = metrics
        .measurements
        .iter()
        .filter(|m| m.strength > 0.50 && m.strength <= 0.75)
        .count();
    let intrusive = metrics
        .measurements
        .iter()
        .filter(|m| m.strength > 0.75)
        .count();

    writeln!(
        writer,
        "- **Contract Coupling** (weakest): {} ({:.1}%)",
        contract,
        (contract as f64 / total) * 100.0
    )?;
    writeln!(
        writer,
        "- **Model Coupling**: {} ({:.1}%)",
        model,
        (model as f64 / total) * 100.0
    )?;
    writeln!(
        writer,
        "- **Functional Coupling**: {} ({:.1}%)",
        functional,
        (functional as f64 / total) * 100.0
    )?;
    writeln!(
        writer,
        "- **Intrusive Coupling** (strongest): {} ({:.1}%)\n",
        intrusive,
        (intrusive as f64 / total) * 100.0
    )?;

    Ok(())
}

/// Writes the detected issues section
///
/// Analyzes each coupling measurement and reports any identified problems
/// with severity levels and recommendations.
///
/// # Arguments
///
/// * `metrics` - Project metrics
/// * `writer` - Output destination
///
/// # Returns
///
/// Returns `Ok(())` on success, or an IO error
fn write_issues<W: Write>(metrics: &ProjectMetrics, writer: &mut W) -> io::Result<()> {
    writeln!(writer, "## Detected Issues\n")?;

    let mut has_issues = false;

    for measurement in &metrics.measurements {
        let score = BalancedCouplingScore::new(
            measurement.strength,
            measurement.distance,
            measurement.volatility,
        );

        let issues = score.identify_issues();
        if !issues.is_empty() {
            has_issues = true;
            writeln!(
                writer,
                "### {} → {}\n",
                measurement.from_module, measurement.to_module
            )?;

            writeln!(
                writer,
                "**Metrics**: Strength={:.2}, Distance={:.2}, Volatility={:.2}\n",
                measurement.strength, measurement.distance, measurement.volatility
            )?;

            for issue in issues {
                writeln!(writer, "**{:?}**: {}", issue.severity, issue.description)?;
                writeln!(writer, "- *Recommendation*: {}\n", issue.recommendation)?;
            }
        }
    }

    if !has_issues {
        writeln!(writer, "No significant coupling issues detected.\n")?;
    }

    Ok(())
}

/// Writes the volatility analysis section
///
/// Reports on high-volatility modules that change frequently and may
/// represent core business logic requiring special attention.
///
/// # Arguments
///
/// * `metrics` - Project metrics
/// * `writer` - Output destination
///
/// # Returns
///
/// Returns `Ok(())` on success, or an IO error
fn write_volatility_analysis<W: Write>(metrics: &ProjectMetrics, writer: &mut W) -> io::Result<()> {
    writeln!(writer, "## High Volatility Analysis\n")?;

    // Extract high volatility files (> 0.7)
    let high_volatility: Vec<_> = metrics
        .measurements
        .iter()
        .filter(|m| m.volatility > 0.7)
        .collect();

    if high_volatility.is_empty() {
        writeln!(writer, "No high volatility modules detected.\n")?;
    } else {
        writeln!(writer, "Modules with high change frequency:\n")?;
        for measurement in high_volatility {
            writeln!(
                writer,
                "- `{}` (volatility: {:.2})",
                measurement.from_module, measurement.volatility
            )?;
        }
        writeln!(writer)?;
    }

    Ok(())
}

/// Generates a condensed summary report suitable for CI/CD output
///
/// # Arguments
///
/// * `metrics` - Project metrics to summarize
/// * `writer` - Output destination
///
/// # Returns
///
/// Returns `Ok(())` on success, or an IO error
///
/// # Example
///
/// ```rust,no_run
/// use rust_coupling_analysis::{analyze_project, report::generate_summary};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let metrics = analyze_project("./src")?;
/// generate_summary(&metrics, &mut std::io::stdout())?;
/// # Ok(())
/// # }
/// ```
pub fn generate_summary<W: Write>(metrics: &ProjectMetrics, writer: &mut W) -> io::Result<()> {
    writeln!(writer, "Coupling Analysis Summary:")?;
    writeln!(writer, "  Files: {}", metrics.file_count)?;
    writeln!(writer, "  Modules: {}", metrics.module_count)?;
    writeln!(writer, "  Couplings: {}", metrics.measurements.len())?;

    if !metrics.measurements.is_empty() {
        writeln!(writer, "  Avg Strength: {:.2}", metrics.average_strength())?;

        // Count issues
        let mut issue_count = 0;
        let mut critical_count = 0;

        for measurement in &metrics.measurements {
            let score = BalancedCouplingScore::new(
                measurement.strength,
                measurement.distance,
                measurement.volatility,
            );
            let issues = score.identify_issues();
            issue_count += issues.len();
            critical_count += issues
                .iter()
                .filter(|i| matches!(i.severity, crate::balance::IssueSeverity::Critical))
                .count();
        }

        writeln!(writer, "  Issues: {}", issue_count)?;
        writeln!(writer, "  Critical: {}", critical_count)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::{CouplingMeasurement, ProjectMetrics};

    #[test]
    fn test_generate_empty_report() {
        let metrics = ProjectMetrics::default();
        let mut buffer = Vec::new();
        let result = generate_report(&metrics, &mut buffer);
        assert!(result.is_ok());
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("# Coupling Analysis Report"));
    }

    #[test]
    fn test_generate_summary() {
        let mut metrics = ProjectMetrics::default();
        metrics.file_count = 10;
        metrics.module_count = 5;

        let mut buffer = Vec::new();
        let result = generate_summary(&metrics, &mut buffer);
        assert!(result.is_ok());
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Files: 10"));
    }

    #[test]
    fn test_write_strength_distribution() {
        let mut metrics = ProjectMetrics::default();
        metrics.add_measurement(CouplingMeasurement::new(
            "a".to_string(),
            "b".to_string(),
            0.2,
            0.3,
            0.1,
        ));
        metrics.add_measurement(CouplingMeasurement::new(
            "c".to_string(),
            "d".to_string(),
            0.8,
            0.4,
            0.2,
        ));

        let mut buffer = Vec::new();
        let result = write_strength_distribution(&metrics, &mut buffer);
        assert!(result.is_ok());
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Contract Coupling"));
        assert!(output.contains("Intrusive Coupling"));
    }

    #[test]
    fn test_write_issues() {
        let mut metrics = ProjectMetrics::default();
        // Add a problematic coupling: strong + far distance
        metrics.add_measurement(CouplingMeasurement::new(
            "module_a".to_string(),
            "module_b".to_string(),
            0.9,
            0.9,
            0.5,
        ));

        let mut buffer = Vec::new();
        let result = write_issues(&metrics, &mut buffer);
        assert!(result.is_ok());
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("module_a"));
    }
}
