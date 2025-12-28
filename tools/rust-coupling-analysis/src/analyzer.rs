//! AST-based coupling analyzer
//!
//! This module provides tools for analyzing Rust code structure and
//! detecting various forms of coupling through AST traversal.

use crate::metrics::CouplingMetrics;
use syn::visit::Visit;
use syn::{ItemFn, ItemImpl, Signature};

/// AST visitor for analyzing coupling in Rust code
///
/// Traverses the syntax tree and collects coupling metrics including
/// integration strength and connascence patterns.
#[derive(Debug)]
pub struct CouplingAnalyzer {
    /// Collected coupling metrics
    pub metrics: CouplingMetrics,
    /// Current module being analyzed
    pub current_module: String,
}

impl CouplingAnalyzer {
    /// Creates a new analyzer for the given module
    ///
    /// # Arguments
    ///
    /// * `module_name` - Identifier for the module being analyzed
    ///
    /// # Returns
    ///
    /// A new CouplingAnalyzer instance with empty metrics
    ///
    /// # Example
    ///
    /// ```
    /// use rust_coupling_analysis::analyzer::CouplingAnalyzer;
    ///
    /// let analyzer = CouplingAnalyzer::new("my_module".to_string());
    /// assert_eq!(analyzer.current_module, "my_module");
    /// ```
    pub fn new(module_name: String) -> Self {
        Self {
            metrics: CouplingMetrics::default(),
            current_module: module_name,
        }
    }

    /// Analyzes a function signature for coupling patterns
    ///
    /// Detects:
    /// - Connascence of Position (too many parameters)
    /// - Parameter complexity
    ///
    /// # Arguments
    ///
    /// * `sig` - Function signature to analyze
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rust_coupling_analysis::analyzer::CouplingAnalyzer;
    /// use syn::parse_quote;
    ///
    /// let mut analyzer = CouplingAnalyzer::new("test".to_string());
    /// let sig: syn::Signature = parse_quote! {
    ///     fn example(a: i32, b: i32, c: i32, d: i32) -> ()
    /// };
    /// analyzer.analyze_function_signature(&sig);
    /// // This should detect Connascence of Position (4 parameters > 3)
    /// ```
    pub fn analyze_function_signature(&mut self, sig: &Signature) {
        // Check for Connascence of Position - too many parameters
        if sig.inputs.len() > 3 {
            self.metrics.position_coupling.push(format!(
                "Function {} has {} parameters (exceeds recommended limit of 3)",
                sig.ident,
                sig.inputs.len()
            ));
        }
    }

    /// Calculates the distance between two module paths
    ///
    /// Distance is measured as the number of hierarchy steps needed
    /// to navigate from one module to another.
    ///
    /// # Arguments
    ///
    /// * `from_path` - Source module path (e.g., "crate::module::submodule")
    /// * `to_path` - Target module path
    ///
    /// # Returns
    ///
    /// Number of steps in the module hierarchy
    ///
    /// # Example
    ///
    /// ```
    /// use rust_coupling_analysis::analyzer::CouplingAnalyzer;
    ///
    /// let analyzer = CouplingAnalyzer::new("test".to_string());
    /// let distance = analyzer.calculate_distance(
    ///     "crate::module::a",
    ///     "crate::module::b"
    /// );
    /// assert_eq!(distance, 2); // up from 'a', down to 'b'
    /// ```
    pub fn calculate_distance(&self, from_path: &str, to_path: &str) -> usize {
        let from_parts: Vec<&str> = from_path.split("::").collect();
        let to_parts: Vec<&str> = to_path.split("::").collect();

        // Find common ancestor
        let common = from_parts
            .iter()
            .zip(to_parts.iter())
            .take_while(|(a, b)| a == b)
            .count();

        // Distance = steps up from source + steps down to target
        (from_parts.len() - common) + (to_parts.len() - common)
    }

    /// Returns the collected metrics
    ///
    /// # Returns
    ///
    /// Reference to the accumulated CouplingMetrics
    pub fn metrics(&self) -> &CouplingMetrics {
        &self.metrics
    }

    /// Consumes the analyzer and returns the metrics
    ///
    /// # Returns
    ///
    /// The accumulated CouplingMetrics
    pub fn into_metrics(self) -> CouplingMetrics {
        self.metrics
    }
}

impl<'ast> Visit<'ast> for CouplingAnalyzer {
    /// Visits a function definition and analyzes its signature
    ///
    /// Detects coupling patterns in function declarations including
    /// parameter counts and complexity.
    ///
    /// # Arguments
    ///
    /// * `node` - The function item AST node
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        // Analyze function signature for coupling patterns
        self.analyze_function_signature(&node.sig);

        // Continue visiting nested items
        syn::visit::visit_item_fn(self, node);
    }

    /// Visits an impl block and classifies coupling type
    ///
    /// Distinguishes between:
    /// - Contract Coupling: trait implementations (weaker)
    /// - Intrusive Coupling: direct type implementations (stronger)
    ///
    /// # Arguments
    ///
    /// * `node` - The impl block AST node
    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        if node.trait_.is_some() {
            // Trait implementation = Contract Coupling (weaker)
            self.metrics.contract_count += 1;
        } else {
            // Direct implementation on concrete type = Intrusive Coupling (stronger)
            self.metrics.intrusive_count += 1;
        }

        // Continue visiting nested items
        syn::visit::visit_item_impl(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = CouplingAnalyzer::new("test_module".to_string());
        assert_eq!(analyzer.current_module, "test_module");
        assert_eq!(analyzer.metrics.intrusive_count, 0);
    }

    #[test]
    fn test_function_signature_analysis() {
        let mut analyzer = CouplingAnalyzer::new("test".to_string());
        let sig: Signature = parse_quote! {
            fn example(a: i32, b: i32, c: i32, d: i32) -> ()
        };
        analyzer.analyze_function_signature(&sig);
        assert_eq!(analyzer.metrics.position_coupling.len(), 1);
    }

    #[test]
    fn test_calculate_distance_same_level() {
        let analyzer = CouplingAnalyzer::new("test".to_string());
        let distance = analyzer.calculate_distance("crate::module::a", "crate::module::b");
        assert_eq!(distance, 2); // up 1 from 'a', down 1 to 'b'
    }

    #[test]
    fn test_calculate_distance_different_level() {
        let analyzer = CouplingAnalyzer::new("test".to_string());
        let distance = analyzer.calculate_distance("crate::a::b::c", "crate::x::y");
        // From "crate::a::b::c" to "crate::x::y"
        // Common ancestor: "crate" (1 part)
        // Up from c: 3 steps (to a, to crate)
        // Down to y: 2 steps (to x, to y)
        // Total: 3 + 2 = 5
        assert_eq!(distance, 5);
    }

    #[test]
    fn test_visit_trait_impl() {
        let mut analyzer = CouplingAnalyzer::new("test".to_string());
        let impl_block: syn::ItemImpl = parse_quote! {
            impl Display for MyType {
                fn fmt(&self, f: &mut Formatter) -> Result {
                    write!(f, "MyType")
                }
            }
        };
        analyzer.visit_item_impl(&impl_block);
        assert_eq!(analyzer.metrics.contract_count, 1);
        assert_eq!(analyzer.metrics.intrusive_count, 0);
    }

    #[test]
    fn test_visit_inherent_impl() {
        let mut analyzer = CouplingAnalyzer::new("test".to_string());
        let impl_block: syn::ItemImpl = parse_quote! {
            impl MyType {
                fn new() -> Self {
                    MyType {}
                }
            }
        };
        analyzer.visit_item_impl(&impl_block);
        assert_eq!(analyzer.metrics.intrusive_count, 1);
        assert_eq!(analyzer.metrics.contract_count, 0);
    }
}
