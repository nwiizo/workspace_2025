/// 実践的なIPCクライアント実装
///
/// 構造化されたメッセージプロトコルを使用してサーバーと通信

use anyhow::{Context, Result};
use rust_signal_ipc::ipc::{IPCMessage, MessageType};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};
use tracing::{info, error, debug};

fn main() -> Result<()> {
    // ログの初期化
    tracing_subscriber::fmt::init();
    
    info!("IPCクライアントを起動");
    
    // サーバーに接続
    let mut stream = UnixStream::connect("/tmp/rust_ipc_structured.sock")
        .context("サーバーへの接続に失敗")?;
    
    // タイムアウト設定
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    
    info!("サーバーに接続しました");
    
    // 対話的モード
    println!("\nコマンドを入力してください:");
    println!("  ping      - 接続確認");
    println!("  time      - サーバーの現在時刻");
    println!("  info      - サーバー情報");
    println!("  echo TEXT - エコーバック");
    println!("  calc:A op B - 簡単な計算 (例: calc:10 + 20)");
    println!("  notify TEXT - 通知送信");
    println!("  heartbeat - ハートビート送信");
    println!("  stress N  - N個のメッセージを連続送信");
    println!("  quit      - 終了");
    println!();
    
    let stdin = std::io::stdin();
    let mut input = String::new();
    
    loop {
        print!("> ");
        std::io::stdout().flush()?;
        
        input.clear();
        stdin.read_line(&mut input)?;
        let command = input.trim();
        
        if command.is_empty() {
            continue;
        }
        
        if command == "quit" {
            info!("クライアントを終了します");
            break;
        }
        
        // ストレステストの処理
        if command.starts_with("stress ") {
            if let Ok(count) = command[7..].parse::<usize>() {
                run_stress_test(&mut stream, count)?;
                continue;
            }
        }
        
        // メッセージタイプを決定
        let (message_type, payload) = if command == "heartbeat" {
            (MessageType::Heartbeat, b"".to_vec())
        } else if command.starts_with("notify ") {
            let text = &command[7..];
            (MessageType::Notification, text.as_bytes().to_vec())
        } else if command.starts_with("echo ") {
            let text = &command[5..];
            (MessageType::Request, format!("echo").as_bytes().to_vec())
        } else {
            (MessageType::Request, command.as_bytes().to_vec())
        };
        
        // メッセージを作成して送信
        let request = IPCMessage::new(message_type.clone(), payload);
        let request_id = request.id;  // リクエストIDを保存
        
        debug!("送信: メッセージID={}", request_id);
        let start = Instant::now();
        send_message(&mut stream, &request)?;
        
        // レスポンスを受信
        match receive_message(&mut stream) {
            Ok(response) => {
                let elapsed = start.elapsed();
                let response_text = String::from_utf8_lossy(&response.payload);
                
                // 相関IDの確認
                if let Some(correlation_id) = response.correlation_id {
                    if correlation_id == request_id {
                        debug!("相関ID一致: リクエスト={}, レスポンス={}", request_id, response.id);
                    } else {
                        error!("相関ID不一致: 期待={}, 実際={}", request_id, correlation_id);
                    }
                }
                
                match response.message_type {
                    MessageType::Response => {
                        println!("応答: {} (レイテンシ: {:?})", response_text, elapsed);
                    }
                    MessageType::Error => {
                        error!("エラー: {}", response_text);
                    }
                    _ => {
                        println!("予期しない応答タイプ: {:?}", response.message_type);
                    }
                }
            }
            Err(e) => {
                error!("レスポンス受信エラー: {}", e);
            }
        }
    }
    
    Ok(())
}

/// メッセージを送信
fn send_message(stream: &mut UnixStream, message: &IPCMessage) -> Result<()> {
    let bytes = message.to_bytes()?;
    let size = bytes.len() as u32;
    
    // サイズを送信（4バイト）
    stream.write_all(&size.to_le_bytes())?;
    // メッセージ本体を送信
    stream.write_all(&bytes)?;
    stream.flush()?;
    
    debug!("メッセージ送信: {} bytes", bytes.len());
    Ok(())
}

/// メッセージを受信
fn receive_message(stream: &mut UnixStream) -> Result<IPCMessage> {
    // メッセージサイズを読み取り（4バイト）
    let mut size_buf = [0u8; 4];
    stream.read_exact(&mut size_buf)?;
    let message_size = u32::from_le_bytes(size_buf) as usize;
    
    // サイズチェック
    if message_size > IPCMessage::MAX_PAYLOAD_SIZE {
        return Err(anyhow::anyhow!("受信メッセージが大きすぎます: {} bytes", message_size));
    }
    
    // メッセージ本体を読み取り
    let mut message_buf = vec![0u8; message_size];
    stream.read_exact(&mut message_buf)?;
    
    // デシリアライズ
    let message = IPCMessage::from_bytes(&message_buf)?;
    debug!("メッセージ受信: {} bytes", message_size);
    
    Ok(message)
}

/// ストレステスト：複数のメッセージを連続送信
fn run_stress_test(stream: &mut UnixStream, count: usize) -> Result<()> {
    info!("ストレステスト開始: {}個のメッセージを送信", count);
    
    let start = Instant::now();
    let mut success_count = 0;
    let mut total_latency = Duration::ZERO;
    
    for i in 0..count {
        let request = IPCMessage::request(format!("stress test message {}", i).as_bytes().to_vec());
        
        let msg_start = Instant::now();
        
        match send_message(stream, &request) {
            Ok(_) => {
                match receive_message(stream) {
                    Ok(_response) => {
                        success_count += 1;
                        total_latency += msg_start.elapsed();
                    }
                    Err(e) => {
                        error!("メッセージ {} の受信失敗: {}", i, e);
                    }
                }
            }
            Err(e) => {
                error!("メッセージ {} の送信失敗: {}", i, e);
            }
        }
        
        // 進捗表示
        if (i + 1) % 100 == 0 {
            println!("進捗: {}/{}", i + 1, count);
        }
    }
    
    let total_time = start.elapsed();
    let avg_latency = if success_count > 0 {
        total_latency / success_count as u32
    } else {
        Duration::ZERO
    };
    
    println!("\n=== ストレステスト結果 ===");
    println!("送信メッセージ数: {}", count);
    println!("成功数: {}", success_count);
    println!("失敗数: {}", count - success_count);
    println!("総時間: {:?}", total_time);
    println!("平均レイテンシ: {:?}", avg_latency);
    println!("スループット: {:.2} msg/sec", success_count as f64 / total_time.as_secs_f64());
    
    Ok(())
}