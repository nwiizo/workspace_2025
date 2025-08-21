/// 記事セクション1: パイプを使った入出力制御
/// 
/// 子プロセスとパイプで通信する例を示します。
/// stdin/stdoutを通じて親子プロセス間でデータをやり取りします。
use std::io::Write;
use std::process::{Command, Stdio};

fn main() -> std::io::Result<()> {
    println!("=== パイプを使った入出力制御 ===\n");
    
    let mut child = Command::new("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    
    // 標準入力に書き込み
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(b"Hello from parent process!\n")?;
        stdin.write_all(b"This is line 2\n")?;
        stdin.write_all(b"Goodbye!\n")?;
    }
    
    // 出力を取得
    let output = child.wait_with_output()?;
    println!("子プロセスからの出力:");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    
    println!("終了ステータス: {}", output.status);
    
    Ok(())
}