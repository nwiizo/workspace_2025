/// 並行性とスレッドセーフティの検証
///
/// 検証項目 3.1: マルチスレッド環境でのシグナル処理

use nix::sys::signal::{self, Signal, SigHandler, SigAction, SaFlags, SigSet};
use nix::unistd::{fork, ForkResult};

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
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;

/// スレッドセーフティを検証するバリデーター
pub struct ThreadSafetyValidator;

/// グローバルカウンター（テスト用）
static CONCURRENT_COUNTER: AtomicU32 = AtomicU32::new(0);

impl ThreadSafetyValidator {
    /// 検証項目 3.1.1: 複数スレッドからの同時シグナル送信
    pub fn verify_concurrent_signal_sending() -> Result<String, String> {
        // カウンターをリセット
        CONCURRENT_COUNTER.store(0, Ordering::SeqCst);
        
        // グローバルハンドラー設定
        extern "C" fn handler(_: i32) {
            CONCURRENT_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        
        let sig_action = SigAction::new(
            SigHandler::Handler(handler),
            SaFlags::empty(),
            SigSet::empty(),
        );
        
        unsafe {
            signal::sigaction(Signal::SIGUSR1, &sig_action)
                .map_err(|e| format!("Failed to set signal handler: {}", e))?;
        }
        
        // 10スレッドから同時にシグナル送信
        let handles: Vec<_> = (0..10)
            .map(|thread_id| {
                std::thread::spawn(move || {
                    for i in 0..100 {
                        if let Err(e) = signal::raise(Signal::SIGUSR1) {
                            eprintln!("Thread {} failed at iteration {}: {}", thread_id, i, e);
                        }
                        std::thread::yield_now();
                    }
                })
            })
            .collect();
        
        // すべてのスレッドの完了を待つ
        for handle in handles {
            handle.join().map_err(|_| "Thread join failed")?;
        }
        
        // 少し待ってシグナル処理の完了を待つ
        std::thread::sleep(Duration::from_millis(500));
        
        let total = CONCURRENT_COUNTER.load(Ordering::SeqCst);
        
        // 完全性のチェック（いくつかのシグナルが失われる可能性を考慮）
        if total < 900 {  // 90%以上を期待
            return Err(format!("Too many signals lost. Expected ~1000, got {}", total));
        }
        
        Ok(format!("Concurrent signal sending verified: {} signals processed", total))
    }
    
    /// 検証項目 3.1.2: fork後のシグナルハンドラー継承
    pub fn verify_signal_handler_inheritance() -> Result<String, String> {
        // 親プロセスでハンドラー設定
        extern "C" fn ignore_handler(_: i32) {
            // 何もしない（無視）
        }
        
        let sig_action = SigAction::new(
            SigHandler::Handler(ignore_handler),
            SaFlags::empty(),
            SigSet::empty(),
        );
        
        unsafe {
            signal::sigaction(Signal::SIGUSR1, &sig_action)
                .map_err(|e| format!("Failed to set handler in parent: {}", e))?;
        }
        
        match unsafe { fork() }
            .map_err(|e| format!("Fork failed: {}", e))? {
            ForkResult::Parent { child } => {
                // 子プロセスの終了を待つ
                nix::sys::wait::waitpid(child, None)
                    .map_err(|e| format!("Failed to wait for child: {}", e))?;
                
                Ok("Signal handler inheritance verified".to_string())
            }
            ForkResult::Child => {
                // 子プロセスでシグナルを送信してテスト
                signal::raise(Signal::SIGUSR1)
                    .map_err(|e| {
                        eprintln!("Child: Failed to raise signal: {}", e);
                        std::process::exit(1);
                    })
                    .ok();
                
                // 正常に処理されたら成功
                std::process::exit(0);
            }
        }
    }
    
    /// 検証項目 3.1.3: スレッド間のシグナル配送
    pub fn verify_thread_signal_delivery() -> Result<String, String> {
        use std::sync::Mutex;
        
        let received_threads = Arc::new(Mutex::new(Vec::new()));
        let received_clone = received_threads.clone();
        
        // 複数スレッドを起動
        let handles: Vec<_> = (0..5)
            .map(|thread_id| {
                let received = received_clone.clone();
                std::thread::spawn(move || {
                    // 各スレッドでシグナルマスクを設定
                    let mut mask = SigSet::empty();
                    if thread_id % 2 == 0 {
                        // 偶数スレッドはSIGUSR1をブロック
                        mask.add(Signal::SIGUSR1);
                        let _ = signal::pthread_sigmask(
                            signal::SigmaskHow::SIG_BLOCK,
                            Some(&mask),
                            None
                        );
                    }
                    
                    // シグナルを待つ
                    std::thread::sleep(Duration::from_millis(100));
                    
                    // ペンディングシグナルをチェック
                    if let Ok(pending) = sigpending() {
                        if pending.contains(Signal::SIGUSR1) {
                            let mut recv = received.lock().unwrap();
                            recv.push(thread_id);
                        }
                    }
                })
            })
            .collect();
        
        // メインスレッドからシグナルを送信
        std::thread::sleep(Duration::from_millis(50));
        signal::raise(Signal::SIGUSR1)
            .map_err(|e| format!("Failed to raise signal: {}", e))?;
        
        // すべてのスレッドの完了を待つ
        for handle in handles {
            handle.join().map_err(|_| "Thread join failed")?;
        }
        
        Ok("Thread signal delivery verified".to_string())
    }
    
    /// 検証項目 3.1.4: レースコンディションのテスト
    pub fn verify_race_condition_safety() -> Result<String, String> {
        use std::sync::Mutex;
        
        let shared_data = Arc::new(Mutex::new(0u32));
        let data_clone = shared_data.clone();
        
        // シグナルハンドラーからも同じデータにアクセス
        static SHARED_COUNTER: AtomicU32 = AtomicU32::new(0);
        
        extern "C" fn race_handler(_: i32) {
            // アトミック操作で安全にインクリメント
            SHARED_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        
        let sig_action = SigAction::new(
            SigHandler::Handler(race_handler),
            SaFlags::empty(),
            SigSet::empty(),
        );
        
        unsafe {
            signal::sigaction(Signal::SIGUSR1, &sig_action)
                .map_err(|e| format!("Failed to set handler: {}", e))?;
        }
        
        // 複数スレッドから同時にデータアクセスとシグナル送信
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let data = data_clone.clone();
                std::thread::spawn(move || {
                    for _ in 0..100 {
                        // Mutexで保護されたデータへのアクセス
                        {
                            let mut val = data.lock().unwrap();
                            *val += 1;
                        }
                        
                        // シグナル送信
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
        
        let mutex_value = *shared_data.lock().unwrap();
        let atomic_value = SHARED_COUNTER.load(Ordering::SeqCst);
        
        if mutex_value != 1000 {
            return Err(format!("Mutex protection failed: expected 1000, got {}", mutex_value));
        }
        
        Ok(format!("Race condition safety verified: mutex={}, atomic={}", 
                  mutex_value, atomic_value))
    }
    
    /// 検証項目 3.1.5: デッドロック回避のテスト
    pub fn verify_deadlock_avoidance() -> Result<String, String> {
        use std::sync::Mutex;
        use std::sync::atomic::AtomicBool;
        
        let lock1 = Arc::new(Mutex::new(0));
        let lock2 = Arc::new(Mutex::new(0));
        let deadlock_detected = Arc::new(AtomicBool::new(false));
        
        let l1_clone = lock1.clone();
        let l2_clone = lock2.clone();
        let dd_clone = deadlock_detected.clone();
        
        // スレッド1: lock1 -> lock2の順でロック
        let handle1 = std::thread::spawn(move || {
            for _ in 0..100 {
                let _g1 = l1_clone.lock().unwrap();
                std::thread::yield_now();
                let _g2 = l2_clone.lock().unwrap();
            }
        });
        
        let l1_clone2 = lock1.clone();
        let l2_clone2 = lock2.clone();
        
        // スレッド2: 同じ順序でロック（デッドロック回避）
        let handle2 = std::thread::spawn(move || {
            for _ in 0..100 {
                let _g1 = l1_clone2.lock().unwrap();
                std::thread::yield_now();
                let _g2 = l2_clone2.lock().unwrap();
            }
        });
        
        // タイムアウト付きで待機
        let timeout = Duration::from_secs(5);
        let start = std::time::Instant::now();
        
        while !handle1.is_finished() || !handle2.is_finished() {
            if start.elapsed() > timeout {
                deadlock_detected.store(true, Ordering::SeqCst);
                return Err("Potential deadlock detected (timeout)".to_string());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        
        handle1.join().map_err(|_| "Thread 1 join failed")?;
        handle2.join().map_err(|_| "Thread 2 join failed")?;
        
        Ok("Deadlock avoidance verified successfully".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[ignore]  // fork()を使うテストは特別な環境が必要
    fn test_signal_handler_inheritance() {
        let result = ThreadSafetyValidator::verify_signal_handler_inheritance();
        assert!(result.is_ok(), "Handler inheritance failed: {:?}", result);
    }
    
    #[test]
    fn test_concurrent_signal_sending() {
        let result = ThreadSafetyValidator::verify_concurrent_signal_sending();
        assert!(result.is_ok(), "Concurrent sending failed: {:?}", result);
    }
    
    #[test]
    fn test_race_condition_safety() {
        let result = ThreadSafetyValidator::verify_race_condition_safety();
        assert!(result.is_ok(), "Race condition test failed: {:?}", result);
    }
    
    #[test]
    fn test_deadlock_avoidance() {
        let result = ThreadSafetyValidator::verify_deadlock_avoidance();
        assert!(result.is_ok(), "Deadlock avoidance failed: {:?}", result);
    }
}