/// 記事セクション3: プロセスグループの管理
/// 
/// 複数のプロセスをグループとして管理し、まとめてシグナルを送信します。
/// nixクレートのsetpgid、killpgを使用します。
use nix::sys::signal::{killpg, Signal};
use nix::unistd::{fork, setpgid, ForkResult, Pid};
use nix::sys::wait::waitpid;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== プロセスグループの管理 ===\n");
    
    match unsafe { fork() }? {
        ForkResult::Parent { child } => {
            // 親プロセス
            println!("親: 子プロセス {} を作成", child);
            
            // 子プロセスを新しいプロセスグループのリーダーにする
            setpgid(child, child)?;
            println!("親: プロセスグループ {} を作成", child);
            
            // さらに子プロセスを作成（同じグループに追加）
            for i in 1..3 {
                match unsafe { fork() }? {
                    ForkResult::Parent { new_child } => {
                        // 新しい子プロセスを同じグループに追加
                        setpgid(new_child, child)?;
                        println!("親: プロセス {} をグループ {} に追加", new_child, child);
                    }
                    ForkResult::Child => {
                        // 子プロセス
                        worker_process(i);
                    }
                }
            }
            
            // 3秒待ってからグループ全体にシグナルを送信
            println!("\n3秒後にグループ全体にSIGTERMを送信します...");
            sleep(Duration::from_secs(3));
            
            println!("親: グループ {} 全体にSIGTERMを送信", child);
            killpg(child, Signal::SIGTERM)?;
            
            // 全ての子プロセスの終了を待つ
            println!("親: 子プロセスの終了を待機中...");
            while let Ok(status) = waitpid(None, None) {
                println!("親: 子プロセスが終了 - {:?}", status);
            }
        }
        ForkResult::Child => {
            // 最初の子プロセス
            worker_process(0);
        }
    }
    
    println!("\n全てのプロセスが終了しました");
    Ok(())
}

/// ワーカープロセスの処理
fn worker_process(id: usize) {
    let my_pid = nix::unistd::getpid();
    
    // 自分をプロセスグループに参加させる
    let _ = setpgid(my_pid, my_pid);
    
    println!("  ワーカー{} (PID: {}): 作業を開始", id, my_pid);
    
    // 作業をシミュレート
    loop {
        sleep(Duration::from_secs(1));
        println!("  ワーカー{}: 作業中...", id);
    }
}