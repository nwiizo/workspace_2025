use std::os::unix::process::CommandExt;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
enum ProcessError {
    #[error("Failed to spawn process: {0}")]
    SpawnError(#[from] std::io::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== セキュアなプロセス起動のデモ ===\n");

    // 1. 入力検証とサニタイゼーション
    println!("1. 入力検証とサニタイゼーション:");
    secure_input_handling()?;

    // 2. 権限を落としてプロセスを実行
    println!("\n2. 権限を落としてプロセスを実行:");
    drop_privileges_demo()?;

    // 3. 環境変数のクリーンアップ
    println!("\n3. 環境変数のクリーンアップ:");
    clean_environment()?;

    // 4. リソース制限の設定
    println!("\n4. リソース制限の設定:");
    resource_limits_demo()?;

    // 5. プロセスガードによる自動クリーンアップ
    println!("\n5. プロセスガードによる自動クリーンアップ:");
    process_guard_demo()?;

    Ok(())
}

fn secure_input_handling() -> Result<(), ProcessError> {
    // 危険な入力の例
    let user_inputs = vec![
        "normal_file.txt",
        "../../../etc/passwd", // パストラバーサル
        "file.txt; rm -rf /",  // コマンドインジェクション
        "$(whoami)",           // コマンド置換
        "`id`",                // バッククォート
    ];

    for input in user_inputs {
        println!("  入力: '{}'", input);

        match validate_input(input) {
            Ok(safe_input) => {
                println!("    ✓ 安全な入力: '{}'", safe_input);
                // 安全に使用可能
                let _ = Command::new("echo").arg(&safe_input).output()?;
            }
            Err(e) => {
                println!("    ✗ 拒否: {}", e);
            }
        }
    }

    Ok(())
}

fn validate_input(input: &str) -> Result<String, ProcessError> {
    // 危険な文字のチェック
    let dangerous_chars = [';', '&', '|', '$', '`', '>', '<', '(', ')', '{', '}'];

    for c in dangerous_chars.iter() {
        if input.contains(*c) {
            return Err(ProcessError::InvalidInput(format!(
                "危険な文字 '{}' が含まれています",
                c
            )));
        }
    }

    // パストラバーサルのチェック
    if input.contains("..") {
        return Err(ProcessError::InvalidInput(
            "パストラバーサルの可能性があります".to_string(),
        ));
    }

    // ホワイトリスト方式（英数字、アンダースコア、ドット、ハイフンのみ許可）
    if !input
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == '-')
    {
        return Err(ProcessError::InvalidInput(
            "許可されていない文字が含まれています".to_string(),
        ));
    }

    Ok(input.to_string())
}

fn drop_privileges_demo() -> Result<(), ProcessError> {
    println!("  現在のUID/GIDで実行後、権限を落とす例:");

    let mut cmd = Command::new("id");

    // 通常のUID/GIDで実行
    let output = cmd.output()?;
    println!(
        "    通常実行: {}",
        String::from_utf8_lossy(&output.stdout).trim()
    );

    // 権限を落として実行（実際には適切なUID/GIDを使用）
    #[cfg(unix)]
    {
        let mut cmd = Command::new("id");
        unsafe {
            // 注意: 実際の環境では適切なUID/GIDを使用
            // ここではデモのため、現在のUID/GIDを使用
            let uid = libc::getuid();
            let gid = libc::getgid();

            cmd.uid(uid).gid(gid);
        }

        let output = cmd.output()?;
        println!(
            "    権限設定後: {}",
            String::from_utf8_lossy(&output.stdout).trim()
        );
    }

    Ok(())
}

fn clean_environment() -> Result<(), ProcessError> {
    // 環境変数をクリーンにして実行
    let output = Command::new("sh")
        .arg("-c")
        .arg("echo \"PATH=$PATH\"; echo \"HOME=$HOME\"; echo \"USER=$USER\"")
        .env_clear() // すべての環境変数をクリア
        .env("PATH", "/usr/bin:/bin") // 最小限のPATHのみ設定
        .env("HOME", "/tmp") // 安全なHOME
        .output()?;

    println!("  クリーンな環境での実行結果:");
    println!(
        "{}",
        String::from_utf8_lossy(&output.stdout)
            .trim()
            .replace("\n", "\n    ")
    );

    Ok(())
}

fn resource_limits_demo() -> Result<(), ProcessError> {
    println!("  リソース制限を設定してプロセスを実行:");

    #[cfg(target_os = "linux")]
    {
        use nix::sys::resource::{setrlimit, Resource};

        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg("echo 'プロセス開始'; echo 'プロセス終了'");

        unsafe {
            cmd.pre_exec(|| {
                // CPU時間制限: 10秒
                setrlimit(Resource::RLIMIT_CPU, 10, 10)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

                // プロセス数制限
                setrlimit(Resource::RLIMIT_NPROC, 100, 100)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

                println!("    リソース制限を設定しました");
                Ok(())
            });
        }

        let output = cmd.output()?;
        println!(
            "    実行結果: {}",
            String::from_utf8_lossy(&output.stdout).trim()
        );
    }

    #[cfg(not(target_os = "linux"))]
    {
        println!("    Linux固有の機能のため、このプラットフォームでは利用できません");
    }

    Ok(())
}

// プロセスの自動クリーンアップ
struct ProcessGuard {
    child: Option<std::process::Child>,
    name: String,
}

impl ProcessGuard {
    fn wait(&mut self) -> Result<(), ProcessError> {
        if let Some(mut child) = self.child.take() {
            let status = child.wait()?;
            println!("    ProcessGuard: '{}' が正常終了 ({})", self.name, status);
        }
        Ok(())
    }
}

impl Drop for ProcessGuard {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            println!("    ProcessGuard: '{}' のクリーンアップ", self.name);
            // プロセスがまだ実行中なら終了させる
            let _ = child.kill();
            let _ = child.wait();
            println!("    ProcessGuard: '{}' を強制終了しました", self.name);
        }
    }
}

fn process_guard_demo() -> Result<(), ProcessError> {
    println!("  自動クリーンアップのデモ:");

    {
        let cmd = Command::new("sleep").arg("2").spawn()?;
        let mut guard = ProcessGuard {
            child: Some(cmd),
            name: "sleep_process".to_string(),
        };

        println!("    1秒待機...");
        std::thread::sleep(std::time::Duration::from_secs(1));

        // スコープを抜ける前に正常終了を待つ
        guard.wait()?;
    }

    println!("\n  異常終了時の自動クリーンアップ:");
    {
        let cmd = Command::new("sleep").arg("10").spawn()?;
        let _guard = ProcessGuard {
            child: Some(cmd),
            name: "long_sleep".to_string(),
        };

        println!("    スコープを抜けます（自動的にkillされます）");
        // スコープを抜けるとDropが呼ばれ、プロセスが自動的に終了
    }

    println!("  すべてのプロセスがクリーンアップされました");
    Ok(())
}
