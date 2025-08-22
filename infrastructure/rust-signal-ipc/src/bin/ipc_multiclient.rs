/// 複数クライアントシミュレーター
///
/// 複数のクライアントプロセスを同時に起動して
/// サーバーとの通信をテストする

use anyhow::{Context, Result};
use rust_signal_ipc::ipc::{IPCMessage, MessageType};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tracing::{info, error, debug};

/// クライアントワーカーの統計情報
#[derive(Debug, Clone, Default)]
struct ClientStats {
    messages_sent: usize,
    messages_received: usize,
    errors: usize,
    total_latency_ms: u128,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let args: Vec<String> = std::env::args().collect();
    let num_clients = if args.len() > 1 {
        args[1].parse::<usize>().unwrap_or(5)
    } else {
        5
    };
    
    let messages_per_client = if args.len() > 2 {
        args[2].parse::<usize>().unwrap_or(10)
    } else {
        10
    };
    
    info!("複数クライアントシミュレーター起動");
    info!("クライアント数: {}", num_clients);
    info!("メッセージ数/クライアント: {}", messages_per_client);
    
    let all_stats = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];
    
    let start_time = Instant::now();
    
    // 複数のクライアントを起動
    for client_id in 0..num_clients {
        let stats_clone = all_stats.clone();
        
        let handle = thread::spawn(move || {
            let stats = match run_client_worker(client_id, messages_per_client) {
                Ok(s) => s,
                Err(e) => {
                    error!("クライアント {} エラー: {}", client_id, e);
                    ClientStats::default()
                }
            };
            
            let mut all = stats_clone.lock().unwrap();
            all.push(stats);
        });
        
        handles.push(handle);
        
        // クライアント起動を少しずらす
        thread::sleep(Duration::from_millis(50));
    }
    
    // すべてのクライアントの完了を待つ
    for handle in handles {
        handle.join().unwrap();
    }
    
    let total_time = start_time.elapsed();
    
    // 統計情報の集計と表示
    let all_stats = all_stats.lock().unwrap();
    print_statistics(&all_stats, total_time);
    
    Ok(())
}

/// 個別のクライアントワーカー
fn run_client_worker(worker_id: usize, message_count: usize) -> Result<ClientStats> {
    debug!("ワーカー {} 起動", worker_id);
    
    // サーバーに接続
    let mut stream = UnixStream::connect("/tmp/rust_ipc_structured.sock")
        .context(format!("ワーカー {} の接続失敗", worker_id))?;
    
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    
    let mut stats = ClientStats::default();
    
    // メッセージ送受信
    for i in 0..message_count {
        let command = match i % 5 {
            0 => "ping".to_string(),
            1 => "time".to_string(),
            2 => "info".to_string(),
            3 => format!("calc:10 + {}", i),
            _ => format!("echo worker-{}-message-{}", worker_id, i),
        };
        
        let request = IPCMessage::request(command.as_bytes().to_vec());
        
        let start = Instant::now();
        
        // 送信
        match send_message(&mut stream, &request) {
            Ok(_) => {
                stats.messages_sent += 1;
                
                // 受信
                match receive_message(&mut stream) {
                    Ok(response) => {
                        stats.messages_received += 1;
                        stats.total_latency_ms += start.elapsed().as_millis();
                        
                        if response.message_type == MessageType::Error {
                            let error_msg = String::from_utf8_lossy(&response.payload);
                            error!("ワーカー {} エラー応答: {}", worker_id, error_msg);
                            stats.errors += 1;
                        }
                    }
                    Err(e) => {
                        error!("ワーカー {} 受信エラー: {}", worker_id, e);
                        stats.errors += 1;
                    }
                }
            }
            Err(e) => {
                error!("ワーカー {} 送信エラー: {}", worker_id, e);
                stats.errors += 1;
            }
        }
        
        // 少し待機
        thread::sleep(Duration::from_millis(10));
    }
    
    // ハートビート送信
    let heartbeat = IPCMessage::new(MessageType::Heartbeat, vec![]);
    let _ = send_message(&mut stream, &heartbeat);
    
    debug!("ワーカー {} 完了", worker_id);
    Ok(stats)
}

/// メッセージ送信
fn send_message(stream: &mut UnixStream, message: &IPCMessage) -> Result<()> {
    let bytes = message.to_bytes()?;
    let size = bytes.len() as u32;
    
    stream.write_all(&size.to_le_bytes())?;
    stream.write_all(&bytes)?;
    stream.flush()?;
    
    Ok(())
}

/// メッセージ受信
fn receive_message(stream: &mut UnixStream) -> Result<IPCMessage> {
    let mut size_buf = [0u8; 4];
    stream.read_exact(&mut size_buf)?;
    let message_size = u32::from_le_bytes(size_buf) as usize;
    
    if message_size > IPCMessage::MAX_PAYLOAD_SIZE {
        return Err(anyhow::anyhow!("メッセージサイズ超過: {} bytes", message_size));
    }
    
    let mut message_buf = vec![0u8; message_size];
    stream.read_exact(&mut message_buf)?;
    
    let message = IPCMessage::from_bytes(&message_buf)?;
    Ok(message)
}

/// 統計情報の表示
fn print_statistics(all_stats: &[ClientStats], total_time: Duration) {
    let mut total_sent = 0;
    let mut total_received = 0;
    let mut total_errors = 0;
    let mut total_latency = 0u128;
    
    for (i, stats) in all_stats.iter().enumerate() {
        println!("クライアント {}: 送信={}, 受信={}, エラー={}, 平均レイテンシ={:.2}ms",
            i,
            stats.messages_sent,
            stats.messages_received,
            stats.errors,
            if stats.messages_received > 0 {
                stats.total_latency_ms as f64 / stats.messages_received as f64
            } else {
                0.0
            }
        );
        
        total_sent += stats.messages_sent;
        total_received += stats.messages_received;
        total_errors += stats.errors;
        total_latency += stats.total_latency_ms;
    }
    
    println!("\n=== 総合統計 ===");
    println!("クライアント数: {}", all_stats.len());
    println!("総送信メッセージ数: {}", total_sent);
    println!("総受信メッセージ数: {}", total_received);
    println!("総エラー数: {}", total_errors);
    println!("成功率: {:.2}%", 
        if total_sent > 0 {
            (total_received as f64 / total_sent as f64) * 100.0
        } else {
            0.0
        }
    );
    println!("平均レイテンシ: {:.2}ms",
        if total_received > 0 {
            total_latency as f64 / total_received as f64
        } else {
            0.0
        }
    );
    println!("総実行時間: {:?}", total_time);
    println!("スループット: {:.2} msg/sec",
        total_received as f64 / total_time.as_secs_f64()
    );
}