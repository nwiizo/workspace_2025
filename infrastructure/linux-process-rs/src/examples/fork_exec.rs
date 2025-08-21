use nix::sys::wait::waitpid;
use nix::unistd::{execve, fork, getpid, getppid, ForkResult};
use std::ffi::CString;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Fork/Exec パターンのデモ ===\n");

    // 1. 基本的なfork例
    println!("1. 基本的なfork:");
    basic_fork()?;

    thread::sleep(Duration::from_secs(1));

    // 2. fork + exec例
    println!("\n2. fork + exec:");
    fork_and_exec()?;

    thread::sleep(Duration::from_secs(1));

    // 3. 複数の子プロセス
    println!("\n3. 複数の子プロセス:");
    multiple_children()?;

    Ok(())
}

fn basic_fork() -> Result<(), Box<dyn std::error::Error>> {
    match unsafe { fork() }? {
        ForkResult::Parent { child } => {
            println!("  親プロセス: PID={}, 子PID={}", getpid(), child);

            // 子プロセスの終了を待つ
            let status = waitpid(Some(child), None)?;
            println!("  子プロセスの終了ステータス: {:?}", status);
        }
        ForkResult::Child => {
            println!("  子プロセス: PID={}, 親PID={}", getpid(), getppid());

            // 少し作業をシミュレート
            thread::sleep(Duration::from_millis(500));
            println!("  子プロセス: 処理完了");

            // 子プロセスを終了
            std::process::exit(0);
        }
    }

    Ok(())
}

fn fork_and_exec() -> Result<(), Box<dyn std::error::Error>> {
    match unsafe { fork() }? {
        ForkResult::Parent { child } => {
            println!("  親プロセス: 子プロセス({})を生成", child);

            // 子プロセスの終了を待つ
            let status = waitpid(Some(child), None)?;
            println!("  親プロセス: 子プロセスが終了 - {:?}", status);
        }
        ForkResult::Child => {
            println!("  子プロセス: execveでechoコマンドに置き換え");

            // exec()を使って新しいプログラムに置き換え
            let path = CString::new("/bin/echo")?;
            let args = vec![
                CString::new("echo")?,
                CString::new("Hello")?,
                CString::new("from")?,
                CString::new("exec!")?,
            ];
            let env = vec![CString::new("PATH=/bin:/usr/bin")?];

            // execveは成功時には戻らない
            execve(&path, &args, &env).expect("execve failed");
        }
    }

    Ok(())
}

fn multiple_children() -> Result<(), Box<dyn std::error::Error>> {
    let num_children = 3;
    let mut children = Vec::new();

    for i in 0..num_children {
        match unsafe { fork() }? {
            ForkResult::Parent { child } => {
                println!("  親: 子プロセス {} (PID={}) を生成", i + 1, child);
                children.push(child);
            }
            ForkResult::Child => {
                // 子プロセスの処理
                let my_pid = getpid();
                println!("    子 {}: PID={} で開始", i + 1, my_pid);

                // ランダムな時間作業
                thread::sleep(Duration::from_millis((i as u64 + 1) * 200));

                println!("    子 {}: 処理完了", i + 1);
                std::process::exit(i);
            }
        }
    }

    // すべての子プロセスの終了を待つ
    println!("\n  親: すべての子プロセスの終了を待機中...");
    for child in children {
        match waitpid(Some(child), None)? {
            nix::sys::wait::WaitStatus::Exited(pid, status) => {
                println!("  親: 子プロセス {} が終了 (終了コード: {})", pid, status);
            }
            status => {
                println!("  親: 子プロセスのステータス: {:?}", status);
            }
        }
    }

    println!("\n  親: すべての子プロセスが終了しました");
    Ok(())
}
