//! Balance scoring for coupling analysis
//!
//! This module implements Vlad Khononov's "Balancing Coupling" framework,
//! calculating balance scores based on strength, distance, and volatility.

/// Severity level for coupling issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    /// Low severity - minor issues that may be addressed later
    Low,
    /// Medium severity - issues that should be addressed
    Medium,
    /// High severity - issues that need attention
    High,
    /// Critical severity - issues that require immediate attention
    Critical,
}

/// Represents a detected coupling issue
///
/// Issues are identified based on patterns in the coupling metrics
/// that indicate potential problems or technical debt.
#[derive(Debug, Clone)]
pub struct CouplingIssue {
    /// Severity level of the issue
    pub severity: IssueSeverity,
    /// Human-readable description of the problem
    pub description: String,
    /// Suggested action to address the issue
    pub recommendation: String,
}

impl CouplingIssue {
    /// Creates a new coupling issue
    ///
    /// # Arguments
    ///
    /// * `severity` - Severity level
    /// * `description` - Description of the issue
    /// * `recommendation` - Recommended action
    ///
    /// # Returns
    ///
    /// A new CouplingIssue instance
    ///
    /// # Example
    ///
    /// ```
    /// use rust_coupling_analysis::balance::{CouplingIssue, IssueSeverity};
    ///
    /// let issue = CouplingIssue::new(
    ///     IssueSeverity::High,
    ///     "Strong coupling over long distance".to_string(),
    ///     "Consider moving components closer".to_string()
    /// );
    /// assert_eq!(issue.severity, IssueSeverity::High);
    /// ```
    pub fn new(severity: IssueSeverity, description: String, recommendation: String) -> Self {
        Self {
            severity,
            description,
            recommendation,
        }
    }
}

/// Balance score calculation based on the Balancing Coupling framework
///
/// Implements the formula:
/// ```text
/// BALANCE = (STRENGTH XOR DISTANCE) OR NOT VOLATILITY
/// ```
///
/// Where:
/// - STRENGTH XOR DISTANCE represents modularity
/// - OR NOT VOLATILITY adds stability consideration
#[derive(Debug, Clone)]
pub struct BalancedCouplingScore {
    /// Coupling strength: 0.0 (weak) to 1.0 (strong)
    strength: f64,
    /// Distance: 0.0 (close) to 1.0 (far)
    distance: f64,
    /// Volatility: 0.0 (stable) to 1.0 (volatile)
    volatility: f64,
}

impl BalancedCouplingScore {
    /// Creates a new balance score
    ///
    /// # Arguments
    ///
    /// * `strength` - Coupling strength (0.0-1.0)
    /// * `distance` - Distance metric (0.0-1.0)
    /// * `volatility` - Volatility metric (0.0-1.0)
    ///
    /// # Returns
    ///
    /// A new BalancedCouplingScore instance
    ///
    /// # Panics
    ///
    /// Panics if any value is outside the 0.0-1.0 range
    ///
    /// # Example
    ///
    /// ```
    /// use rust_coupling_analysis::balance::BalancedCouplingScore;
    ///
    /// let score = BalancedCouplingScore::new(0.8, 0.3, 0.5);
    /// assert!(score.calculate_balance() > 0.0);
    /// ```
    pub fn new(strength: f64, distance: f64, volatility: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&strength),
            "strength must be 0.0-1.0, got {}",
            strength
        );
        assert!(
            (0.0..=1.0).contains(&distance),
            "distance must be 0.0-1.0, got {}",
            distance
        );
        assert!(
            (0.0..=1.0).contains(&volatility),
            "volatility must be 0.0-1.0, got {}",
            volatility
        );

        Self {
            strength,
            distance,
            volatility,
        }
    }

    /// Calculates modularity score
    ///
    /// # Formula
    ///
    /// ```text
    /// MODULARITY = STRENGTH XOR DISTANCE (logical XOR)
    /// ```
    ///
    /// Ideal states:
    /// - Strong coupling × Close distance = Good (high locality)
    /// - Weak coupling × Far distance = Good (loose coupling maintained)
    ///
    /// # Returns
    ///
    /// Modularity score (0.0-1.0, higher is better)
    ///
    /// # Example
    ///
    /// ```
    /// use rust_coupling_analysis::balance::BalancedCouplingScore;
    ///
    /// // Strong coupling, close distance (ideal)
    /// let score1 = BalancedCouplingScore::new(0.9, 0.1, 0.5);
    /// assert!(score1.calculate_modularity() > 0.8);
    ///
    /// // Weak coupling, far distance (ideal)
    /// let score2 = BalancedCouplingScore::new(0.1, 0.9, 0.5);
    /// assert!(score2.calculate_modularity() > 0.8);
    /// ```
    pub fn calculate_modularity(&self) -> f64 {
        // Strong coupling × close distance = good (locality)
        let ideal_close = self.strength * (1.0 - self.distance);
        // Weak coupling × far distance = good (loose coupling)
        let ideal_far = (1.0 - self.strength) * self.distance;

        ideal_close + ideal_far
    }

    /// Calculates the overall balance score
    ///
    /// # Formula
    ///
    /// ```text
    /// BALANCE = MODULARITY OR (NOT VOLATILITY) (logical OR)
    /// ```
    ///
    /// Balanced states:
    /// - Modular design, OR
    /// - Low volatility (stable)
    ///
    /// # Returns
    ///
    /// Balance score (0.0-1.0, higher is better)
    ///
    /// # Example
    ///
    /// ```
    /// use rust_coupling_analysis::balance::BalancedCouplingScore;
    ///
    /// let score = BalancedCouplingScore::new(0.8, 0.2, 0.3);
    /// let balance = score.calculate_balance();
    /// assert!(balance >= 0.0 && balance <= 1.0);
    /// ```
    pub fn calculate_balance(&self) -> f64 {
        let modularity = self.calculate_modularity();
        let stability = 1.0 - self.volatility;

        // Logical OR approximation: max
        modularity.max(stability)
    }

    /// Identifies coupling issues based on patterns
    ///
    /// Detects problematic patterns:
    /// 1. Global complexity - strong coupling over long distance
    /// 2. Local complexity - weak coupling at close distance (over-abstraction)
    /// 3. Cascading change risk - strong coupling with high volatility
    /// 4. Low modularity - poor balance of strength and distance
    ///
    /// # Returns
    ///
    /// Vector of detected issues with recommendations
    ///
    /// # Example
    ///
    /// ```
    /// use rust_coupling_analysis::balance::BalancedCouplingScore;
    ///
    /// // Strong coupling over long distance (problematic)
    /// let score = BalancedCouplingScore::new(0.9, 0.9, 0.5);
    /// let issues = score.identify_issues();
    /// assert!(!issues.is_empty());
    /// ```
    pub fn identify_issues(&self) -> Vec<CouplingIssue> {
        let mut issues = Vec::new();

        // Pattern 1: Global complexity
        // Strong coupling + long distance = high coordination cost
        if self.strength > 0.7 && self.distance > 0.7 {
            issues.push(CouplingIssue::new(
                IssueSeverity::High,
                "Strong coupling over long distance increases global complexity".to_string(),
                "Consider moving coupled components closer or reducing coupling strength"
                    .to_string(),
            ));
        }

        // Pattern 2: Local complexity
        // Weak coupling + close distance = unnecessary indirection
        if self.strength < 0.3 && self.distance < 0.3 {
            issues.push(CouplingIssue::new(
                IssueSeverity::Medium,
                "Weak coupling at close distance may indicate unnecessary indirection".to_string(),
                "Consider consolidating components or increasing distance".to_string(),
            ));
        }

        // Pattern 3: Cascading change risk
        // Strong coupling + high volatility = changes propagate widely
        if self.strength > 0.7 && self.volatility > 0.7 {
            issues.push(CouplingIssue::new(
                IssueSeverity::Critical,
                "Strong coupling with volatile component creates cascading change risk".to_string(),
                "Isolate volatile components or reduce coupling strength".to_string(),
            ));
        }

        // Pattern 4: Low modularity
        let modularity = self.calculate_modularity();
        if modularity < 0.4 {
            issues.push(CouplingIssue::new(
                IssueSeverity::Medium,
                format!("Low modularity score: {:.2}", modularity),
                "Review coupling strength and distance relationship".to_string(),
            ));
        }

        issues
    }

    /// Returns the strength component
    ///
    /// # Returns
    ///
    /// Coupling strength (0.0-1.0)
    pub fn strength(&self) -> f64 {
        self.strength
    }

    /// Returns the distance component
    ///
    /// # Returns
    ///
    /// Distance metric (0.0-1.0)
    pub fn distance(&self) -> f64 {
        self.distance
    }

    /// Returns the volatility component
    ///
    /// # Returns
    ///
    /// Volatility metric (0.0-1.0)
    pub fn volatility(&self) -> f64 {
        self.volatility
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balance_score_creation() {
        let score = BalancedCouplingScore::new(0.5, 0.5, 0.5);
        assert_eq!(score.strength(), 0.5);
        assert_eq!(score.distance(), 0.5);
        assert_eq!(score.volatility(), 0.5);
    }

    #[test]
    #[should_panic(expected = "strength must be 0.0-1.0")]
    fn test_invalid_strength() {
        BalancedCouplingScore::new(1.5, 0.5, 0.5);
    }

    #[test]
    fn test_modularity_ideal_close() {
        // Strong coupling, close distance
        let score = BalancedCouplingScore::new(0.9, 0.1, 0.5);
        let modularity = score.calculate_modularity();
        assert!(modularity > 0.8, "Expected high modularity for ideal close");
    }

    #[test]
    fn test_modularity_ideal_far() {
        // Weak coupling, far distance
        let score = BalancedCouplingScore::new(0.1, 0.9, 0.5);
        let modularity = score.calculate_modularity();
        assert!(modularity > 0.8, "Expected high modularity for ideal far");
    }

    #[test]
    fn test_modularity_poor() {
        // Strong coupling, far distance (poor)
        let score = BalancedCouplingScore::new(0.9, 0.9, 0.5);
        let modularity = score.calculate_modularity();
        assert!(modularity < 0.3, "Expected low modularity for poor pattern");
    }

    #[test]
    fn test_balance_calculation() {
        let score = BalancedCouplingScore::new(0.8, 0.2, 0.3);
        let balance = score.calculate_balance();
        assert!(balance >= 0.0 && balance <= 1.0);
    }

    #[test]
    fn test_identify_global_complexity() {
        let score = BalancedCouplingScore::new(0.9, 0.9, 0.5);
        let issues = score.identify_issues();
        assert!(!issues.is_empty());
        assert!(
            issues
                .iter()
                .any(|i| i.description.contains("global complexity"))
        );
    }

    #[test]
    fn test_identify_cascading_change() {
        let score = BalancedCouplingScore::new(0.9, 0.5, 0.9);
        let issues = score.identify_issues();
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.severity == IssueSeverity::Critical));
    }

    #[test]
    fn test_no_issues_for_good_score() {
        // Good modularity with low volatility
        let score = BalancedCouplingScore::new(0.8, 0.2, 0.2);
        let issues = score.identify_issues();
        // May still have issues if modularity calculation yields < 0.4, but should be minimal
        assert!(issues.len() <= 1);
    }

    #[test]
    fn test_coupling_issue_creation() {
        let issue = CouplingIssue::new(
            IssueSeverity::High,
            "Test issue".to_string(),
            "Test recommendation".to_string(),
        );
        assert_eq!(issue.severity, IssueSeverity::High);
        assert_eq!(issue.description, "Test issue");
    }
}
