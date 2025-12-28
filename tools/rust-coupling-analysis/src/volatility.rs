//! Volatility analysis from Git history
//!
//! This module analyzes file change frequency using Git history to
//! determine volatility levels of components.

use std::collections::HashMap;
use std::process::Command;
use thiserror::Error;

/// Errors that can occur during volatility analysis
#[derive(Error, Debug)]
pub enum VolatilityError {
    /// Git command execution failed
    #[error("Git command failed: {0}")]
    GitCommandFailed(String),

    /// Failed to parse Git output
    #[error("Failed to parse Git output: {0}")]
    ParseError(String),

    /// IO error occurred
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// UTF-8 conversion error
    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

/// Volatility levels based on change frequency
///
/// Classifies components according to Domain-Driven Design subdomains:
/// - Generic: Low volatility, stable components
/// - Supporting: Medium volatility
/// - Core: High volatility, frequently changing business logic
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VolatilityLevel {
    /// Low volatility - Generic subdomain components
    Low,
    /// Medium volatility - Supporting subdomain components
    Medium,
    /// High volatility - Core subdomain components
    High,
}

impl VolatilityLevel {
    /// Converts volatility level to a normalized score (0.0-1.0)
    ///
    /// # Returns
    ///
    /// - Low: 0.0 (stable)
    /// - Medium: 0.5
    /// - High: 1.0 (volatile)
    ///
    /// # Example
    ///
    /// ```
    /// use rust_coupling_analysis::volatility::VolatilityLevel;
    ///
    /// assert_eq!(VolatilityLevel::Low.to_score(), 0.0);
    /// assert_eq!(VolatilityLevel::Medium.to_score(), 0.5);
    /// assert_eq!(VolatilityLevel::High.to_score(), 1.0);
    /// ```
    pub fn to_score(self) -> f64 {
        match self {
            VolatilityLevel::Low => 0.0,
            VolatilityLevel::Medium => 0.5,
            VolatilityLevel::High => 1.0,
        }
    }
}

/// Analyzer for measuring file change frequency from Git history
///
/// Tracks how often files change over a specified time period to
/// determine which components are volatile (frequently changing) vs
/// stable (rarely changing).
#[derive(Debug, Default, Clone)]
pub struct VolatilityAnalyzer {
    /// Map of file paths to their change counts
    file_changes: HashMap<String, usize>,
}

impl VolatilityAnalyzer {
    /// Creates a new empty volatility analyzer
    ///
    /// # Returns
    ///
    /// A new VolatilityAnalyzer with no recorded changes
    ///
    /// # Example
    ///
    /// ```
    /// use rust_coupling_analysis::volatility::VolatilityAnalyzer;
    ///
    /// let analyzer = VolatilityAnalyzer::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyzes Git history to collect file change frequencies
    ///
    /// Executes `git log` to retrieve file modifications over the
    /// specified time period.
    ///
    /// # Arguments
    ///
    /// * `months` - Number of months of history to analyze
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if Git command fails
    ///
    /// # Errors
    ///
    /// Returns `VolatilityError` if:
    /// - Git command execution fails
    /// - Not in a Git repository
    /// - Output cannot be parsed
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rust_coupling_analysis::volatility::VolatilityAnalyzer;
    ///
    /// let mut analyzer = VolatilityAnalyzer::new();
    /// analyzer.analyze_git_history(6)?; // Analyze last 6 months
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn analyze_git_history(&mut self, months: usize) -> Result<(), VolatilityError> {
        let output = Command::new("git")
            .args([
                "log",
                "--pretty=format:",
                "--name-only",
                &format!("--since={} months ago", months),
            ])
            .output()?;

        // Check if Git command succeeded
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VolatilityError::GitCommandFailed(stderr.to_string()));
        }

        let files = String::from_utf8(output.stdout)?;

        // Count occurrences of each file
        for file in files.lines().filter(|l| !l.is_empty()) {
            *self.file_changes.entry(file.to_string()).or_insert(0) += 1;
        }

        Ok(())
    }

    /// Classifies a file's volatility level
    ///
    /// # Arguments
    ///
    /// * `file` - Path to the file to classify
    ///
    /// # Returns
    ///
    /// The volatility level based on change frequency:
    /// - 0-2 changes: Low
    /// - 3-10 changes: Medium
    /// - 11+ changes: High
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rust_coupling_analysis::volatility::{VolatilityAnalyzer, VolatilityLevel};
    ///
    /// let mut analyzer = VolatilityAnalyzer::new();
    /// analyzer.analyze_git_history(6)?;
    /// let level = analyzer.classify_volatility("src/main.rs");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn classify_volatility(&self, file: &str) -> VolatilityLevel {
        let changes = self.file_changes.get(file).copied().unwrap_or(0);

        match changes {
            0..=2 => VolatilityLevel::Low,
            3..=10 => VolatilityLevel::Medium,
            _ => VolatilityLevel::High,
        }
    }

    /// Gets the change count for a specific file
    ///
    /// # Arguments
    ///
    /// * `file` - Path to the file
    ///
    /// # Returns
    ///
    /// Number of times the file was changed, or 0 if not found
    ///
    /// # Example
    ///
    /// ```
    /// use rust_coupling_analysis::volatility::VolatilityAnalyzer;
    ///
    /// let analyzer = VolatilityAnalyzer::new();
    /// assert_eq!(analyzer.get_change_count("src/main.rs"), 0);
    /// ```
    pub fn get_change_count(&self, file: &str) -> usize {
        self.file_changes.get(file).copied().unwrap_or(0)
    }

    /// Retrieves the most frequently changed files
    ///
    /// # Arguments
    ///
    /// * `n` - Number of top files to return
    ///
    /// # Returns
    ///
    /// Vector of (file_path, change_count) tuples, sorted by change count descending
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rust_coupling_analysis::volatility::VolatilityAnalyzer;
    ///
    /// let mut analyzer = VolatilityAnalyzer::new();
    /// analyzer.analyze_git_history(6)?;
    /// let top_files = analyzer.top_volatile_files(5);
    /// for (file, count) in top_files {
    ///     println!("{}: {} changes", file, count);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn top_volatile_files(&self, n: usize) -> Vec<(&str, usize)> {
        let mut files: Vec<_> = self
            .file_changes
            .iter()
            .map(|(file, &count)| (file.as_str(), count))
            .collect();

        files.sort_by(|a, b| b.1.cmp(&a.1));
        files.truncate(n);
        files
    }

    /// Returns the total number of tracked files
    ///
    /// # Returns
    ///
    /// Number of unique files that have been changed
    ///
    /// # Example
    ///
    /// ```
    /// use rust_coupling_analysis::volatility::VolatilityAnalyzer;
    ///
    /// let analyzer = VolatilityAnalyzer::new();
    /// assert_eq!(analyzer.file_count(), 0);
    /// ```
    pub fn file_count(&self) -> usize {
        self.file_changes.len()
    }

    /// Calculates volatility as a normalized score (0.0-1.0)
    ///
    /// # Arguments
    ///
    /// * `file` - Path to the file
    ///
    /// # Returns
    ///
    /// Volatility score where 0.0 is stable and 1.0 is highly volatile
    ///
    /// # Example
    ///
    /// ```
    /// use rust_coupling_analysis::volatility::VolatilityAnalyzer;
    ///
    /// let analyzer = VolatilityAnalyzer::new();
    /// let score = analyzer.volatility_score("src/main.rs");
    /// assert_eq!(score, 0.0); // No changes recorded
    /// ```
    pub fn volatility_score(&self, file: &str) -> f64 {
        self.classify_volatility(file).to_score()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volatility_level_to_score() {
        assert_eq!(VolatilityLevel::Low.to_score(), 0.0);
        assert_eq!(VolatilityLevel::Medium.to_score(), 0.5);
        assert_eq!(VolatilityLevel::High.to_score(), 1.0);
    }

    #[test]
    fn test_volatility_analyzer_creation() {
        let analyzer = VolatilityAnalyzer::new();
        assert_eq!(analyzer.file_count(), 0);
    }

    #[test]
    fn test_classify_volatility() {
        let mut analyzer = VolatilityAnalyzer::new();
        analyzer.file_changes.insert("low.rs".to_string(), 1);
        analyzer.file_changes.insert("medium.rs".to_string(), 5);
        analyzer.file_changes.insert("high.rs".to_string(), 15);

        assert_eq!(analyzer.classify_volatility("low.rs"), VolatilityLevel::Low);
        assert_eq!(
            analyzer.classify_volatility("medium.rs"),
            VolatilityLevel::Medium
        );
        assert_eq!(
            analyzer.classify_volatility("high.rs"),
            VolatilityLevel::High
        );
    }

    #[test]
    fn test_get_change_count() {
        let mut analyzer = VolatilityAnalyzer::new();
        analyzer.file_changes.insert("test.rs".to_string(), 5);

        assert_eq!(analyzer.get_change_count("test.rs"), 5);
        assert_eq!(analyzer.get_change_count("nonexistent.rs"), 0);
    }

    #[test]
    fn test_top_volatile_files() {
        let mut analyzer = VolatilityAnalyzer::new();
        analyzer.file_changes.insert("a.rs".to_string(), 10);
        analyzer.file_changes.insert("b.rs".to_string(), 5);
        analyzer.file_changes.insert("c.rs".to_string(), 15);

        let top = analyzer.top_volatile_files(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0], ("c.rs", 15));
        assert_eq!(top[1], ("a.rs", 10));
    }

    #[test]
    fn test_volatility_score() {
        let mut analyzer = VolatilityAnalyzer::new();
        analyzer.file_changes.insert("stable.rs".to_string(), 1);
        analyzer.file_changes.insert("volatile.rs".to_string(), 20);

        assert_eq!(analyzer.volatility_score("stable.rs"), 0.0);
        assert_eq!(analyzer.volatility_score("volatile.rs"), 1.0);
    }
}
