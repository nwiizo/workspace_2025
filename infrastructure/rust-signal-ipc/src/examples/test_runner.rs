/// 統合テストランナー
///
/// すべての検証項目を実行し、結果をレポート

use std::time::{Duration, Instant};
use anyhow::Result;

use super::basic_signal::{SignalValidator, PracticalSignalHandler};
use super::signal_mask::SignalMaskValidator;
use super::thread_safety::ThreadSafetyValidator;
use super::performance::PerformanceValidator;
use super::edge_cases::EdgeCaseValidator;

/// テスト結果を保持する構造体
#[derive(Debug)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub duration: Duration,
    pub message: Option<String>,
}

impl TestResult {
    fn new(name: String, passed: bool, duration: Duration, message: Option<String>) -> Self {
        Self {
            name,
            passed,
            duration,
            message,
        }
    }
}

/// 統合テストランナー
pub struct SignalTestRunner {
    results: Vec<TestResult>,
    verbose: bool,
}

impl SignalTestRunner {
    pub fn new(verbose: bool) -> Self {
        Self {
            results: Vec::new(),
            verbose,
        }
    }
    
    /// すべての検証項目を実行
    pub fn run_all_tests(&mut self) {
        println!("\n🚀 シグナル処理検証システム開始\n");
        println!("{}", "=".repeat(60));
        
        // 基本機能テスト
        println!("\n📋 基本シグナル処理テスト");
        println!("{}", "-".repeat(40));
        
        self.run_test("Basic Signal Delivery", || {
            PracticalSignalHandler::test_signal_delivery()
        });
        
        self.run_test("Signal Validator Creation", || {
            let validator = SignalValidator::new();
            if validator.received_signals.load(std::sync::atomic::Ordering::SeqCst) == 0 {
                Ok("Validator created successfully".to_string())
            } else {
                Err("Validator initialization failed".to_string())
            }
        });
        
        // シグナルマスクテスト
        println!("\n🎭 シグナルマスクテスト");
        println!("{}", "-".repeat(40));
        
        self.run_test("Signal Blocking", || {
            SignalMaskValidator::verify_signal_blocking()
        });
        
        self.run_test("Multiple Signal Masking", || {
            SignalMaskValidator::verify_multiple_signal_masking()
        });
        
        self.run_test("Signal Set Operations", || {
            SignalMaskValidator::verify_sigset_operations()
        });
        
        self.run_test("Mask Save/Restore", || {
            SignalMaskValidator::verify_mask_save_restore()
        });
        
        // 並行性テスト
        println!("\n🔄 並行性とスレッドセーフティテスト");
        println!("{}", "-".repeat(40));
        
        self.run_test("Concurrent Signal Sending", || {
            ThreadSafetyValidator::verify_concurrent_signal_sending()
        });
        
        self.run_test("Thread Signal Delivery", || {
            ThreadSafetyValidator::verify_thread_signal_delivery()
        });
        
        self.run_test("Race Condition Safety", || {
            ThreadSafetyValidator::verify_race_condition_safety()
        });
        
        self.run_test("Deadlock Avoidance", || {
            ThreadSafetyValidator::verify_deadlock_avoidance()
        });
        
        // パフォーマンステスト
        println!("\n⚡ パフォーマンステスト");
        println!("{}", "-".repeat(40));
        
        self.run_test("Signal Latency", || {
            match PerformanceValidator::measure_signal_latency() {
                Ok(latency) => {
                    if latency > Duration::from_micros(1000) {
                        Err(format!("Latency too high: {:?}", latency))
                    } else {
                        Ok(format!("Average latency: {:?}", latency))
                    }
                }
                Err(e) => Err(e),
            }
        });
        
        self.run_test("Performance Under Load", || {
            PerformanceValidator::measure_under_load()
        });
        
        self.run_test("Mask Operation Overhead", || {
            match PerformanceValidator::measure_mask_overhead() {
                Ok(overhead) => Ok(format!("Overhead: {:?}", overhead)),
                Err(e) => Err(e),
            }
        });
        
        #[cfg(target_os = "linux")]
        self.run_test("Memory Usage", || {
            PerformanceValidator::measure_memory_usage()
        });
        
        // エッジケーステスト
        println!("\n🔥 エッジケースと異常系テスト");
        println!("{}", "-".repeat(40));
        
        self.run_test("Invalid Signal Handling", || {
            EdgeCaseValidator::verify_invalid_signal_handling()
        });
        
        self.run_test("Signal Storm Resilience", || {
            EdgeCaseValidator::verify_signal_storm_resilience()
        });
        
        self.run_test("Recursive Signal Handling", || {
            EdgeCaseValidator::verify_recursive_signal_handling()
        });
        
        self.run_test("Signal Queue Overflow", || {
            EdgeCaseValidator::verify_signal_queue_overflow()
        });
        
        self.run_test("Reentrant Safety", || {
            EdgeCaseValidator::verify_reentrant_safety()
        });
    }
    
    /// 個別のテストを実行
    fn run_test<F>(&mut self, name: &str, test_fn: F)
    where
        F: FnOnce() -> Result<String, String>,
    {
        let start = Instant::now();
        let result = test_fn();
        let duration = start.elapsed();
        
        let passed = result.is_ok();
        let message = match result {
            Ok(msg) => Some(msg),
            Err(msg) => Some(msg),
        };
        
        self.results.push(TestResult::new(
            name.to_string(),
            passed,
            duration,
            message.clone(),
        ));
        
        // リアルタイム出力
        if passed {
            print!("✅ {} ", name);
            if self.verbose {
                print!("({:?})", duration);
                if let Some(msg) = message {
                    print!(" - {}", msg);
                }
            }
            println!();
        } else {
            print!("❌ {} ", name);
            if self.verbose {
                print!("({:?})", duration);
            }
            if let Some(msg) = message {
                print!(" - {}", msg);
            }
            println!();
        }
    }
    
    /// テスト結果のサマリーを生成
    pub fn generate_report(&self) -> String {
        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        
        let total_time: Duration = self.results.iter()
            .map(|r| r.duration)
            .sum();
        
        let mut report = String::new();
        report.push_str(&format!("\n{}\n", "=".repeat(60)));
        report.push_str("📊 テスト結果サマリー\n");
        report.push_str(&format!("{}\n", "=".repeat(60)));
        
        report.push_str(&format!("総テスト数: {}\n", total));
        report.push_str(&format!("✅ 成功: {} ({:.1}%)\n", 
            passed, (passed as f64 / total as f64) * 100.0));
        report.push_str(&format!("❌ 失敗: {} ({:.1}%)\n", 
            failed, (failed as f64 / total as f64) * 100.0));
        report.push_str(&format!("⏱️  総実行時間: {:?}\n", total_time));
        
        if total > 0 {
            report.push_str(&format!("⏱️  平均実行時間: {:?}\n", 
                total_time / total as u32));
        }
        
        // 失敗したテストの詳細
        if failed > 0 {
            report.push_str(&format!("\n{}\n", "-".repeat(40)));
            report.push_str("❌ 失敗したテスト:\n");
            for result in &self.results {
                if !result.passed {
                    report.push_str(&format!("  - {}", result.name));
                    if let Some(msg) = &result.message {
                        report.push_str(&format!(": {}", msg));
                    }
                    report.push_str("\n");
                }
            }
        }
        
        // パフォーマンス統計
        report.push_str(&format!("\n{}\n", "-".repeat(40)));
        report.push_str("⚡ パフォーマンス統計:\n");
        
        let mut test_times: Vec<(&str, Duration)> = self.results.iter()
            .map(|r| (r.name.as_str(), r.duration))
            .collect();
        test_times.sort_by_key(|&(_, d)| d);
        
        report.push_str("  最速テスト:\n");
        for (name, duration) in test_times.iter().take(3) {
            report.push_str(&format!("    - {} ({:?})\n", name, duration));
        }
        
        if test_times.len() > 3 {
            report.push_str("  最遅テスト:\n");
            for (name, duration) in test_times.iter().rev().take(3) {
                report.push_str(&format!("    - {} ({:?})\n", name, duration));
            }
        }
        
        report.push_str(&format!("{}\n", "=".repeat(60)));
        
        report
    }
    
    /// 結果をJSONフォーマットで出力
    pub fn results_as_json(&self) -> String {
        let results: Vec<_> = self.results.iter()
            .map(|r| {
                format!(
                    r#"{{"name":"{}","passed":{},"duration_ms":{},"message":{}}}"#,
                    r.name,
                    r.passed,
                    r.duration.as_millis(),
                    r.message.as_ref()
                        .map(|m| format!(r#""{}""#, m.replace('"', r#"\""#)))
                        .unwrap_or_else(|| "null".to_string())
                )
            })
            .collect();
        
        format!(r#"{{"tests":[{}]}}"#, results.join(","))
    }
}

/// 簡易的なベンチマーク機能
pub fn run_benchmark() {
    println!("\n🎯 ベンチマーク実行\n");
    println!("{}", "=".repeat(60));
    
    // シグナル送受信のベンチマーク
    println!("\n📊 シグナル送受信ベンチマーク");
    let iterations = 10_000;
    let start = Instant::now();
    
    for _ in 0..iterations {
        let _ = nix::sys::signal::raise(nix::sys::signal::Signal::SIGUSR1);
    }
    
    let elapsed = start.elapsed();
    let per_operation = elapsed / iterations;
    
    println!("  総時間: {:?}", elapsed);
    println!("  操作数: {}", iterations);
    println!("  平均時間/操作: {:?}", per_operation);
    println!("  スループット: {:.0} ops/sec", 
        iterations as f64 / elapsed.as_secs_f64());
}