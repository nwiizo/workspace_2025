/// Unix Domain Socketサーバーのシンプルな実装
///
/// 基本的なメッセージの送受信を行うサーバー

use anyhow::{Context, Result};
use std::os::unix::net::{UnixStream, UnixListener};
use std::io::{Write, BufReader, BufRead};
use std::thread;
use std::path::Path;
use tracing::{info, error};

fn main() -> Result<()> {
    // ログの初期化
    tracing_subscriber::fmt::init();
    
    let socket_path = "/tmp/rust_ipc.sock";
    
    // 既存のソケットファイルを削除
    if Path::new(socket_path).exists() {
        std::fs::remove_file(socket_path)
            .context("既存のソケットファイルの削除に失敗")?;
    }
    
    let listener = UnixListener::bind(socket_path)
        .context("Unix Domain Socketのバインドに失敗")?;
    
    info!("Unix Domain Socket サーバーが起動しました: {}", socket_path);
    info!("クライアントからの接続を待機中...");
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                info!("新しいクライアントが接続しました");
                thread::spawn(move || {
                    if let Err(e) = handle_client(stream) {
                        error!("クライアント処理エラー: {}", e);
                    }
                });
            }
            Err(err) => {
                error!("接続エラー: {}", err);
            }
        }
    }
    
    Ok(())
}

/// クライアント接続を処理
fn handle_client(mut stream: UnixStream) -> Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut buffer = String::new();
    
    loop {
        buffer.clear();
        let bytes_read = reader.read_line(&mut buffer)?;
        
        if bytes_read == 0 {
            info!("クライアントが切断しました");
            break;
        }
        
        let message = buffer.trim();
        info!("受信: {}", message);
        
        // 簡単なコマンド処理
        let response = match message {
            "quit" => {
                stream.write_all(b"Goodbye!\n")?;
                break;
            }
            "ping" => "pong\n".to_string(),
            "time" => {
                let time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
                format!("{}\n", time)
            }
            _ => {
                // エコーバック
                format!("ECHO: {}\n", message.to_uppercase())
            }
        };
        
        stream.write_all(response.as_bytes())?;
        stream.flush()?;
    }
    
    Ok(())
}