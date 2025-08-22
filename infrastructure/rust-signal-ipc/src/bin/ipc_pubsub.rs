/// Pub/Subパターンの双方向通信実装
///
/// 複数のクライアントがトピックを購読し、
/// メッセージをブロードキャストできるシステム

use anyhow::{Context, Result};
use rust_signal_ipc::ipc::{IPCMessage, MessageType};
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::os::unix::net::{UnixStream, UnixListener};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tracing::{info, warn, error, debug};
use serde::{Serialize, Deserialize};

/// Pub/Sub用のメッセージ
#[derive(Serialize, Deserialize, Debug, Clone)]
struct PubSubMessage {
    topic: String,
    content: String,
    sender_id: usize,
}

/// クライアント接続情報
struct ClientConnection {
    id: usize,
    stream: UnixStream,
    subscriptions: HashSet<String>,
}

/// Pub/Subサーバーの状態
struct PubSubServer {
    clients: Arc<Mutex<HashMap<usize, Arc<Mutex<ClientConnection>>>>>,
    topics: Arc<Mutex<HashMap<String, HashSet<usize>>>>,
    next_client_id: Arc<Mutex<usize>>,
}

impl PubSubServer {
    fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            topics: Arc::new(Mutex::new(HashMap::new())),
            next_client_id: Arc::new(Mutex::new(1)),
        }
    }

    fn add_client(&self, stream: UnixStream) -> usize {
        let mut next_id = self.next_client_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;

        let client = Arc::new(Mutex::new(ClientConnection {
            id,
            stream,
            subscriptions: HashSet::new(),
        }));

        let mut clients = self.clients.lock().unwrap();
        clients.insert(id, client);

        info!("クライアント {} が接続", id);
        id
    }

    fn remove_client(&self, id: usize) {
        // トピックから削除
        let mut topics = self.topics.lock().unwrap();
        for subscribers in topics.values_mut() {
            subscribers.remove(&id);
        }

        // クライアントリストから削除
        let mut clients = self.clients.lock().unwrap();
        clients.remove(&id);

        info!("クライアント {} が切断", id);
    }

    fn subscribe(&self, client_id: usize, topic: &str) -> Result<()> {
        let mut topics = self.topics.lock().unwrap();
        topics.entry(topic.to_string())
            .or_insert_with(HashSet::new)
            .insert(client_id);

        let clients = self.clients.lock().unwrap();
        if let Some(client) = clients.get(&client_id) {
            let mut client = client.lock().unwrap();
            client.subscriptions.insert(topic.to_string());
        }

        info!("クライアント {} がトピック '{}' を購読", client_id, topic);
        Ok(())
    }

    fn unsubscribe(&self, client_id: usize, topic: &str) -> Result<()> {
        let mut topics = self.topics.lock().unwrap();
        if let Some(subscribers) = topics.get_mut(topic) {
            subscribers.remove(&client_id);
        }

        let clients = self.clients.lock().unwrap();
        if let Some(client) = clients.get(&client_id) {
            let mut client = client.lock().unwrap();
            client.subscriptions.remove(topic);
        }

        info!("クライアント {} がトピック '{}' の購読を解除", client_id, topic);
        Ok(())
    }

    fn publish(&self, topic: &str, message: PubSubMessage) -> Result<usize> {
        let topics = self.topics.lock().unwrap();
        let subscribers = match topics.get(topic) {
            Some(subs) => subs.clone(),
            None => {
                warn!("トピック '{}' に購読者なし", topic);
                return Ok(0);
            }
        };

        let clients = self.clients.lock().unwrap();
        let mut sent_count = 0;

        for &subscriber_id in &subscribers {
            // 送信者自身には送らない
            if subscriber_id == message.sender_id {
                continue;
            }

            if let Some(client) = clients.get(&subscriber_id) {
                let mut client = client.lock().unwrap();
                
                // メッセージをシリアライズして送信
                let payload = bincode::serialize(&message)?;
                let ipc_msg = IPCMessage::notification(payload);
                
                if let Err(e) = send_message(&mut client.stream, &ipc_msg) {
                    error!("クライアント {} への送信失敗: {}", subscriber_id, e);
                } else {
                    sent_count += 1;
                }
            }
        }

        debug!("トピック '{}' に {} 件のメッセージを配信", topic, sent_count);
        Ok(sent_count)
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let socket_path = "/tmp/rust_ipc_pubsub.sock";
    
    if Path::new(socket_path).exists() {
        std::fs::remove_file(socket_path)?;
    }
    
    let listener = UnixListener::bind(socket_path)?;
    info!("Pub/Subサーバーが起動: {}", socket_path);
    
    let server = Arc::new(PubSubServer::new());
    
    // 統計情報表示
    let stats_server = server.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(30));
            let clients = stats_server.clients.lock().unwrap();
            let topics = stats_server.topics.lock().unwrap();
            info!("統計: クライアント数={}, トピック数={}", 
                clients.len(), topics.len());
        }
    });
    
    // クライアント接続受付
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let client_id = server.add_client(stream.try_clone()?);
                let client_server = server.clone();
                
                thread::spawn(move || {
                    if let Err(e) = handle_pubsub_client(stream, client_id, client_server.clone()) {
                        error!("クライアント {} エラー: {}", client_id, e);
                    }
                    client_server.remove_client(client_id);
                });
            }
            Err(e) => {
                error!("接続受付エラー: {}", e);
            }
        }
    }
    
    Ok(())
}

fn handle_pubsub_client(mut stream: UnixStream, client_id: usize, server: Arc<PubSubServer>) -> Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(60)))?;
    
    loop {
        // コマンドを受信
        let mut size_buf = [0u8; 4];
        match stream.read_exact(&mut size_buf) {
            Ok(_) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                warn!("クライアント {} タイムアウト", client_id);
                break;
            }
            Err(e) => return Err(e.into()),
        }
        
        let message_size = u32::from_le_bytes(size_buf) as usize;
        let mut message_buf = vec![0u8; message_size];
        stream.read_exact(&mut message_buf)?;
        
        let request = IPCMessage::from_bytes(&message_buf)?;
        let request_id = request.id;  // リクエストIDを保存
        let command = String::from_utf8_lossy(&request.payload);
        
        // コマンド処理
        let response = if command.starts_with("SUB:") {
            let topic = &command[4..];
            server.subscribe(client_id, topic)?;
            format!("Subscribed to '{}'", topic)
        } else if command.starts_with("UNSUB:") {
            let topic = &command[6..];
            server.unsubscribe(client_id, topic)?;
            format!("Unsubscribed from '{}'", topic)
        } else if command.starts_with("PUB:") {
            let parts: Vec<&str> = command[4..].splitn(2, ':').collect();
            if parts.len() == 2 {
                let topic = parts[0];
                let content = parts[1];
                let msg = PubSubMessage {
                    topic: topic.to_string(),
                    content: content.to_string(),
                    sender_id: client_id,
                };
                let count = server.publish(topic, msg)?;
                format!("Published to {} subscribers", count)
            } else {
                "Invalid PUB format. Use PUB:topic:message".to_string()
            }
        } else if command == "LIST" {
            let topics = server.topics.lock().unwrap();
            let list: Vec<String> = topics.keys()
                .map(|t| format!("{} ({} subscribers)", t, topics[t].len()))
                .collect();
            format!("Topics: {}", list.join(", "))
        } else {
            format!("Unknown command: {}", command)
        };
        
        // レスポンス送信（リクエストIDを相関IDとして使用）
        let response_msg = IPCMessage::response(response.as_bytes().to_vec(), request_id);
        send_message(&mut stream, &response_msg)?;
    }
    
    Ok(())
}

fn send_message(stream: &mut UnixStream, message: &IPCMessage) -> Result<()> {
    let bytes = message.to_bytes()?;
    let size = bytes.len() as u32;
    stream.write_all(&size.to_le_bytes())?;
    stream.write_all(&bytes)?;
    stream.flush()?;
    Ok(())
}