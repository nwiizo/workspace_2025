/// シグナルマスクの検証
///
/// 検証項目 2.2: シグナルマスクの動作検証

use nix::sys::signal::{self, Signal, SigSet, pthread_sigmask, SigmaskHow};
use std::time::Duration;
use anyhow::Result;

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

/// シグナルマスクの動作を検証するバリデーター
pub struct SignalMaskValidator;

impl SignalMaskValidator {
    /// 検証項目 2.2.1: シグナルのブロック
    pub fn verify_signal_blocking() -> Result<String, String> {
        let mut mask = SigSet::empty();
        mask.add(Signal::SIGUSR1);
        
        // シグナルをブロック
        pthread_sigmask(
            signal::SigmaskHow::SIG_BLOCK,
            Some(&mask),
            None
        ).map_err(|e| format!("Failed to block signal: {}", e))?;
        
        // ブロック中にシグナルを送信
        signal::raise(Signal::SIGUSR1)
            .map_err(|e| format!("Failed to raise signal: {}", e))?;
        
        // ペンディング状態を確認
        let pending = sigpending()
            .map_err(|e| format!("Failed to get pending signals: {}", e))?;
        
        if !pending.contains(Signal::SIGUSR1) {
            return Err("Signal not pending after blocking".to_string());
        }
        
        // ブロック解除
        pthread_sigmask(
            signal::SigmaskHow::SIG_UNBLOCK,
            Some(&mask),
            None
        ).map_err(|e| format!("Failed to unblock signal: {}", e))?;
        
        // 少し待ってシグナルが処理されるのを待つ
        std::thread::sleep(Duration::from_millis(50));
        
        Ok("Signal blocking and unblocking verified successfully".to_string())
    }
    
    /// 検証項目 2.2.2: 複数シグナルの同時マスク
    pub fn verify_multiple_signal_masking() -> Result<String, String> {
        let mut mask = SigSet::empty();
        mask.add(Signal::SIGUSR1);
        mask.add(Signal::SIGUSR2);
        
        // 複数シグナルを同時にブロック
        pthread_sigmask(
            signal::SigmaskHow::SIG_SETMASK,
            Some(&mask),
            None
        ).map_err(|e| format!("Failed to set signal mask: {}", e))?;
        
        // 各シグナルを送信
        for sig in &[Signal::SIGUSR1, Signal::SIGUSR2] {
            signal::raise(*sig)
                .map_err(|e| format!("Failed to raise {:?}: {}", sig, e))?;
        }
        
        // すべてペンディング状態か確認
        let pending = sigpending()
            .map_err(|e| format!("Failed to get pending signals: {}", e))?;
        
        for sig in &[Signal::SIGUSR1, Signal::SIGUSR2] {
            if !pending.contains(*sig) {
                return Err(format!("{:?} not pending", sig));
            }
        }
        
        // マスクをクリア
        let empty_mask = SigSet::empty();
        pthread_sigmask(
            signal::SigmaskHow::SIG_SETMASK,
            Some(&empty_mask),
            None
        ).map_err(|e| format!("Failed to clear signal mask: {}", e))?;
        
        Ok("Multiple signal masking verified successfully".to_string())
    }
    
    /// 検証項目 2.2.3: シグナルマスクの継承
    pub fn verify_mask_inheritance() -> Result<String, String> {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};
        
        let mut mask = SigSet::empty();
        mask.add(Signal::SIGUSR1);
        
        // 親スレッドでシグナルをブロック
        pthread_sigmask(
            signal::SigmaskHow::SIG_BLOCK,
            Some(&mask),
            None
        ).map_err(|e| format!("Failed to block signal in parent: {}", e))?;
        
        let mask_inherited = Arc::new(AtomicBool::new(false));
        let mask_inherited_clone = mask_inherited.clone();
        
        // 子スレッドを生成
        let handle = std::thread::spawn(move || {
            // 子スレッドで現在のマスクを取得
            let mut current_mask = SigSet::empty();
            if let Ok(_) = signal::pthread_sigmask(
                signal::SigmaskHow::SIG_BLOCK,
                None,
                Some(&mut current_mask)
            ) {
                // SIGUSR1がブロックされているか確認
                if current_mask.contains(Signal::SIGUSR1) {
                    mask_inherited_clone.store(true, Ordering::SeqCst);
                }
            }
        });
        
        handle.join().map_err(|_| "Thread join failed")?;
        
        // マスクをクリア
        let empty_mask = SigSet::empty();
        pthread_sigmask(
            signal::SigmaskHow::SIG_SETMASK,
            Some(&empty_mask),
            None
        ).map_err(|e| format!("Failed to clear mask: {}", e))?;
        
        if mask_inherited.load(Ordering::SeqCst) {
            Ok("Signal mask inheritance verified".to_string())
        } else {
            Err("Signal mask not inherited by child thread".to_string())
        }
    }
    
    /// 検証項目 2.2.4: シグナルマスクの保存と復元
    pub fn verify_mask_save_restore() -> Result<String, String> {
        // 元のマスクを保存
        let mut original_mask = SigSet::empty();
        pthread_sigmask(
            signal::SigmaskHow::SIG_SETMASK,
            None,
            Some(&mut original_mask)
        ).map_err(|e| format!("Failed to get original mask: {}", e))?;
        
        // 新しいマスクを設定
        let mut new_mask = SigSet::empty();
        new_mask.add(Signal::SIGUSR1);
        new_mask.add(Signal::SIGUSR2);
        
        pthread_sigmask(
            signal::SigmaskHow::SIG_SETMASK,
            Some(&new_mask),
            None
        ).map_err(|e| format!("Failed to set new mask: {}", e))?;
        
        // 現在のマスクを確認
        let mut current_mask = SigSet::empty();
        pthread_sigmask(
            signal::SigmaskHow::SIG_SETMASK,
            None,
            Some(&mut current_mask)
        ).map_err(|e| format!("Failed to get current mask: {}", e))?;
        
        if !current_mask.contains(Signal::SIGUSR1) || !current_mask.contains(Signal::SIGUSR2) {
            return Err("New mask not properly set".to_string());
        }
        
        // 元のマスクを復元
        pthread_sigmask(
            signal::SigmaskHow::SIG_SETMASK,
            Some(&original_mask),
            None
        ).map_err(|e| format!("Failed to restore original mask: {}", e))?;
        
        Ok("Signal mask save/restore verified successfully".to_string())
    }
    
    /// 検証項目 2.2.5: シグナルセットの操作
    pub fn verify_sigset_operations() -> Result<String, String> {
        // 空のシグナルセットを作成
        let mut sigset = SigSet::empty();
        
        // シグナルを追加
        sigset.add(Signal::SIGUSR1);
        sigset.add(Signal::SIGUSR2);
        sigset.add(Signal::SIGTERM);
        
        // シグナルが含まれているか確認
        if !sigset.contains(Signal::SIGUSR1) {
            return Err("SIGUSR1 not in sigset".to_string());
        }
        
        // シグナルを削除
        sigset.remove(Signal::SIGUSR2);
        
        if sigset.contains(Signal::SIGUSR2) {
            return Err("SIGUSR2 still in sigset after removal".to_string());
        }
        
        // 全シグナルを含むセットを作成
        let full_set = SigSet::all();
        
        // 一般的なシグナルが含まれているか確認
        if !full_set.contains(Signal::SIGTERM) {
            return Err("Full sigset doesn't contain SIGTERM".to_string());
        }
        
        // セットをクリア
        sigset.clear();
        
        if sigset.contains(Signal::SIGUSR1) {
            return Err("Sigset not cleared properly".to_string());
        }
        
        Ok("Signal set operations verified successfully".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_signal_blocking() {
        let result = SignalMaskValidator::verify_signal_blocking();
        assert!(result.is_ok(), "Signal blocking test failed: {:?}", result);
    }
    
    #[test]
    fn test_multiple_signal_masking() {
        let result = SignalMaskValidator::verify_multiple_signal_masking();
        assert!(result.is_ok(), "Multiple signal masking failed: {:?}", result);
    }
    
    #[test]
    fn test_sigset_operations() {
        let result = SignalMaskValidator::verify_sigset_operations();
        assert!(result.is_ok(), "Sigset operations failed: {:?}", result);
    }
    
    #[test]
    fn test_mask_save_restore() {
        let result = SignalMaskValidator::verify_mask_save_restore();
        assert!(result.is_ok(), "Mask save/restore failed: {:?}", result);
    }
}