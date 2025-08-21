use nix::sys::signal::{sigaction, SigAction, SigHandler, SigSet, Signal};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{fork, ForkResult, Pid};
use std::thread;
use std::time::Duration;

static mut CHILD_EXITED: bool = false;

// SIGCHLDシグナルハンドラ
extern "C" fn handle_sigchld(_: i32) {
    unsafe {
        CHILD_EXITED = true;
    }
    // すべての終了した子プロセスを回収
    loop {
        match waitpid(Pid::from_raw(-1), Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::Exited(pid, status)) => {
                eprintln!("  [SIGCHLD] 子プロセス {} が終了 (status: {})", pid, status);
            }
            Ok(WaitStatus::Signaled(pid, signal, _)) => {
                eprintln!(
                    "  [SIGCHLD] 子プロセス {} がシグナル {:?} で終了",
                    pid, signal
                );
            }
            Ok(WaitStatus::StillAlive) => break,
            Ok(_) => continue,
            Err(_) => break,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ゾンビプロセス対策のデモ ===\n");

    // 1. ゾンビプロセスの発生例（悪い例）
    println!("1. ゾンビプロセスの発生例（悪い例）:");
    demonstrate_zombie()?;

    thread::sleep(Duration::from_secs(1));

    // 2. SIGCHLDハンドラによる自動回収
    println!("\n2. SIGCHLDハンドラによる自動回収:");
    setup_zombie_reaper()?;

    thread::sleep(Duration::from_secs(1));

    // 3. ダブルフォークによる孤児プロセス化
    println!("\n3. ダブルフォークによる孤児プロセス化:");
    double_fork_technique()?;

    Ok(())
}

fn demonstrate_zombie() -> Result<(), Box<dyn std::error::Error>> {
    println!("  子プロセスを生成してすぐに終了させます（wait()しない）");

    match unsafe { fork() }? {
        ForkResult::Parent { child } => {
            println!("  親: 子プロセス {} を生成", child);
            println!("  親: wait()せずに3秒間スリープ...");
            println!("  親: この間、子プロセスはゾンビ状態になります");

            // psコマンドでゾンビプロセスを確認
            thread::sleep(Duration::from_millis(500));
            println!("\n  現在のプロセス状態:");
            std::process::Command::new("ps").args(["aux"]).status()?;

            thread::sleep(Duration::from_secs(2));

            // 最後にwait()して回収
            println!("\n  親: wait()で子プロセスを回収");
            waitpid(Some(child), None)?;
            println!("  親: ゾンビプロセスが回収されました");
        }
        ForkResult::Child => {
            println!("  子: すぐに終了します");
            std::process::exit(0);
        }
    }

    Ok(())
}

fn setup_zombie_reaper() -> Result<(), Box<dyn std::error::Error>> {
    // SIGCHLDシグナルハンドラを設定
    let sig_action = SigAction::new(
        SigHandler::Handler(handle_sigchld),
        nix::sys::signal::SaFlags::SA_RESTART,
        SigSet::empty(),
    );

    unsafe {
        sigaction(Signal::SIGCHLD, &sig_action)?;
    }
    println!("  SIGCHLDハンドラを設定しました");

    // 複数の子プロセスを生成
    for i in 0..3 {
        match unsafe { fork() }? {
            ForkResult::Parent { child } => {
                println!("  親: 子プロセス {} (PID={}) を生成", i + 1, child);
            }
            ForkResult::Child => {
                let delay = (i + 1) as u64;
                println!("    子 {}: {}秒後に終了", i + 1, delay);
                thread::sleep(Duration::from_secs(delay));
                std::process::exit(i);
            }
        }
    }

    // 親プロセスは他の作業を続ける
    println!("\n  親: 他の作業を実行中...");
    for i in 0..5 {
        thread::sleep(Duration::from_secs(1));
        println!("  親: 作業中... {}/5", i + 1);
        unsafe {
            if CHILD_EXITED {
                println!("  親: 子プロセスの終了を検知（SIGCHLDハンドラで自動回収済み）");
                CHILD_EXITED = false;
            }
        }
    }

    println!("  親: すべての処理が完了");
    Ok(())
}

fn double_fork_technique() -> Result<(), Box<dyn std::error::Error>> {
    println!("  ダブルフォークでデーモンプロセスを作成");

    match unsafe { fork() }? {
        ForkResult::Parent { child } => {
            println!("  親: 第一子プロセス {} を生成", child);
            // 第一子プロセスの終了を待つ
            waitpid(Some(child), None)?;
            println!("  親: 第一子プロセスを回収完了");
        }
        ForkResult::Child => {
            // 第一子プロセス
            println!("    第一子: 第二子プロセスを生成");

            match unsafe { fork() }? {
                ForkResult::Parent { child } => {
                    println!("    第一子: 第二子プロセス {} を生成して即終了", child);
                    // 第一子はすぐに終了
                    std::process::exit(0);
                }
                ForkResult::Child => {
                    // 第二子プロセス（孤児プロセス）
                    println!("      第二子: 孤児プロセスとなりinitが親になります");
                    println!("      第二子: 5秒間実行後に終了");

                    // デーモンプロセスとしての処理
                    for i in 0..5 {
                        thread::sleep(Duration::from_secs(1));
                        eprintln!("      第二子: デーモン処理中... {}/5", i + 1);
                    }

                    println!("      第二子: 処理完了、終了");
                    std::process::exit(0);
                }
            }
        }
    }

    thread::sleep(Duration::from_secs(1));
    println!("\n  親: 第二子プロセスはinitに引き取られ、ゾンビ化を防ぎます");

    Ok(())
}
