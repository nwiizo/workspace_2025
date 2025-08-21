use nix::sys::signal::{killpg, Signal};
use nix::unistd::{fork, getpgid, getpid, setpgid, setsid, ForkResult};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== プロセスグループとセッション管理のデモ ===\n");

    // 1. プロセスグループの作成と管理
    println!("1. プロセスグループの作成と管理:");
    process_group_demo()?;

    thread::sleep(Duration::from_secs(1));

    // 2. セッションリーダーの作成
    println!("\n2. セッションリーダーの作成:");
    session_leader_demo()?;

    thread::sleep(Duration::from_secs(1));

    // 3. デーモン化の実装
    println!("\n3. デーモン化の実装:");
    daemonize_demo()?;

    Ok(())
}

fn process_group_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("  プロセスグループを作成して管理します");

    match unsafe { fork() }? {
        ForkResult::Parent { child } => {
            let parent_pid = getpid();
            let parent_pgid = getpgid(Some(parent_pid))?;
            println!("  親: PID={}, PGID={}", parent_pid, parent_pgid);

            // 子プロセスを新しいプロセスグループのリーダーにする
            setpgid(child, child)?;
            println!(
                "  親: 子プロセス {} を新しいプロセスグループ {} のリーダーに設定",
                child, child
            );

            // プロセスグループ全体にシグナルを送信する準備
            thread::sleep(Duration::from_secs(2));

            println!("  親: プロセスグループ {} にSIGTERMを送信", child);
            killpg(child, Signal::SIGTERM)?;

            println!("  親: 処理完了");
        }
        ForkResult::Child => {
            let child_pid = getpid();

            // 新しいプロセスグループを作成
            setpgid(child_pid, child_pid)?;

            let child_pgid = getpgid(Some(child_pid))?;
            println!(
                "    子: PID={}, PGID={} (新しいグループリーダー)",
                child_pid, child_pgid
            );

            // さらに子プロセスを生成（同じプロセスグループ）
            for i in 0..2 {
                match unsafe { fork() }? {
                    ForkResult::Parent { .. } => {
                        // 第一子プロセスは続行
                    }
                    ForkResult::Child => {
                        let grandchild_pid = getpid();
                        let grandchild_pgid = getpgid(Some(grandchild_pid))?;
                        println!(
                            "      孫{}: PID={}, PGID={}",
                            i + 1,
                            grandchild_pid,
                            grandchild_pgid
                        );

                        // 孫プロセスの処理
                        loop {
                            thread::sleep(Duration::from_secs(1));
                            eprintln!("      孫{}: 実行中...", i + 1);
                        }
                    }
                }
            }

            // 第一子プロセスの処理
            loop {
                thread::sleep(Duration::from_secs(1));
                eprintln!("    子: 実行中...");
            }
        }
    }

    Ok(())
}

fn session_leader_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("  新しいセッションを作成します");

    match unsafe { fork() }? {
        ForkResult::Parent { child } => {
            println!("  親: 子プロセス {} を生成", child);
            thread::sleep(Duration::from_secs(3));
            println!("  親: 処理完了");
        }
        ForkResult::Child => {
            let pid = getpid();
            println!("    子: PID={}", pid);

            // 新しいセッションを作成（セッションリーダーになる）
            let sid = setsid()?;
            println!("    子: 新しいセッションID={} (セッションリーダー)", sid);

            // セッションリーダーとしての処理
            for i in 0..3 {
                thread::sleep(Duration::from_secs(1));
                eprintln!("    子: セッションリーダーとして実行中... {}/3", i + 1);
            }

            println!("    子: セッション処理完了");
            std::process::exit(0);
        }
    }

    Ok(())
}

fn daemonize_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("  完全なデーモン化プロセスを実行");

    // ステップ1: 最初のfork
    match unsafe { fork() }? {
        ForkResult::Parent { child } => {
            println!("  親: 第一子プロセス {} を生成して終了", child);
            // 親プロセスは終了
            return Ok(());
        }
        ForkResult::Child => {
            // 第一子プロセス
        }
    }

    // ステップ2: 新しいセッションを作成
    let sid = setsid()?;
    eprintln!("  デーモン: 新しいセッションID={}", sid);

    // ステップ3: 2回目のfork（セッションリーダーにならないため）
    match unsafe { fork() }? {
        ForkResult::Parent { .. } => {
            // 第一子プロセスは終了
            std::process::exit(0);
        }
        ForkResult::Child => {
            // 第二子プロセス（真のデーモン）
        }
    }

    // ステップ4: ワーキングディレクトリを変更
    std::env::set_current_dir("/")?;
    eprintln!("  デーモン: ワーキングディレクトリを / に変更");

    // ステップ5: ファイルモード作成マスクをクリア
    unsafe {
        libc::umask(0);
    }

    // ステップ6: 標準入出力を閉じる（本番環境では/dev/nullにリダイレクト）
    eprintln!("  デーモン: デーモン化完了、バックグラウンドで実行中...");

    // デーモンとしての処理
    for i in 0..5 {
        thread::sleep(Duration::from_secs(1));
        // 通常はログファイルに出力
        eprintln!("  デーモン: バックグラウンド処理中... {}/5", i + 1);
    }

    eprintln!("  デーモン: 処理完了、終了");
    Ok(())
}
