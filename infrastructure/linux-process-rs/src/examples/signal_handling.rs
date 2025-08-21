use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use signal_hook::{consts::SIGINT, consts::SIGTERM, iterator::Signals};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== シグナル処理のデモ ===\n");

    // 1. シグナルハンドラの設定
    println!("1. シグナルハンドラの設定:");
    setup_signal_handlers()?;

    // 2. 子プロセスへのシグナル送信
    println!("\n2. 子プロセスへのシグナル送信:");
    send_signal_to_child()?;

    Ok(())
}

fn setup_signal_handlers() -> Result<(), Box<dyn std::error::Error>> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // シグナルハンドラを設定
    let mut signals = Signals::new([SIGINT, SIGTERM])?;

    thread::spawn(move || {
        for sig in signals.forever() {
            match sig {
                SIGINT => {
                    println!("  SIGINT (Ctrl+C) を受信しました");
                    r.store(false, Ordering::SeqCst);
                }
                SIGTERM => {
                    println!("  SIGTERM を受信しました");
                    r.store(false, Ordering::SeqCst);
                }
                _ => unreachable!(),
            }
        }
    });

    println!("  シグナルハンドラを設定しました");
    println!("  5秒間待機中... (Ctrl+Cで中断可能)");

    // メインループ
    let mut count = 0;
    while running.load(Ordering::SeqCst) && count < 5 {
        println!("  作業中... {}/5", count + 1);
        thread::sleep(Duration::from_secs(1));
        count += 1;
    }

    if running.load(Ordering::SeqCst) {
        println!("  正常終了");
    } else {
        println!("  シグナルにより中断されました");
    }

    Ok(())
}

fn send_signal_to_child() -> Result<(), Box<dyn std::error::Error>> {
    println!("  10秒間スリープする子プロセスを起動...");

    let mut child = Command::new("sleep").arg("10").spawn()?;

    let pid = Pid::from_raw(child.id() as i32);
    println!("  子プロセスPID: {}", pid);

    // 2秒待機
    println!("  2秒後にSIGTERMを送信します...");
    thread::sleep(Duration::from_secs(2));

    // SIGTERMを送信
    println!("  SIGTERMを送信");
    kill(pid, Signal::SIGTERM)?;

    // 子プロセスの終了を待つ
    let status = child.wait()?;
    println!("  子プロセスの終了ステータス: {:?}", status);

    // シグナルによる終了かチェック
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            println!("  子プロセスはシグナル {} により終了しました", signal);
        }
    }

    Ok(())
}

// 実際のアプリケーションでの使用例
#[allow(dead_code)]
fn graceful_shutdown_example() -> Result<(), Box<dyn std::error::Error>> {
    let shutdown = Arc::new(AtomicBool::new(false));
    let s = shutdown.clone();

    // グレースフルシャットダウンのためのシグナルハンドラ
    let mut signals = Signals::new([SIGINT, SIGTERM])?;
    thread::spawn(move || {
        for _ in signals.forever() {
            println!("シャットダウンシグナルを受信");
            s.store(true, Ordering::SeqCst);
        }
    });

    // メインループ
    while !shutdown.load(Ordering::SeqCst) {
        // アプリケーションのメイン処理
        thread::sleep(Duration::from_millis(100));
    }

    // クリーンアップ処理
    println!("クリーンアップ処理を実行中...");
    // リソースの解放、接続のクローズなど

    println!("グレースフルシャットダウン完了");
    Ok(())
}
