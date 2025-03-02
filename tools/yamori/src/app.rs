use crate::test::{TestResult, TestConfig};
use std::time::{SystemTime, UNIX_EPOCH, Instant};

// テスト実行の履歴を保存する構造体
#[derive(Clone)]
pub struct TestHistory {
    pub timestamp: u64,
    pub test_results: Vec<TestResult>,
    pub release_mode: bool,
}

pub struct App {
    pub test_results: Vec<TestResult>,
    pub config: TestConfig,
    pub selected_test: usize,
    pub tab_index: usize,
    pub show_help: bool,
    pub release_mode: bool,
    pub history: Vec<TestHistory>,
    pub selected_history: usize,
    pub viewing_history: bool,
    pub show_popup: bool,
    pub popup_type: PopupType,
    pub result_popup_visible: bool,
    pub result_popup_time: Option<Instant>,
    pub result_popup_message: String,
}

#[derive(PartialEq)]
pub enum PopupType {
    None,
    RunTests,
    RunRelease,
    BuildToggle,
    ResultNotification,
}

impl App {
    pub fn new(test_results: Vec<TestResult>, config: TestConfig) -> Self {
        // 初期実行結果を履歴に追加
        let initial_history = TestHistory {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            test_results: test_results.clone(),
            release_mode: false,
        };
        
        App {
            test_results,
            config,
            selected_test: 0,
            tab_index: 0,
            show_help: false,
            release_mode: false,
            history: vec![initial_history],
            selected_history: 0,
            viewing_history: false,
            show_popup: false,
            popup_type: PopupType::None,
            result_popup_visible: false,
            result_popup_time: None,
            result_popup_message: String::new(),
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
        self.tab_index = (self.tab_index + 1) % 5; // 5 tabs: Results, Stats, Diff, Commands, History
    }

    pub fn previous_tab(&mut self) {
        self.tab_index = if self.tab_index > 0 {
            self.tab_index - 1
        } else {
            4
        };
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn toggle_release_mode(&mut self) {
        self.release_mode = !self.release_mode;
        
        // ビルド設定を更新
        if let Some(build) = &mut self.config.build {
            build.release = self.release_mode;
        }
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
    
    pub fn get_command_details(&self) -> Option<(&str, &[String], Option<&String>, u128, bool, Option<&Vec<String>>)> {
        if self.test_results.is_empty() {
            return None;
        }
        
        let test = &self.test_results[self.selected_test];
        Some((
            &test.command,
            &test.args,
            test.input.as_ref(),
            test.execution_time.as_millis(),
            test.is_release,
            test.build_commands.as_ref()
        ))
    }

    pub fn add_to_history(&mut self) {
        let history_entry = TestHistory {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            test_results: self.test_results.clone(),
            release_mode: self.release_mode,
        };
        
        self.history.push(history_entry);
        self.selected_history = self.history.len() - 1;
    }
    
    pub fn toggle_history_view(&mut self) {
        // 履歴タブに切り替える
        self.tab_index = 4; // History tab
        self.selected_history = self.history.len() - 1;
    }
    
    pub fn next_history(&mut self) {
        if !self.history.is_empty() {
            self.selected_history = (self.selected_history + 1) % self.history.len();
            // 選択した履歴の結果を表示
            if let Some(history) = self.history.get(self.selected_history) {
                // 一時的に履歴の結果を表示
                self.test_results = history.test_results.clone();
                self.release_mode = history.release_mode;
            }
        }
    }
    
    pub fn previous_history(&mut self) {
        if !self.history.is_empty() {
            self.selected_history = if self.selected_history > 0 {
                self.selected_history - 1
            } else {
                self.history.len() - 1
            };
            // 選択した履歴の結果を表示
            if let Some(history) = self.history.get(self.selected_history) {
                // 一時的に履歴の結果を表示
                self.test_results = history.test_results.clone();
                self.release_mode = history.release_mode;
            }
        }
    }
    
    pub fn get_history_stats(&self) -> Vec<(u64, usize, usize, bool)> {
        self.history.iter().map(|h| {
            let total = h.test_results.len();
            let passed = h.test_results.iter().filter(|r| r.success).count();
            (h.timestamp, passed, total, h.release_mode)
        }).collect()
    }
    
    pub fn reset_ui_state(&mut self) {
        // 選択されているテストが範囲外になっていないか確認
        if !self.test_results.is_empty() && self.selected_test >= self.test_results.len() {
            self.selected_test = 0;
        }
        
        // タブを結果表示に戻す
        self.tab_index = 0;
        
        // 履歴表示モードをオフにする
        self.viewing_history = false;
    }

    // ポップアップ表示の切り替え
    pub fn toggle_popup(&mut self, popup_type: PopupType) {
        if self.show_popup && self.popup_type == popup_type {
            self.show_popup = false;
            self.popup_type = PopupType::None;
        } else {
            self.show_popup = true;
            self.popup_type = popup_type;
        }
    }

    // ポップアップを閉じる
    pub fn close_popup(&mut self) {
        self.show_popup = false;
        self.popup_type = PopupType::None;
    }
    
    // 結果ポップアップを表示
    pub fn show_result_popup(&mut self, message: String) {
        self.result_popup_visible = true;
        self.result_popup_time = Some(Instant::now());
        self.result_popup_message = message;
        // ResultNotificationを使用して警告を解消
        self.popup_type = PopupType::ResultNotification;
    }
    
    // 結果ポップアップを更新（時間経過で消える）
    pub fn update_result_popup(&mut self) -> bool {
        if let Some(time) = self.result_popup_time {
            // 3秒経過したらポップアップを消す
            if time.elapsed().as_secs() >= 3 {
                self.result_popup_visible = false;
                self.result_popup_time = None;
                return true; // 状態が変わった
            }
        }
        false // 状態は変わっていない
    }
} 