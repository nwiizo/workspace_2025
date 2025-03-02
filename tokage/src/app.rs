use crate::test::TestResult;

pub struct App {
    pub test_results: Vec<TestResult>,
    pub selected_test: usize,
    pub tab_index: usize,
    pub show_help: bool,
}

impl App {
    pub fn new(test_results: Vec<TestResult>) -> Self {
        App {
            test_results,
            selected_test: 0,
            tab_index: 0,
            show_help: false,
        }
    }

    pub fn next(&mut self) {
        if !self.test_results.is_empty() {
            self.selected_test = (self.selected_test + 1) % self.test_results.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.test_results.is_empty() {
            self.selected_test = if self.selected_test > 0 {
                self.selected_test - 1
            } else {
                self.test_results.len() - 1
            };
        }
    }

    pub fn next_tab(&mut self) {
        self.tab_index = (self.tab_index + 1) % 3; // 3 tabs: Results, Stats, Diff
    }

    pub fn previous_tab(&mut self) {
        self.tab_index = if self.tab_index > 0 {
            self.tab_index - 1
        } else {
            2
        };
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn get_stats(&self) -> (usize, usize, f64) {
        let total = self.test_results.len();
        let passed = self.test_results.iter().filter(|r| r.success).count();
        let pass_rate = if total > 0 {
            (passed as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        (passed, total, pass_rate)
    }
} 