/// パフォーマンス検証
///
/// 検証項目 4.1: シグナル処理のレイテンシとスループット測定

use nix::sys::signal::{self, Signal, SigHandler, SigAction, SaFlags, SigSet};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use anyhow::Result;

/// パフォーマンスを検証するバリデーター
pub struct PerformanceValidator;

/// レイテンシ測定用のグローバル変数
static LATENCY_START: AtomicU64 = AtomicU64::new(0);
static LATENCY_END: AtomicU64 = AtomicU64::new(0);
static PERF_COUNTER: AtomicU32 = AtomicU32::new(0);

impl PerformanceValidator {
    /// 検証項目 4.1.1: シグナル送受信のレイテンシ
    pub fn measure_signal_latency() -> Result<Duration, String> {
        // ハンドラーを設定
        extern "C" fn latency_handler(_: i32) {
            LATENCY_END.store(
                Instant::now().elapsed().as_nanos() as u64,
                Ordering::SeqCst
            );
        }
        
        let sig_action = SigAction::new(
            SigHandler::Handler(latency_handler),
            SaFlags::empty(),
            SigSet::empty(),
        );
        
        unsafe {
            signal::sigaction(Signal::SIGUSR1, &sig_action)
                .map_err(|e| format!("Failed to set handler: {}", e))?;
        }
        
        // レイテンシ測定（1000回の平均）
        let mut total_latency = Duration::from_nanos(0);
        let iterations = 1000;
        
        for _ in 0..iterations {
            // 開始時刻を記録
            let start = Instant::now();
            LATENCY_START.store(start.elapsed().as_nanos() as u64, Ordering::SeqCst);
            
            // シグナルを送信
            signal::raise(Signal::SIGUSR1)
                .map_err(|e| format!("Failed to raise signal: {}", e))?;
            
            // ハンドラー実行を待つ
            std::thread::sleep(Duration::from_micros(100));
            
            // レイテンシを計算
            let end_ns = LATENCY_END.load(Ordering::SeqCst);
            let start_ns = LATENCY_START.load(Ordering::SeqCst);
            
            if end_ns > start_ns {
                let latency = Duration::from_nanos(end_ns - start_ns);
                total_latency += latency;
            }
        }
        
        Ok(total_latency / iterations as u32)
    }
    
    /// 検証項目 4.1.2: 高負荷時のシグナル処理性能
    pub fn measure_under_load() -> Result<String, String> {
        let dropped_signals = Arc::new(AtomicU32::new(0));
        let processed_signals = Arc::new(AtomicU32::new(0));
        
        // カウンターをリセット
        PERF_COUNTER.store(0, Ordering::SeqCst);
        
        // シグナルハンドラー設定
        extern "C" fn load_handler(_: i32) {
            PERF_COUNTER.fetch_add(1, Ordering::Relaxed);
            // 重い処理をシミュレート
            std::thread::sleep(Duration::from_micros(10));
        }
        
        let sig_action = SigAction::new(
            SigHandler::Handler(load_handler),
            SaFlags::empty(),
            SigSet::empty(),
        );
        
        unsafe {
            signal::sigaction(Signal::SIGUSR1, &sig_action)
                .map_err(|e| format!("Failed to set handler: {}", e))?;
        }
        
        // 10,000個のシグナルを高速送信
        let signal_count = 10_000;
        let start = Instant::now();
        
        for _ in 0..signal_count {
            if signal::raise(Signal::SIGUSR1).is_err() {
                dropped_signals.fetch_add(1, Ordering::Relaxed);
            } else {
                processed_signals.fetch_add(1, Ordering::Relaxed);
            }
        }
        
        // 処理完了を待つ
        std::thread::sleep(Duration::from_secs(2));
        let elapsed = start.elapsed();
        
        let processed = PERF_COUNTER.load(Ordering::Relaxed);
        let dropped = dropped_signals.load(Ordering::Relaxed);
        let sent = processed_signals.load(Ordering::Relaxed);
        
        let throughput = processed as f64 / elapsed.as_secs_f64();
        
        let report = format!(
            "Performance Report:\n  \
            Sent: {} signals\n  \
            Processed: {} signals\n  \
            Dropped: {} signals\n  \
            Time: {:?}\n  \
            Throughput: {:.2} signals/sec",
            sent, processed, dropped, elapsed, throughput
        );
        
        if dropped > 100 {
            return Err(format!("Too many dropped signals: {}\n{}", dropped, report));
        }
        
        if throughput < 1000.0 {
            return Err(format!("Throughput too low: {:.2} signals/sec\n{}", throughput, report));
        }
        
        Ok(report)
    }
    
    /// 検証項目 4.1.3: シグナルマスク操作のオーバーヘッド測定
    pub fn measure_mask_overhead() -> Result<Duration, String> {
        let iterations = 10_000;
        let start = Instant::now();
        
        for _ in 0..iterations {
            let mut mask = SigSet::empty();
            mask.add(Signal::SIGUSR1);
            
            // ブロック
            signal::pthread_sigmask(
                signal::SigmaskHow::SIG_BLOCK,
                Some(&mask),
                None
            ).map_err(|e| format!("Failed to block: {}", e))?;
            
            // アンブロック
            signal::pthread_sigmask(
                signal::SigmaskHow::SIG_UNBLOCK,
                Some(&mask),
                None
            ).map_err(|e| format!("Failed to unblock: {}", e))?;
        }
        
        let total_time = start.elapsed();
        Ok(total_time / iterations as u32)
    }
    
    /// 検証項目 4.1.4: メモリ使用量の測定
    pub fn measure_memory_usage() -> Result<String, String> {
        use std::collections::HashMap;
        
        // 初期メモリ使用量を取得（簡易的な方法）
        let initial_rss = get_current_rss()?;
        
        // 大量のシグナルハンドラーを設定
        let mut handlers = Vec::new();
        
        for i in 0..100 {
            extern "C" fn dummy_handler(_: i32) {}
            
            let sig_action = SigAction::new(
                SigHandler::Handler(dummy_handler),
                SaFlags::empty(),
                SigSet::empty(),
            );
            
            handlers.push(sig_action);
        }
        
        // メモリ使用量の増加を測定
        let after_handlers_rss = get_current_rss()?;
        
        // シグナル送信のメモリインパクトを測定
        for _ in 0..10_000 {
            let _ = signal::raise(Signal::SIGUSR1);
        }
        
        let final_rss = get_current_rss()?;
        
        let handler_overhead = after_handlers_rss - initial_rss;
        let signal_overhead = final_rss - after_handlers_rss;
        
        Ok(format!(
            "Memory Usage Report:\n  \
            Initial RSS: {} KB\n  \
            After handlers: {} KB (overhead: {} KB)\n  \
            After signals: {} KB (overhead: {} KB)\n  \
            Total overhead: {} KB",
            initial_rss / 1024,
            after_handlers_rss / 1024,
            handler_overhead / 1024,
            final_rss / 1024,
            signal_overhead / 1024,
            (final_rss - initial_rss) / 1024
        ))
    }
    
    /// 検証項目 4.1.5: CPU使用率の測定
    pub fn measure_cpu_usage() -> Result<String, String> {
        use std::thread;
        
        // CPU時間測定の開始
        let start_cpu = get_process_cpu_time()?;
        let start_wall = Instant::now();
        
        // アイドル状態のCPU使用率
        thread::sleep(Duration::from_secs(1));
        
        let idle_cpu = get_process_cpu_time()? - start_cpu;
        
        // シグナル処理中のCPU使用率
        let active_start_cpu = get_process_cpu_time()?;
        
        for _ in 0..10_000 {
            let _ = signal::raise(Signal::SIGUSR1);
            thread::yield_now();
        }
        
        let active_cpu = get_process_cpu_time()? - active_start_cpu;
        let total_wall = start_wall.elapsed();
        
        let idle_percentage = (idle_cpu.as_secs_f64() / 1.0) * 100.0;
        let active_percentage = (active_cpu.as_secs_f64() / total_wall.as_secs_f64()) * 100.0;
        
        Ok(format!(
            "CPU Usage Report:\n  \
            Idle CPU usage: {:.2}%\n  \
            Active CPU usage: {:.2}%\n  \
            Total wall time: {:?}",
            idle_percentage,
            active_percentage,
            total_wall
        ))
    }
}

/// 現在のRSS（Resident Set Size）を取得（Linux用の簡易実装）
fn get_current_rss() -> Result<usize, String> {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        let status = fs::read_to_string("/proc/self/status")
            .map_err(|e| format!("Failed to read /proc/self/status: {}", e))?;
        
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return parts[1].parse::<usize>()
                        .map_err(|e| format!("Failed to parse RSS: {}", e));
                }
            }
        }
        Err("RSS not found in /proc/self/status".to_string())
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        // 他のOSでは固定値を返す（デモ用）
        Ok(1024 * 1024)  // 1MB
    }
}

/// プロセスのCPU時間を取得
fn get_process_cpu_time() -> Result<Duration, String> {
    #[cfg(unix)]
    {
        use libc::{clock_gettime, timespec, CLOCK_PROCESS_CPUTIME_ID};
        
        let mut ts = timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        
        unsafe {
            if clock_gettime(CLOCK_PROCESS_CPUTIME_ID, &mut ts) != 0 {
                return Err("Failed to get CPU time".to_string());
            }
        }
        
        Ok(Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32))
    }
    
    #[cfg(not(unix))]
    {
        Ok(Duration::from_secs(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_measure_mask_overhead() {
        let result = PerformanceValidator::measure_mask_overhead();
        assert!(result.is_ok(), "Mask overhead measurement failed: {:?}", result);
        
        if let Ok(duration) = result {
            println!("Mask operation overhead: {:?}", duration);
            assert!(duration < Duration::from_micros(100), 
                   "Mask operation too slow: {:?}", duration);
        }
    }
    
    #[test]
    fn test_performance_under_load() {
        let result = PerformanceValidator::measure_under_load();
        assert!(result.is_ok(), "Performance test failed: {:?}", result);
        
        if let Ok(report) = result {
            println!("{}", report);
        }
    }
    
    #[test]
    #[cfg(target_os = "linux")]
    fn test_memory_measurement() {
        let result = PerformanceValidator::measure_memory_usage();
        assert!(result.is_ok(), "Memory measurement failed: {:?}", result);
        
        if let Ok(report) = result {
            println!("{}", report);
        }
    }
}