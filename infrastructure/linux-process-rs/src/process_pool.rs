/// プロセスプール - 複数のワーカープロセスを効率的に管理
/// 
/// 複数のワーカープロセスを管理し、タスクを分散処理するための構造体。
/// プロセス数の制限、自動クリーンアップ、状態監視などの機能を提供します。
use crate::errors::{ProcessError, ProcessResult};
use crate::process_guard::ProcessGuard;
use nix::unistd::Pid;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// ワーカープロセスの状態
#[derive(Debug, Clone, PartialEq)]
pub enum WorkerState {
    Running,
    Idle,
    Terminated,
}

/// ワーカープロセスの情報
#[derive(Debug)]
pub struct WorkerInfo {
    pub pid: Pid,
    pub state: WorkerState,
    pub command: String,
    pub started_at: std::time::Instant,
}

/// プロセスプール - 複数のワーカープロセスを管理
pub struct ProcessPool {
    workers: Arc<Mutex<HashMap<Pid, (ProcessGuard, WorkerInfo)>>>,
    max_workers: usize,
    name: String,
}

impl ProcessPool {
    /// 新しいプロセスプールを作成
    /// 
    /// # Arguments
    /// 
    /// * `name` - プールの名前
    /// * `max_workers` - 最大ワーカー数
    /// 
    /// # Example
    /// 
    /// ```
    /// let pool = ProcessPool::new("MyPool", 5);
    /// ```
    pub fn new(name: impl Into<String>, max_workers: usize) -> Self {
        println!("ProcessPool '{}': 最大{}ワーカーで初期化", name.as_ref(), max_workers);
        Self {
            workers: Arc::new(Mutex::new(HashMap::new())),
            max_workers,
            name: name.into(),
        }
    }
    
    /// ワーカープロセスを起動
    /// 
    /// # Arguments
    /// 
    /// * `command` - 実行するコマンド
    /// 
    /// # Returns
    /// 
    /// 起動したプロセスのPID、エラーの場合はProcessError
    pub fn spawn_worker(&self, command: &str) -> ProcessResult<Pid> {
        self.spawn_worker_with_args(command, &[])
    }
    
    /// 引数付きでワーカープロセスを起動
    pub fn spawn_worker_with_args(&self, command: &str, args: &[&str]) -> ProcessResult<Pid> {
        let mut workers = self.workers.lock().unwrap();
        
        // 最大数チェック
        if workers.len() >= self.max_workers {
            return Err(ProcessError::InvalidInput(format!(
                "Maximum workers ({}) reached in pool '{}'",
                self.max_workers, self.name
            )));
        }
        
        // プロセスを起動
        let guard = if args.is_empty() {
            ProcessGuard::new(command)
                .map_err(|e| ProcessError::Io(e))?
        } else {
            ProcessGuard::new_with_args(command, args)
                .map_err(|e| ProcessError::Io(e))?
        };
        
        let pid = guard.pid()
            .ok_or_else(|| ProcessError::InvalidInput("Failed to get PID".into()))?;
        let pid = Pid::from_raw(pid as i32);
        
        let info = WorkerInfo {
            pid,
            state: WorkerState::Running,
            command: if args.is_empty() {
                command.to_string()
            } else {
                format!("{} {}", command, args.join(" "))
            },
            started_at: std::time::Instant::now(),
        };
        
        println!("ProcessPool '{}': ワーカー起動 - PID: {}, Command: {}", 
                 self.name, pid, info.command);
        
        workers.insert(pid, (guard, info));
        Ok(pid)
    }
    
    /// 特定のワーカーを終了
    pub fn terminate_worker(&self, pid: Pid) -> ProcessResult<()> {
        let mut workers = self.workers.lock().unwrap();
        
        if let Some((mut guard, info)) = workers.remove(&pid) {
            println!("ProcessPool '{}': ワーカー終了 - PID: {}", self.name, pid);
            
            // wait()を呼んで確実に終了を待つ
            guard.wait()
                .map_err(|e| ProcessError::Io(e))?;
            
            println!("ProcessPool '{}': ワーカー {} が正常に終了しました", self.name, pid);
            Ok(())
        } else {
            Err(ProcessError::InvalidInput(format!(
                "Worker with PID {} not found in pool '{}'",
                pid, self.name
            )))
        }
    }
    
    /// 全てのワーカーを終了
    pub fn terminate_all(&self) -> ProcessResult<()> {
        let mut workers = self.workers.lock().unwrap();
        let pids: Vec<Pid> = workers.keys().copied().collect();
        
        println!("ProcessPool '{}': 全{}ワーカーを終了します", self.name, pids.len());
        
        for pid in pids {
            if let Some((mut guard, _info)) = workers.remove(&pid) {
                // ProcessGuardのDropが自動的にクリーンアップを行う
                drop(guard);
            }
        }
        
        println!("ProcessPool '{}': 全ワーカーが終了しました", self.name);
        Ok(())
    }
    
    /// アクティブなワーカー数を取得
    pub fn active_workers(&self) -> usize {
        let mut workers = self.workers.lock().unwrap();
        
        // 終了したワーカーを削除
        workers.retain(|_pid, (guard, info)| {
            if guard.is_running() {
                true
            } else {
                println!("ProcessPool '{}': ワーカー {} が終了を検出", self.name, info.pid);
                false
            }
        });
        
        workers.len()
    }
    
    /// ワーカーの情報を取得
    pub fn get_worker_info(&self, pid: Pid) -> Option<WorkerInfo> {
        let workers = self.workers.lock().unwrap();
        workers.get(&pid).map(|(_, info)| info.clone())
    }
    
    /// 全ワーカーの情報を取得
    pub fn list_workers(&self) -> Vec<WorkerInfo> {
        let workers = self.workers.lock().unwrap();
        workers.values().map(|(_, info)| info.clone()).collect()
    }
    
    /// プールのステータスを表示
    pub fn status(&self) {
        let workers = self.workers.lock().unwrap();
        println!("\n=== ProcessPool '{}' Status ===", self.name);
        println!("Active workers: {}/{}", workers.len(), self.max_workers);
        
        for (_, info) in workers.values() {
            let runtime = info.started_at.elapsed();
            println!("  - PID: {}, Command: '{}', Runtime: {:?}", 
                     info.pid, info.command, runtime);
        }
        println!("================================\n");
    }
}

impl Drop for ProcessPool {
    /// プールが破棄される際に全ワーカーを自動的に終了
    fn drop(&mut self) {
        println!("ProcessPool '{}': Dropping, terminating all workers", self.name);
        let _ = self.terminate_all();
    }
}

/// ワーカー情報のClone実装
impl Clone for WorkerInfo {
    fn clone(&self) -> Self {
        Self {
            pid: self.pid,
            state: self.state.clone(),
            command: self.command.clone(),
            started_at: self.started_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_process_pool_spawn() {
        let pool = ProcessPool::new("TestPool", 3);
        
        // ワーカーを起動
        let pid1 = pool.spawn_worker_with_args("sleep", &["0.1"]).unwrap();
        let pid2 = pool.spawn_worker_with_args("sleep", &["0.1"]).unwrap();
        
        assert_eq!(pool.active_workers(), 2);
        
        // 最大数に達するまで起動
        let pid3 = pool.spawn_worker_with_args("sleep", &["0.1"]).unwrap();
        assert_eq!(pool.active_workers(), 3);
        
        // 最大数を超えるとエラー
        assert!(pool.spawn_worker("sleep").is_err());
        
        // 少し待ってワーカーが終了
        thread::sleep(Duration::from_millis(200));
        
        // 終了したワーカーは自動的に削除される
        assert_eq!(pool.active_workers(), 0);
    }
    
    #[test]
    fn test_process_pool_terminate() {
        let pool = ProcessPool::new("TestPool", 5);
        
        // ワーカーを起動
        let pid = pool.spawn_worker_with_args("sleep", &["1"]).unwrap();
        assert_eq!(pool.active_workers(), 1);
        
        // 特定のワーカーを終了
        pool.terminate_worker(pid).unwrap();
        assert_eq!(pool.active_workers(), 0);
    }
    
    #[test]
    fn test_process_pool_auto_cleanup() {
        {
            let pool = ProcessPool::new("TestPool", 2);
            pool.spawn_worker_with_args("sleep", &["1"]).unwrap();
            pool.spawn_worker_with_args("sleep", &["1"]).unwrap();
            
            // プールがスコープを抜けると自動的に全ワーカーが終了
        }
        
        // ここでは全プロセスが終了しているはず
        // ps auxでチェックすることもできる
    }
}