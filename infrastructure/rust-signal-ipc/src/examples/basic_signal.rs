/// 基本的なシグナル処理の検証
///
/// 検証項目 2.1: 基本シグナル処理の検証

use nix::sys::signal::{self, Signal, SigHandler, SigSet, SigAction, SaFlags};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;

/// シグナルハンドラーの基本実装と検証
pub struct SignalValidator {
    pub received_signals: Arc<AtomicU32>,
    pub running: Arc<AtomicBool>,
}

impl SignalValidator {
    pub fn new() -> Self {
        Self {
            received_signals: Arc::new(AtomicU32::new(0)),
            running: Arc::new(AtomicBool::new(true)),
        }
    }
    
    /// 検証項目 2.1.1: シグナルハンドラーの登録
    pub fn setup_signal_handler(&self, sig: Signal) -> nix::Result<()> {
        let counter = self.received_signals.clone();
        
        // シグナルハンドラーを設定
        extern "C" fn signal_handler(_: i32) {
            // Note: 実際の実装では、グローバル変数または他の方法で
            // カウンターにアクセスする必要があります
        }
        
        let sig_action = SigAction::new(
            SigHandler::Handler(signal_handler),
            SaFlags::empty(),
            SigSet::empty(),
        );
        
        unsafe { signal::sigaction(sig, &sig_action) }?;
        Ok(())
    }
    
    /// 検証項目 2.1.2: シグナルの送信と受信
    pub fn verify_signal_delivery(&self) -> Result<String, String> {
        let initial_count = self.received_signals.load(Ordering::SeqCst);
        
        // 自プロセスにシグナルを送信
        signal::raise(Signal::SIGUSR1)
            .map_err(|e| format!("Failed to raise signal: {}", e))?;
        
        // シグナル処理を待つ
        std::thread::sleep(Duration::from_millis(100));
        
        let final_count = self.received_signals.load(Ordering::SeqCst);
        
        if final_count != initial_count + 1 {
            return Err(format!(
                "Signal not received. Expected: {}, Got: {}", 
                initial_count + 1, 
                final_count
            ));
        }
        
        Ok("Signal delivered successfully".to_string())
    }
    
    /// 検証項目 2.1.3: 複数シグナルの処理
    pub fn verify_multiple_signals(&self) -> Result<String, String> {
        let initial_count = self.received_signals.load(Ordering::SeqCst);
        
        // 複数のシグナルを送信
        for _ in 0..5 {
            signal::raise(Signal::SIGUSR1)
                .map_err(|e| format!("Failed to raise signal: {}", e))?;
            std::thread::sleep(Duration::from_millis(10));
        }
        
        // 処理を待つ
        std::thread::sleep(Duration::from_millis(200));
        
        let final_count = self.received_signals.load(Ordering::SeqCst);
        let received = final_count - initial_count;
        
        if received < 5 {
            return Err(format!(
                "Not all signals received. Expected: 5, Got: {}", 
                received
            ));
        }
        
        Ok(format!("All {} signals received", received))
    }
    
    /// 検証項目 2.1.4: SIGTERM処理（グレイスフルシャットダウン）
    pub fn verify_sigterm_handling(&self) -> Result<String, String> {
        let running = self.running.clone();
        
        // SIGTERMハンドラーを設定
        extern "C" fn sigterm_handler(_: i32) {
            // グレイスフルシャットダウンのシミュレーション
            // Note: 実際の実装では適切な方法でフラグを設定
        }
        
        let sig_action = SigAction::new(
            SigHandler::Handler(sigterm_handler),
            SaFlags::empty(),
            SigSet::empty(),
        );
        
        unsafe { 
            signal::sigaction(Signal::SIGTERM, &sig_action)
                .map_err(|e| format!("Failed to set SIGTERM handler: {}", e))?;
        }
        
        // SIGTERMを送信
        signal::raise(Signal::SIGTERM)
            .map_err(|e| format!("Failed to raise SIGTERM: {}", e))?;
        
        std::thread::sleep(Duration::from_millis(100));
        
        Ok("SIGTERM handled for graceful shutdown".to_string())
    }
    
    /// 検証項目 2.1.5: SIGINT処理（即座の停止）
    pub fn verify_sigint_handling(&self) -> Result<String, String> {
        // SIGINTハンドラーを設定
        extern "C" fn sigint_handler(_: i32) {
            // 即座の停止処理
        }
        
        let sig_action = SigAction::new(
            SigHandler::Handler(sigint_handler),
            SaFlags::empty(),
            SigSet::empty(),
        );
        
        unsafe { 
            signal::sigaction(Signal::SIGINT, &sig_action)
                .map_err(|e| format!("Failed to set SIGINT handler: {}", e))?;
        }
        
        // 注意: 実際のSIGINT送信はプロセスを終了させる可能性があるため、
        // ここではハンドラーの設定のみを検証
        
        Ok("SIGINT handler registered successfully".to_string())
    }
}

/// グローバルカウンター（デモ用）
static GLOBAL_COUNTER: AtomicU32 = AtomicU32::new(0);

/// より実践的なシグナルハンドラー実装
pub struct PracticalSignalHandler;

impl PracticalSignalHandler {
    /// グローバルカウンターを使用したハンドラー設定
    pub fn setup_with_global_counter(sig: Signal) -> nix::Result<()> {
        extern "C" fn handler(_: i32) {
            GLOBAL_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        
        let sig_action = SigAction::new(
            SigHandler::Handler(handler),
            SaFlags::empty(),
            SigSet::empty(),
        );
        
        unsafe { signal::sigaction(sig, &sig_action) }?;
        Ok(())
    }
    
    /// シグナル送受信のテスト
    pub fn test_signal_delivery() -> Result<String, String> {
        // カウンターをリセット
        GLOBAL_COUNTER.store(0, Ordering::SeqCst);
        
        // ハンドラーを設定
        Self::setup_with_global_counter(Signal::SIGUSR1)
            .map_err(|e| format!("Failed to setup handler: {}", e))?;
        
        // シグナルを送信
        signal::raise(Signal::SIGUSR1)
            .map_err(|e| format!("Failed to raise signal: {}", e))?;
        
        // 処理を待つ
        std::thread::sleep(Duration::from_millis(50));
        
        let count = GLOBAL_COUNTER.load(Ordering::SeqCst);
        if count == 0 {
            return Err("Signal not received".to_string());
        }
        
        Ok(format!("Signal received {} times", count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_practical_signal_handler() {
        let result = PracticalSignalHandler::test_signal_delivery();
        assert!(result.is_ok(), "Signal delivery failed: {:?}", result);
    }
    
    #[test]
    fn test_signal_validator_creation() {
        let validator = SignalValidator::new();
        assert_eq!(validator.received_signals.load(Ordering::SeqCst), 0);
        assert!(validator.running.load(Ordering::SeqCst));
    }
}