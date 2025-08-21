/// 記事セクション3: シグナル送信
/// 
/// std::processではできなかったシグナル送信を実装します。
/// nixクレートを使用してSIGTERMなど特定のシグナルを送信できます。
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::process::Command;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== nixクレートを使ったシグナル送信 ===\n");
    
    // 子プロセスを起動（10秒間スリープ）
    let mut child = Command::new("sleep")
        .arg("10")
        .spawn()?;
    
    let pid = Pid::from_raw(child.id() as i32);
    println!("子プロセス起動: PID={}", pid);
    println!("2秒後にSIGTERMを送信します...");
    
    // 2秒待ってからSIGTERMを送信
    thread::sleep(Duration::from_secs(2));
    println!("SIGTERMを送信...");
    kill(pid, Signal::SIGTERM)?;
    
    // プロセスの終了を確認
    let status = child.wait()?;
    println!("子プロセス終了: {:?}", status);
    
    if status.success() {
        println!("正常終了");
    } else {
        println!("異常終了（シグナルにより終了）");
    }
    
    Ok(())
}