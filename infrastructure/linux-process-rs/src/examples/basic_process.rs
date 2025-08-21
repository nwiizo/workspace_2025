use std::io::{Read, Write};
use std::process::{Command, Stdio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 基本的なプロセス操作のデモ ===\n");

    // 1. シンプルなコマンド実行
    println!("1. シンプルなコマンド実行:");
    basic_process_spawn()?;

    // 2. パイプを使った入出力制御
    println!("\n2. パイプを使った入出力制御:");
    process_with_pipes()?;

    // 3. 環境変数とワーキングディレクトリの設定
    println!("\n3. 環境変数とワーキングディレクトリの設定:");
    spawn_with_environment()?;

    // 4. Unix固有の機能
    #[cfg(unix)]
    {
        println!("\n4. Unix固有の機能:");
        unix_specific_features()?;
    }

    Ok(())
}

fn basic_process_spawn() -> std::io::Result<()> {
    // シンプルなコマンド実行
    let output = Command::new("echo").arg("Hello from Rust!").output()?;

    println!("  stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("  stderr: {}", String::from_utf8_lossy(&output.stderr));
    println!("  status: {}", output.status);

    Ok(())
}

fn process_with_pipes() -> std::io::Result<()> {
    let mut child = Command::new("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // 標準入力に書き込み
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(b"Hello, Rust Process!\n")?;
        stdin.write_all(b"This is line 2\n")?;
        stdin.flush()?;
    }

    // 標準出力から読み込み
    let mut output = String::new();
    if let Some(mut stdout) = child.stdout.take() {
        stdout.read_to_string(&mut output)?;
    }

    // プロセスの終了を待つ
    let status = child.wait()?;
    println!("  受信したデータ: {}", output.trim());
    println!("  子プロセスの終了ステータス: {}", status);

    Ok(())
}

fn spawn_with_environment() -> std::io::Result<()> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("echo \"MY_VAR=$MY_VAR\"; echo \"現在のディレクトリ: $(pwd)\"")
        .env("MY_VAR", "Hello from Rust")
        .current_dir("/tmp")
        .output()?;

    println!("  {}", String::from_utf8_lossy(&output.stdout).trim());
    Ok(())
}

#[cfg(unix)]
fn unix_specific_features() -> std::io::Result<()> {
    use std::os::unix::process::CommandExt;

    let mut cmd = Command::new("echo");
    cmd.arg("Unix specific features demo");

    unsafe {
        cmd.pre_exec(|| {
            // fork後、exec前に実行される
            eprintln!("  pre_exec: プロセスが起動されようとしています");
            Ok(())
        });
    }

    let output = cmd.output()?;
    println!("  {}", String::from_utf8_lossy(&output.stdout).trim());

    Ok(())
}

#[cfg(not(unix))]
fn unix_specific_features() -> std::io::Result<()> {
    println!("  Unix固有の機能はこのプラットフォームでは利用できません");
    Ok(())
}
