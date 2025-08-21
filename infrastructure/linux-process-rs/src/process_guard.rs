/// ProcessGuardパターン - RAIIを活用した安全なプロセス管理
/// 
/// プロセスのライフサイクルを確実に管理するための構造体。
/// Dropトレイトを実装することで、スコープを抜ける際に
/// 自動的にプロセスをクリーンアップします。
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

/// プロセスの自動クリーンアップを保証する構造体
pub struct ProcessGuard {
    child: Option<Child>,
    name: String,
}

impl ProcessGuard {
    /// 新しいプロセスを起動してProcessGuardでラップする
    /// 
    /// # Arguments
    /// 
    /// * `command` - 実行するコマンド
    /// 
    /// # Returns
    /// 
    /// ProcessGuardインスタンス、起動に失敗した場合はエラー
    /// 
    /// # Example
    /// 
    /// ```
    /// let guard = ProcessGuard::new("sleep")?;
    /// // guardがスコープを抜けると自動的にプロセスが終了
    /// ```
    pub fn new(command: &str) -> std::io::Result<Self> {
        println!("ProcessGuard: '{}' を起動", command);
        let child = Command::new(command).spawn()?;
        let pid = child.id();
        println!("ProcessGuard: PID {} で起動しました", pid);
        
        Ok(Self {
            child: Some(child),
            name: command.to_string(),
        })
    }
    
    /// 引数付きでプロセスを起動
    pub fn new_with_args(command: &str, args: &[&str]) -> std::io::Result<Self> {
        println!("ProcessGuard: '{}' を引数 {:?} で起動", command, args);
        let child = Command::new(command)
            .args(args)
            .spawn()?;
        let pid = child.id();
        println!("ProcessGuard: PID {} で起動しました", pid);
        
        Ok(Self {
            child: Some(child),
            name: format!("{} {:?}", command, args),
        })
    }
    
    /// プロセスの終了を待つ
    pub fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> {
        if let Some(mut child) = self.child.take() {
            println!("ProcessGuard: プロセス '{}' の終了を待機", self.name);
            let status = child.wait()?;
            println!("ProcessGuard: プロセス '{}' が終了: {:?}", self.name, status);
            Ok(status)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Process already terminated"
            ))
        }
    }
    
    /// プロセスがまだ実行中かチェック
    pub fn is_running(&self) -> bool {
        if let Some(ref child) = self.child {
            // try_wait()はmutableな参照が必要ないはず
            // 直接ステータスを確認する別の方法を使用
            #[cfg(unix)]
            {
                use nix::sys::signal::kill;
                use nix::unistd::Pid;
                let pid = Pid::from_raw(child.id() as i32);
                // シグナル0を送信してプロセスの存在を確認
                kill(pid, None).is_ok()
            }
            #[cfg(not(unix))]
            {
                // Windows等の場合は常にtrueを返す（正確な判定は難しい）
                true
            }
        } else {
            false
        }
    }
    
    /// プロセスのPIDを取得
    pub fn pid(&self) -> Option<u32> {
        self.child.as_ref().map(|c| c.id())
    }
}

impl Drop for ProcessGuard {
    /// ProcessGuardがスコープを抜ける際に自動的に呼ばれる
    /// 
    /// 1. まずSIGTERMで優雅に終了を試みる
    /// 2. 500ms待つ
    /// 3. まだ生きていればSIGKILLで強制終了
    /// 4. 必ずwait()してゾンビプロセスを防ぐ
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            // まだ実行中かチェック
            if child.try_wait().ok().flatten().is_none() {
                eprintln!("ProcessGuard: プロセス '{}' を終了します", self.name);
                
                let pid = Pid::from_raw(child.id() as i32);
                
                // まずSIGTERMで優雅に終了を試みる
                if let Err(e) = kill(pid, Signal::SIGTERM) {
                    eprintln!("ProcessGuard: SIGTERM送信失敗: {}", e);
                } else {
                    eprintln!("ProcessGuard: SIGTERMを送信しました");
                }
                
                // 少し待つ（優雅な終了のため）
                thread::sleep(Duration::from_millis(500));
                
                // まだ生きていればSIGKILL
                if child.try_wait().ok().flatten().is_none() {
                    eprintln!("ProcessGuard: プロセスがまだ実行中、SIGKILLで強制終了");
                    if let Err(e) = child.kill() {
                        eprintln!("ProcessGuard: SIGKILL失敗: {}", e);
                    }
                }
                
                // 必ずwait()してゾンビプロセスを防ぐ
                match child.wait() {
                    Ok(status) => {
                        eprintln!("ProcessGuard: プロセス '{}' 終了: {:?}", self.name, status);
                    }
                    Err(e) => {
                        eprintln!("ProcessGuard: wait()失敗: {}", e);
                    }
                }
            } else {
                println!("ProcessGuard: プロセス '{}' は既に終了済み", self.name);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn test_process_guard_auto_cleanup() {
        let start = Instant::now();
        
        {
            let _guard = ProcessGuard::new_with_args("sleep", &["1"]).unwrap();
            // スコープを抜けると自動的にクリーンアップ
        }
        
        // 1秒のsleepが中断されるので、1秒未満で終了するはず
        assert!(start.elapsed() < Duration::from_secs(1));
    }
    
    #[test]
    fn test_process_guard_wait() {
        let mut guard = ProcessGuard::new_with_args("echo", &["test"]).unwrap();
        let status = guard.wait().unwrap();
        assert!(status.success());
    }
    
    #[test]
    fn test_process_guard_is_running() {
        let mut guard = ProcessGuard::new_with_args("sleep", &["0.1"]).unwrap();
        assert!(guard.is_running());
        
        thread::sleep(Duration::from_millis(200));
        assert!(!guard.is_running());
    }
}