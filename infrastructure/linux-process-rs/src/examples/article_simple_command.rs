/// 記事セクション1: 最初の一歩 - シンプルなコマンド実行
/// 
/// Rustでプロセスを扱う最も簡単な方法を示します。
/// std::process::Commandを使った基本的な例です。
use std::process::Command;

fn main() {
    println!("=== シンプルなコマンド実行 ===\n");
    
    // 最もシンプルな例
    let output = Command::new("echo")
        .arg("Hello, Rust!")
        .output()
        .expect("Failed to execute command");
    
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    println!("終了ステータス: {}", output.status);
}