/// Unix Domain Socketクライアントのシンプルな実装
///
/// サーバーと通信する基本的なクライアント

use std::os::unix::net::UnixStream;
use std::io::{Write, BufReader, BufRead};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Unix Domain Socket クライアントを起動しました");
    
    // サーバーに接続
    let mut stream = match UnixStream::connect("/tmp/rust_ipc.sock") {
        Ok(stream) => {
            println!("サーバーに接続しました");
            stream
        }
        Err(e) => {
            eprintln!("サーバーへの接続に失敗しました: {}", e);
            eprintln!("サーバーが起動していることを確認してください");
            return Err(e.into());
        }
    };
    
    println!("コマンドを入力してください:");
    println!("  - 'ping': サーバーの応答を確認");
    println!("  - 'time': サーバーの現在時刻を取得");
    println!("  - 'quit': 終了");
    println!("  - その他: エコーバック（大文字変換）");
    println!();
    
    let stdin = std::io::stdin();
    let mut input = String::new();
    let mut reader = BufReader::new(stream.try_clone()?);
    
    loop {
        print!("> ");
        std::io::stdout().flush()?;
        
        input.clear();
        stdin.read_line(&mut input)?;
        let message = input.trim();
        
        if message.is_empty() {
            continue;
        }
        
        // サーバーにメッセージを送信
        stream.write_all(format!("{}\n", message).as_bytes())?;
        stream.flush()?;
        
        // サーバーからの応答を読み取り
        let mut response = String::new();
        reader.read_line(&mut response)?;
        println!("サーバー応答: {}", response.trim());
        
        if message == "quit" {
            println!("クライアントを終了します");
            break;
        }
    }
    
    Ok(())
}