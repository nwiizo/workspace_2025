//! Coupling metrics data structures
//!
//! This module defines the core data structures for storing and managing
//! coupling metrics across a project.

/// Coupling metrics for measuring various dimensions of code coupling
///
/// This structure tracks integration strength, distance, and connascence
/// across different dimensions as defined in Balancing Coupling framework.
#[derive(Debug, Default, Clone)]
pub struct CouplingMetrics {
    /// Count of intrusive coupling (strongest)
    pub intrusive_count: usize,
    /// Count of functional coupling
    pub functional_count: usize,
    /// Count of model coupling
    pub model_count: usize,
    /// Count of contract coupling (weakest)
    pub contract_count: usize,

    /// Count of couplings within the same module
    pub same_module: usize,
    /// Count of couplings across modules
    pub cross_module: usize,
    /// Count of couplings across crates
    pub cross_crate: usize,

    /// Names involved in name coupling (Connascence of Name)
    pub name_coupling: Vec<String>,
    /// Types involved in type coupling (Connascence of Type)
    pub type_coupling: Vec<String>,
    /// Functions with position coupling (Connascence of Position)
    pub position_coupling: Vec<String>,
}

impl CouplingMetrics {
    /// Creates a new empty CouplingMetrics instance
    ///
    /// # Returns
    ///
    /// A default-initialized CouplingMetrics with all counters at zero
    ///
    /// # Example
    ///
    /// ```
    /// use rust_coupling_analysis::metrics::CouplingMetrics;
    ///
    /// let metrics = CouplingMetrics::new();
    /// assert_eq!(metrics.intrusive_count, 0);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the total count of coupling instances
    ///
    /// # Returns
    ///
    /// Sum of all strength-based coupling counts
    ///
    /// # Example
    ///
    /// ```
    /// use rust_coupling_analysis::metrics::CouplingMetrics;
    ///
    /// let mut metrics = CouplingMetrics::new();
    /// metrics.intrusive_count = 5;
    /// metrics.contract_count = 10;
    /// assert_eq!(metrics.total_coupling_count(), 15);
    /// ```
    pub fn total_coupling_count(&self) -> usize {
        self.intrusive_count + self.functional_count + self.model_count + self.contract_count
    }

    /// Calculates the average strength of coupling
    ///
    /// Strength values:
    /// - Intrusive: 1.0 (strongest)
    /// - Functional: 0.75
    /// - Model: 0.5
    /// - Contract: 0.25 (weakest)
    ///
    /// # Returns
    ///
    /// Average coupling strength, or 0.0 if no couplings exist
    ///
    /// # Example
    ///
    /// ```
    /// use rust_coupling_analysis::metrics::CouplingMetrics;
    ///
    /// let mut metrics = CouplingMetrics::new();
    /// metrics.intrusive_count = 1;
    /// metrics.contract_count = 1;
    /// assert_eq!(metrics.average_strength(), 0.625);
    /// ```
    pub fn average_strength(&self) -> f64 {
        let total = self.total_coupling_count();
        if total == 0 {
            return 0.0;
        }

        let weighted_sum = (self.intrusive_count as f64 * 1.0)
            + (self.functional_count as f64 * 0.75)
            + (self.model_count as f64 * 0.5)
            + (self.contract_count as f64 * 0.25);

        weighted_sum / total as f64
    }
}

/// Represents a single coupling measurement between two modules
#[derive(Debug, Clone)]
pub struct CouplingMeasurement {
    /// Source module path
    pub from_module: String,
    /// Target module path
    pub to_module: String,
    /// Coupling strength (0.0 = weak, 1.0 = strong)
    pub strength: f64,
    /// Distance between modules (0.0 = close, 1.0 = far)
    pub distance: f64,
    /// Volatility of the modules (0.0 = stable, 1.0 = volatile)
    pub volatility: f64,
}

impl CouplingMeasurement {
    /// Creates a new coupling measurement
    ///
    /// # Arguments
    ///
    /// * `from_module` - Source module identifier
    /// * `to_module` - Target module identifier
    /// * `strength` - Coupling strength (0.0-1.0)
    /// * `distance` - Distance metric (0.0-1.0)
    /// * `volatility` - Change frequency (0.0-1.0)
    ///
    /// # Returns
    ///
    /// A new CouplingMeasurement instance
    ///
    /// # Example
    ///
    /// ```
    /// use rust_coupling_analysis::metrics::CouplingMeasurement;
    ///
    /// let measurement = CouplingMeasurement::new(
    ///     "module_a".to_string(),
    ///     "module_b".to_string(),
    ///     0.8,
    ///     0.3,
    ///     0.5
    /// );
    /// assert_eq!(measurement.from_module, "module_a");
    /// ```
    pub fn new(
        from_module: String,
        to_module: String,
        strength: f64,
        distance: f64,
        volatility: f64,
    ) -> Self {
        Self {
            from_module,
            to_module,
            strength,
            distance,
            volatility,
        }
    }
}

/// Project-wide metrics aggregation
#[derive(Debug, Default, Clone)]
pub struct ProjectMetrics {
    /// All coupling measurements in the project
    pub measurements: Vec<CouplingMeasurement>,
    /// Total number of files analyzed
    pub file_count: usize,
    /// Total number of modules identified
    pub module_count: usize,
}

impl ProjectMetrics {
    /// Creates a new empty ProjectMetrics instance
    ///
    /// # Returns
    ///
    /// A default-initialized ProjectMetrics
    ///
    /// # Example
    ///
    /// ```
    /// use rust_coupling_analysis::metrics::ProjectMetrics;
    ///
    /// let metrics = ProjectMetrics::new();
    /// assert_eq!(metrics.file_count, 0);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a coupling measurement to the project
    ///
    /// # Arguments
    ///
    /// * `measurement` - The coupling measurement to add
    ///
    /// # Example
    ///
    /// ```
    /// use rust_coupling_analysis::metrics::{ProjectMetrics, CouplingMeasurement};
    ///
    /// let mut metrics = ProjectMetrics::new();
    /// let measurement = CouplingMeasurement::new(
    ///     "mod_a".to_string(),
    ///     "mod_b".to_string(),
    ///     0.5, 0.3, 0.2
    /// );
    /// metrics.add_measurement(measurement);
    /// assert_eq!(metrics.measurements.len(), 1);
    /// ```
    pub fn add_measurement(&mut self, measurement: CouplingMeasurement) {
        self.measurements.push(measurement);
    }

    /// Returns the average coupling strength across all measurements
    ///
    /// # Returns
    ///
    /// Average strength, or 0.0 if no measurements exist
    ///
    /// # Example
    ///
    /// ```
    /// use rust_coupling_analysis::metrics::{ProjectMetrics, CouplingMeasurement};
    ///
    /// let mut metrics = ProjectMetrics::new();
    /// metrics.add_measurement(CouplingMeasurement::new(
    ///     "a".to_string(), "b".to_string(), 0.8, 0.3, 0.2
    /// ));
    /// metrics.add_measurement(CouplingMeasurement::new(
    ///     "c".to_string(), "d".to_string(), 0.4, 0.5, 0.1
    /// ));
    /// assert!((metrics.average_strength() - 0.6).abs() < 0.0001);
    /// ```
    pub fn average_strength(&self) -> f64 {
        if self.measurements.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.measurements.iter().map(|m| m.strength).sum();
        sum / self.measurements.len() as f64
    }

    /// Returns the average distance across all measurements
    ///
    /// # Returns
    ///
    /// Average distance, or 0.0 if no measurements exist
    pub fn average_distance(&self) -> f64 {
        if self.measurements.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.measurements.iter().map(|m| m.distance).sum();
        sum / self.measurements.len() as f64
    }

    /// Returns the average volatility across all measurements
    ///
    /// # Returns
    ///
    /// Average volatility, or 0.0 if no measurements exist
    pub fn average_volatility(&self) -> f64 {
        if self.measurements.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.measurements.iter().map(|m| m.volatility).sum();
        sum / self.measurements.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coupling_metrics_default() {
        let metrics = CouplingMetrics::default();
        assert_eq!(metrics.intrusive_count, 0);
        assert_eq!(metrics.total_coupling_count(), 0);
    }

    #[test]
    fn test_average_strength_calculation() {
        let mut metrics = CouplingMetrics::new();
        metrics.intrusive_count = 2;
        metrics.contract_count = 2;
        // (2 * 1.0 + 2 * 0.25) / 4 = 2.5 / 4 = 0.625
        assert_eq!(metrics.average_strength(), 0.625);
    }

    #[test]
    fn test_project_metrics_average() {
        let mut project = ProjectMetrics::new();
        project.add_measurement(CouplingMeasurement::new(
            "a".to_string(),
            "b".to_string(),
            0.8,
            0.4,
            0.2,
        ));
        project.add_measurement(CouplingMeasurement::new(
            "c".to_string(),
            "d".to_string(),
            0.4,
            0.6,
            0.8,
        ));

        // Use approximate comparison for floating-point values
        assert!((project.average_strength() - 0.6).abs() < 0.0001);
        assert!((project.average_distance() - 0.5).abs() < 0.0001);
        assert!((project.average_volatility() - 0.5).abs() < 0.0001);
    }
}
