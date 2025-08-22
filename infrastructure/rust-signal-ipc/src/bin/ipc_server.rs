/// 実践的なIPCサーバー実装
///
/// 構造化されたメッセージプロトコルを使用して
/// 複数のクライアントと通信可能なサーバー

use anyhow::{Context, Result};
use rust_signal_ipc::ipc::{IPCMessage, MessageType};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::os::unix::net::{UnixStream, UnixListener};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tracing::{info, warn, error, debug};

/// クライアント情報
#[derive(Debug, Clone)]
struct ClientInfo {
    id: usize,
    connected_at: Instant,
    message_count: usize,
}

/// サーバーの状態管理
struct ServerState {
    clients: Arc<Mutex<HashMap<usize, ClientInfo>>>,
    next_client_id: Arc<Mutex<usize>>,
    total_messages: Arc<Mutex<usize>>,
}

impl ServerState {
    fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            next_client_id: Arc::new(Mutex::new(1)),
            total_messages: Arc::new(Mutex::new(0)),
        }
    }

    fn add_client(&self) -> usize {
        let mut next_id = self.next_client_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;

        let mut clients = self.clients.lock().unwrap();
        clients.insert(id, ClientInfo {
            id,
            connected_at: Instant::now(),
            message_count: 0,
        });

        id
    }

    fn remove_client(&self, id: usize) {
        let mut clients = self.clients.lock().unwrap();
        if let Some(client) = clients.remove(&id) {
            let duration = client.connected_at.elapsed();
            info!(
                "クライアント {} が切断: 接続時間 {:?}, 処理メッセージ数 {}",
                id, duration, client.message_count
            );
        }
    }

    fn increment_message_count(&self, client_id: usize) {
        let mut clients = self.clients.lock().unwrap();
        if let Some(client) = clients.get_mut(&client_id) {
            client.message_count += 1;
        }
        
        let mut total = self.total_messages.lock().unwrap();
        *total += 1;
    }

    fn get_stats(&self) -> (usize, usize) {
        let clients = self.clients.lock().unwrap();
        let total = self.total_messages.lock().unwrap();
        (clients.len(), *total)
    }
}

fn main() -> Result<()> {
    // ログの初期化
    tracing_subscriber::fmt::init();
    
    let socket_path = "/tmp/rust_ipc_structured.sock";
    
    // 既存のソケットファイルを削除
    if Path::new(socket_path).exists() {
        std::fs::remove_file(socket_path)
            .context("既存のソケットファイルの削除に失敗")?;
    }
    
    let listener = UnixListener::bind(socket_path)
        .context("Unix Domain Socketのバインドに失敗")?;
    
    info!("IPCサーバーが起動しました: {}", socket_path);
    info!("構造化メッセージプロトコルを使用");
    
    let state = Arc::new(ServerState::new());
    
    // 統計情報表示スレッド
    let stats_state = state.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(10));
            let (clients, messages) = stats_state.get_stats();
            info!("統計情報: 接続中のクライアント数={}, 総メッセージ数={}", clients, messages);
        }
    });
    
    // メイン受付ループ
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let client_id = state.add_client();
                info!("新しいクライアントが接続: ID={}", client_id);
                
                let client_state = state.clone();
                thread::spawn(move || {
                    if let Err(e) = handle_client(stream, client_id, &client_state) {
                        error!("クライアント {} の処理エラー: {}", client_id, e);
                    }
                    client_state.remove_client(client_id);
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
fn handle_client(mut stream: UnixStream, client_id: usize, state: &ServerState) -> Result<()> {
    // タイムアウト設定
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    
    loop {
        // メッセージサイズを読み取り（4バイト）
        let mut size_buf = [0u8; 4];
        match stream.read_exact(&mut size_buf) {
            Ok(_) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                debug!("クライアント {} が正常に切断", client_id);
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                warn!("クライアント {} からの読み取りタイムアウト", client_id);
                break;
            }
            Err(e) => {
                error!("クライアント {} からの読み取りエラー: {}", client_id, e);
                break;
            }
        }
        
        let message_size = u32::from_le_bytes(size_buf) as usize;
        
        // サイズチェック
        if message_size > IPCMessage::MAX_PAYLOAD_SIZE {
            error!("クライアント {} から過大なメッセージ: {} bytes", client_id, message_size);
            send_error_response(&mut stream, "Message too large")?;
            continue;
        }
        
        // メッセージ本体を読み取り
        let mut message_buf = vec![0u8; message_size];
        stream.read_exact(&mut message_buf)?;
        
        // デシリアライズ
        match IPCMessage::from_bytes(&message_buf) {
            Ok(request) => {
                debug!("クライアント {} からメッセージ受信: {:?}", client_id, request.message_type);
                state.increment_message_count(client_id);
                
                // リクエストを処理してレスポンスを生成
                let response = process_request(request, client_id)?;
                
                // レスポンスを送信
                send_message(&mut stream, &response)?;
            }
            Err(e) => {
                error!("クライアント {} からの無効なメッセージ: {}", client_id, e);
                send_error_response(&mut stream, "Invalid message format")?;
            }
        }
    }
    
    Ok(())
}

/// リクエストを処理してレスポンスを生成
fn process_request(request: IPCMessage, client_id: usize) -> Result<IPCMessage> {
    let payload_str = String::from_utf8_lossy(&request.payload);
    let request_id = request.id;  // リクエストIDを保存（相関IDとして使用）
    
    let response_payload = match request.message_type {
        MessageType::Request => {
            // コマンド処理
            match payload_str.as_ref() {
                "ping" => b"pong".to_vec(),
                "time" => {
                    let time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%3f");
                    format!("{}", time).into_bytes()
                }
                "info" => {
                    format!("Client ID: {}, Server PID: {}", client_id, std::process::id()).into_bytes()
                }
                "echo" => request.payload.clone(),
                cmd if cmd.starts_with("calc:") => {
                    // 簡単な計算機能
                    let expr = cmd.strip_prefix("calc:").unwrap().trim();
                    match evaluate_simple_math(expr) {
                        Ok(result) => format!("{}", result).into_bytes(),
                        Err(e) => format!("Error: {}", e).into_bytes(),
                    }
                }
                _ => format!("Unknown command: {}", payload_str).into_bytes(),
            }
        }
        MessageType::Notification => {
            info!("クライアント {} から通知: {}", client_id, payload_str);
            b"Notification received".to_vec()
        }
        MessageType::Heartbeat => {
            debug!("クライアント {} からハートビート", client_id);
            b"alive".to_vec()
        }
        _ => b"Unsupported message type".to_vec(),
    };
    
    // リクエストIDを相関IDとして設定してレスポンスを作成
    Ok(IPCMessage::response(response_payload, request_id))
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
    
    Ok(())
}

/// エラーレスポンスを送信
fn send_error_response(stream: &mut UnixStream, error_msg: &str) -> Result<()> {
    let response = IPCMessage::error(error_msg.to_string(), None);
    send_message(stream, &response)
}

/// 簡単な数式を評価（デモ用）
fn evaluate_simple_math(expr: &str) -> Result<f64> {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if parts.len() != 3 {
        return Err(anyhow::anyhow!("式は 'a op b' の形式で入力してください"));
    }
    
    let a: f64 = parts[0].parse().context("最初の数値が無効")?;
    let b: f64 = parts[2].parse().context("2番目の数値が無効")?;
    
    match parts[1] {
        "+" => Ok(a + b),
        "-" => Ok(a - b),
        "*" => Ok(a * b),
        "/" => {
            if b == 0.0 {
                Err(anyhow::anyhow!("ゼロ除算"))
            } else {
                Ok(a / b)
            }
        }
        op => Err(anyhow::anyhow!("未対応の演算子: {}", op)),
    }
}