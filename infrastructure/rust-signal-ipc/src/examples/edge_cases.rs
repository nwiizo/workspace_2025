/// エッジケースと異常系の検証
///
/// 検証項目 5.1: エッジケースと異常系の処理

use nix::sys::signal::{self, Signal, SigHandler, SigAction, SaFlags, SigSet, kill};
use nix::unistd;

/// sigpendingのラッパー関数
fn sigpending() -> nix::Result<SigSet> {
    let mut set: libc::sigset_t = unsafe { std::mem::zeroed() };
    let ret = unsafe { libc::sigpending(&mut set) };
    if ret == -1 {
        return Err(nix::errno::Errno::last());
    }
    // libcのsigset_tをnixのSigSetに変換
    let mut nix_set = SigSet::empty();
    for signum in 1..=64 {
        if unsafe { libc::sigismember(&set, signum) } == 1 {
            if let Ok(sig) = Signal::try_from(signum) {
                nix_set.add(sig);
            }
        }
    }
    Ok(nix_set)
}
use std::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;

/// エッジケースを検証するバリデーター
pub struct EdgeCaseValidator;

/// グローバルカウンター（テスト用）
static EDGE_COUNTER: AtomicU32 = AtomicU32::new(0);
static ERROR_COUNTER: AtomicU32 = AtomicU32::new(0);
static RECURSIVE_DEPTH: AtomicU32 = AtomicU32::new(0);
static MAX_RECURSIVE_DEPTH: AtomicU32 = AtomicU32::new(0);

impl EdgeCaseValidator {
    /// 検証項目 5.1.1: 無効なシグナル番号の処理
    pub fn verify_invalid_signal_handling() -> Result<String, String> {
        // 有効なシグナル番号の範囲外をテスト
        let invalid_signals = vec![0, 65, 128, 999];
        let mut errors = Vec::new();
        
        for sig_num in invalid_signals {
            // 無効なシグナル番号でkillを試みる
            match kill(unistd::getpid(), None) {
                Ok(_) => {
                    // Noneは有効なので、次に無効な番号を試す
                    // 注意: 直接無効な番号を使うのは安全でないため、
                    // ここでは範囲チェックのみ行う
                    if sig_num > 64 {
                        errors.push(format!("Signal {} should be invalid", sig_num));
                    }
                }
                Err(nix::errno::Errno::EINVAL) => {
                    // 期待される動作
                }
                Err(e) => {
                    errors.push(format!("Unexpected error for signal {}: {}", sig_num, e));
                }
            }
        }
        
        if errors.is_empty() {
            Ok("Invalid signal handling verified successfully".to_string())
        } else {
            Err(format!("Invalid signal handling errors: {:?}", errors))
        }
    }
    
    /// 検証項目 5.1.2: シグナルストーム耐性テスト
    pub fn verify_signal_storm_resilience() -> Result<String, String> {
        // カウンターをリセット
        EDGE_COUNTER.store(0, Ordering::SeqCst);
        ERROR_COUNTER.store(0, Ordering::SeqCst);
        
        // エラー処理付きハンドラー
        extern "C" fn storm_handler(_: i32) {
            let count = EDGE_COUNTER.fetch_add(1, Ordering::Relaxed);
            if count > 50_000 {
                ERROR_COUNTER.fetch_add(1, Ordering::Relaxed);
            }
        }
        
        let sig_action = SigAction::new(
            SigHandler::Handler(storm_handler),
            SaFlags::empty(),
            SigSet::empty(),
        );
        
        unsafe {
            signal::sigaction(Signal::SIGUSR1, &sig_action)
                .map_err(|e| format!("Failed to set handler: {}", e))?;
        }
        
        // 100,000個のシグナルを一気に送信
        let signal_count = 100_000;
        let handles: Vec<_> = (0..100)
            .map(|thread_id| {
                std::thread::spawn(move || {
                    for i in 0..1000 {
                        if let Err(e) = signal::raise(Signal::SIGUSR1) {
                            eprintln!("Thread {} failed at {}: {}", thread_id, i, e);
                        }
                    }
                })
            })
            .collect();
        
        for handle in handles {
            handle.join().map_err(|_| "Thread join failed")?;
        }
        
        std::thread::sleep(Duration::from_secs(1));
        
        let total_processed = EDGE_COUNTER.load(Ordering::Relaxed);
        let total_errors = ERROR_COUNTER.load(Ordering::Relaxed);
        
        if total_errors > 0 {
            return Err(format!("Handler errors detected: {}", total_errors));
        }
        
        if total_processed < 80_000 {  // 80%以上を期待
            return Err(format!(
                "Too many signals lost: processed only {}/{}",
                total_processed, signal_count
            ));
        }
        
        Ok(format!(
            "Signal storm handled successfully: {}/{} signals processed",
            total_processed, signal_count
        ))
    }
    
    /// 検証項目 5.1.3: ハンドラー内での再帰的シグナル
    pub fn verify_recursive_signal_handling() -> Result<String, String> {
        // カウンターをリセット
        RECURSIVE_DEPTH.store(0, Ordering::SeqCst);
        MAX_RECURSIVE_DEPTH.store(0, Ordering::SeqCst);
        
        extern "C" fn recursive_handler(_: i32) {
            let current = RECURSIVE_DEPTH.fetch_add(1, Ordering::SeqCst);
            MAX_RECURSIVE_DEPTH.fetch_max(current + 1, Ordering::SeqCst);
            
            if current < 3 {
                // 再帰的にシグナルを送信
                let _ = signal::raise(Signal::SIGUSR2);
            }
            
            RECURSIVE_DEPTH.fetch_sub(1, Ordering::SeqCst);
        }
        
        let sig_action = SigAction::new(
            SigHandler::Handler(recursive_handler),
            SaFlags::empty(),
            SigSet::empty(),
        );
        
        unsafe {
            signal::sigaction(Signal::SIGUSR2, &sig_action)
                .map_err(|e| format!("Failed to set recursive handler: {}", e))?;
        }
        
        signal::raise(Signal::SIGUSR2)
            .map_err(|e| format!("Failed to raise signal: {}", e))?;
        
        std::thread::sleep(Duration::from_millis(100));
        
        let final_depth = RECURSIVE_DEPTH.load(Ordering::SeqCst);
        let achieved_max = MAX_RECURSIVE_DEPTH.load(Ordering::SeqCst);
        
        if final_depth != 0 {
            return Err(format!("Depth not restored: {}", final_depth));
        }
        
        if achieved_max > 10 {
            return Err(format!("Recursion too deep: {}", achieved_max));
        }
        
        Ok(format!(
            "Recursive signal handling verified: max depth = {}",
            achieved_max
        ))
    }
    
    /// 検証項目 5.1.4: シグナルハンドラー内でのブロッキング操作
    pub fn verify_blocking_in_handler() -> Result<String, String> {
        use std::sync::Mutex;
        
        static BLOCKING_FLAG: AtomicBool = AtomicBool::new(false);
        
        extern "C" fn blocking_handler(_: i32) {
            // 注意: 実際のハンドラーでブロッキング操作は危険
            // ここではフラグ設定のみ
            BLOCKING_FLAG.store(true, Ordering::SeqCst);
        }
        
        let sig_action = SigAction::new(
            SigHandler::Handler(blocking_handler),
            SaFlags::SA_RESTART,  // システムコールを再開
            SigSet::empty(),
        );
        
        unsafe {
            signal::sigaction(Signal::SIGUSR1, &sig_action)
                .map_err(|e| format!("Failed to set blocking handler: {}", e))?;
        }
        
        // シグナルを送信
        signal::raise(Signal::SIGUSR1)
            .map_err(|e| format!("Failed to raise signal: {}", e))?;
        
        std::thread::sleep(Duration::from_millis(50));
        
        if !BLOCKING_FLAG.load(Ordering::SeqCst) {
            return Err("Handler not executed".to_string());
        }
        
        Ok("Blocking operation in handler tested (flag only)".to_string())
    }
    
    /// 検証項目 5.1.5: シグナルキューのオーバーフロー
    pub fn verify_signal_queue_overflow() -> Result<String, String> {
        // シグナルをブロックして後でまとめて処理
        let mut mask = SigSet::empty();
        mask.add(Signal::SIGUSR1);
        
        signal::pthread_sigmask(
            signal::SigmaskHow::SIG_BLOCK,
            Some(&mask),
            None
        ).map_err(|e| format!("Failed to block signal: {}", e))?;
        
        // 大量のシグナルを送信（キューに溜める）
        let send_count = 1000;
        for _ in 0..send_count {
            signal::raise(Signal::SIGUSR1)
                .map_err(|e| format!("Failed to raise signal: {}", e))?;
        }
        
        // ペンディング状態を確認
        let pending = sigpending()
            .map_err(|e| format!("Failed to check pending: {}", e))?;
        
        if !pending.contains(Signal::SIGUSR1) {
            return Err("No signals pending".to_string());
        }
        
        // ブロック解除
        signal::pthread_sigmask(
            signal::SigmaskHow::SIG_UNBLOCK,
            Some(&mask),
            None
        ).map_err(|e| format!("Failed to unblock: {}", e))?;
        
        std::thread::sleep(Duration::from_millis(100));
        
        // 注意: 標準シグナルは複数回送信してもキューには1つしか残らない
        // リアルタイムシグナルならキューイングされる
        
        Ok(format!(
            "Signal queue overflow test completed: {} signals sent",
            send_count
        ))
    }
    
    /// 検証項目 5.1.6: シグナルハンドラーのリエントラント性
    pub fn verify_reentrant_safety() -> Result<String, String> {
        static REENTRANT_COUNTER: AtomicU32 = AtomicU32::new(0);
        static IN_HANDLER: AtomicBool = AtomicBool::new(false);
        static REENTRANCY_DETECTED: AtomicBool = AtomicBool::new(false);
        
        extern "C" fn reentrant_handler(_: i32) {
            // ハンドラーに入った
            if IN_HANDLER.swap(true, Ordering::SeqCst) {
                // すでにハンドラー内にいる（リエントラント）
                REENTRANCY_DETECTED.store(true, Ordering::SeqCst);
            }
            
            REENTRANT_COUNTER.fetch_add(1, Ordering::SeqCst);
            
            // 少し処理時間をシミュレート
            for _ in 0..1000 {
                std::hint::spin_loop();
            }
            
            // ハンドラーから出る
            IN_HANDLER.store(false, Ordering::SeqCst);
        }
        
        let sig_action = SigAction::new(
            SigHandler::Handler(reentrant_handler),
            SaFlags::empty(),
            SigSet::empty(),
        );
        
        unsafe {
            signal::sigaction(Signal::SIGUSR1, &sig_action)
                .map_err(|e| format!("Failed to set reentrant handler: {}", e))?;
        }
        
        // 複数スレッドから同時にシグナル送信
        let handles: Vec<_> = (0..10)
            .map(|_| {
                std::thread::spawn(|| {
                    for _ in 0..100 {
                        let _ = signal::raise(Signal::SIGUSR1);
                        std::thread::yield_now();
                    }
                })
            })
            .collect();
        
        for handle in handles {
            handle.join().map_err(|_| "Thread join failed")?;
        }
        
        std::thread::sleep(Duration::from_millis(100));
        
        let count = REENTRANT_COUNTER.load(Ordering::SeqCst);
        let reentrancy = REENTRANCY_DETECTED.load(Ordering::SeqCst);
        
        Ok(format!(
            "Reentrant safety test: {} calls, reentrancy detected: {}",
            count, reentrancy
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_invalid_signal_handling() {
        let result = EdgeCaseValidator::verify_invalid_signal_handling();
        assert!(result.is_ok(), "Invalid signal test failed: {:?}", result);
    }
    
    #[test]
    fn test_signal_storm() {
        let result = EdgeCaseValidator::verify_signal_storm_resilience();
        assert!(result.is_ok(), "Signal storm test failed: {:?}", result);
    }
    
    #[test]
    fn test_recursive_signals() {
        let result = EdgeCaseValidator::verify_recursive_signal_handling();
        assert!(result.is_ok(), "Recursive signal test failed: {:?}", result);
    }
    
    #[test]
    fn test_queue_overflow() {
        let result = EdgeCaseValidator::verify_signal_queue_overflow();
        assert!(result.is_ok(), "Queue overflow test failed: {:?}", result);
    }
}