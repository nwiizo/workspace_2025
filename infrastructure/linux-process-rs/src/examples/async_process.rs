use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::task::JoinSet;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 非同期プロセス管理のデモ (Tokio) ===\n");

    // 1. 基本的な非同期コマンド実行
    println!("1. 基本的な非同期コマンド実行:");
    basic_async_command().await?;

    // 2. ストリーミング出力の処理
    println!("\n2. ストリーミング出力の処理:");
    streaming_output().await?;

    // 3. 複数プロセスの並行実行
    println!("\n3. 複数プロセスの並行実行:");
    concurrent_processes().await?;

    // 4. タイムアウト処理
    println!("\n4. タイムアウト処理:");
    process_with_timeout().await?;

    // 5. パイプラインの構築
    println!("\n5. パイプラインの構築:");
    pipeline_example().await?;

    Ok(())
}

async fn basic_async_command() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("echo")
        .arg("Hello from async Rust!")
        .output()
        .await?;

    println!("  出力: {}", String::from_utf8_lossy(&output.stdout));
    println!("  ステータス: {}", output.status);

    Ok(())
}

async fn streaming_output() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new("sh");
    cmd.arg("-c")
        .arg("for i in 1 2 3 4 5; do echo \"Line $i\"; sleep 0.5; done")
        .stdout(std::process::Stdio::piped());

    let mut child = cmd.spawn()?;

    let stdout = child.stdout.take().expect("Failed to get stdout");

    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();

    println!("  ストリーミング出力:");
    while let Some(line) = lines.next_line().await? {
        println!("    >>> {}", line);
    }

    let status = child.wait().await?;
    println!("  プロセス終了: {}", status);

    Ok(())
}

async fn concurrent_processes() -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = JoinSet::new();

    println!("  5つのプロセスを並行実行:");

    for i in 0..5 {
        tasks.spawn(async move {
            let sleep_time = (5 - i) as f32 / 2.0;
            let output = Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "sleep {}; echo 'Task {} completed after {}s'",
                    sleep_time, i, sleep_time
                ))
                .output()
                .await
                .expect("Failed to execute command");

            (i, String::from_utf8_lossy(&output.stdout).to_string())
        });
    }

    // すべてのタスクの完了を待つ
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok((id, output)) => print!("    Task {}: {}", id, output),
            Err(e) => eprintln!("    Task failed: {}", e),
        }
    }

    Ok(())
}

async fn process_with_timeout() -> Result<(), Box<dyn std::error::Error>> {
    println!("  10秒のsleepコマンドを5秒でタイムアウト:");

    let mut child = Command::new("sleep").arg("10").spawn()?;

    match timeout(Duration::from_secs(5), child.wait()).await {
        Ok(Ok(status)) => {
            println!("    プロセスが完了: {}", status);
        }
        Ok(Err(e)) => {
            println!("    プロセスエラー: {}", e);
        }
        Err(_) => {
            println!("    タイムアウト！プロセスを強制終了...");
            child.kill().await?;
            println!("    プロセスを強制終了しました");
        }
    }

    Ok(())
}

async fn pipeline_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("  パイプライン: echo | grep | wc -l");

    // シンプルなアプローチ: シェル経由で実行
    let output = Command::new("sh")
        .arg("-c")
        .arg("echo 'line1\nline2\ntest\nline3\ntest' | grep line | wc -l")
        .output()
        .await?;

    let result = String::from_utf8_lossy(&output.stdout);
    println!("    'line'を含む行数: {}", result.trim());

    Ok(())
}
