/// プロセス間通信（IPC）のメッセージ構造定義
///
/// シンプルなIPCプロトコルの実装

use serde::{Serialize, Deserialize};
use crate::errors::{IPCError, Result};

/// IPCメッセージの種類
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum MessageType {
    Request,
    Response,
    Notification,
    Heartbeat,
    Error,
}

/// シンプルなIPCメッセージ
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IPCMessage {
    /// メッセージの種類
    pub message_type: MessageType,
    /// ペイロード（実際のデータ）
    pub payload: Vec<u8>,
    /// タイムスタンプ
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl IPCMessage {
    /// 最大ペイロードサイズ（1MB）
    pub const MAX_PAYLOAD_SIZE: usize = 1024 * 1024;
    
    /// 新しいメッセージを作成
    pub fn new(message_type: MessageType, payload: Vec<u8>) -> Self {
        Self {
            message_type,
            payload,
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// リクエストメッセージを作成
    pub fn request(payload: Vec<u8>) -> Self {
        Self::new(MessageType::Request, payload)
    }
    
    /// レスポンスメッセージを作成
    pub fn response(payload: Vec<u8>) -> Self {
        Self::new(MessageType::Response, payload)
    }
    
    /// 通知メッセージを作成
    pub fn notification(payload: Vec<u8>) -> Self {
        Self::new(MessageType::Notification, payload)
    }
    
    /// メッセージの検証
    pub fn validate(&self) -> Result<()> {
        // ペイロードサイズチェック
        if self.payload.len() > Self::MAX_PAYLOAD_SIZE {
            return Err(IPCError::protocol(
                format!("payload too large: {} bytes", self.payload.len())
            ));
        }
        
        // タイムスタンプの妥当性チェック
        let now = chrono::Utc::now();
        let time_diff = now.signed_duration_since(self.timestamp);
        
        // 1時間以上古いメッセージは拒否
        if time_diff.num_hours() > 1 {
            return Err(IPCError::protocol("message too old"));
        }
        
        // 5分以上未来のメッセージは拒否
        if time_diff.num_minutes() < -5 {
            return Err(IPCError::protocol("message timestamp is in the future"));
        }
        
        Ok(())
    }
    
    /// バイナリへのシリアライズ
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(Into::into)
    }
    
    /// バイナリからのデシリアライズ
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let msg: Self = bincode::deserialize(bytes)?;
        msg.validate()?;
        Ok(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_creation() {
        let msg = IPCMessage::request(b"test".to_vec());
        assert_eq!(msg.message_type, MessageType::Request);
        assert_eq!(msg.payload, b"test");
    }
    
    #[test]
    fn test_serialization() {
        let original = IPCMessage::request(b"test payload".to_vec());
        let bytes = original.to_bytes().unwrap();
        let restored = IPCMessage::from_bytes(&bytes).unwrap();
        
        assert_eq!(original.message_type, restored.message_type);
        assert_eq!(original.payload, restored.payload);
    }
    
    #[test]
    fn test_validation() {
        // 正常なメッセージ
        let msg = IPCMessage::request(b"test".to_vec());
        assert!(msg.validate().is_ok());
        
        // ペイロードが大きすぎる
        let large_payload = vec![0u8; IPCMessage::MAX_PAYLOAD_SIZE + 1];
        let msg = IPCMessage::request(large_payload);
        assert!(msg.validate().is_err());
    }
}