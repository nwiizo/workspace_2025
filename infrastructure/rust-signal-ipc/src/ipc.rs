/// プロセス間通信（IPC）のメッセージ構造定義
///
/// UUIDとcorrelation_idを含む実践的なIPCプロトコル

use serde::{Serialize, Deserialize};
use uuid::Uuid;
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

/// IPCメッセージ（UUID付き）
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IPCMessage {
    /// メッセージの一意識別子
    pub id: Uuid,
    /// メッセージの種類
    pub message_type: MessageType,
    /// ペイロード（実際のデータ）
    pub payload: Vec<u8>,
    /// タイムスタンプ
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 相関ID（リクエスト・レスポンスの対応付け）
    pub correlation_id: Option<Uuid>,
}

impl IPCMessage {
    /// 最大ペイロードサイズ（1MB）
    pub const MAX_PAYLOAD_SIZE: usize = 1024 * 1024;
    
    /// 新しいメッセージを作成
    pub fn new(message_type: MessageType, payload: Vec<u8>) -> Self {
        Self {
            id: Uuid::new_v4(),
            message_type,
            payload,
            timestamp: chrono::Utc::now(),
            correlation_id: None,
        }
    }
    
    /// リクエストメッセージを作成
    pub fn request(payload: Vec<u8>) -> Self {
        Self::new(MessageType::Request, payload)
    }
    
    /// レスポンスメッセージを作成（相関ID付き）
    pub fn response(payload: Vec<u8>, correlation_id: Uuid) -> Self {
        let mut msg = Self::new(MessageType::Response, payload);
        msg.correlation_id = Some(correlation_id);
        msg
    }
    
    /// 通知メッセージを作成
    pub fn notification(payload: Vec<u8>) -> Self {
        Self::new(MessageType::Notification, payload)
    }
    
    /// ハートビートメッセージを作成
    pub fn heartbeat() -> Self {
        Self::new(MessageType::Heartbeat, vec![])
    }
    
    /// エラーメッセージを作成
    pub fn error(error_msg: String, correlation_id: Option<Uuid>) -> Self {
        let mut msg = Self::new(MessageType::Error, error_msg.into_bytes());
        msg.correlation_id = correlation_id;
        msg
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
        assert!(msg.id != Uuid::nil());
        assert!(msg.correlation_id.is_none());
    }
    
    #[test]
    fn test_response_with_correlation() {
        let request = IPCMessage::request(b"request".to_vec());
        let response = IPCMessage::response(b"response".to_vec(), request.id);
        
        assert_eq!(response.message_type, MessageType::Response);
        assert_eq!(response.correlation_id, Some(request.id));
    }
    
    #[test]
    fn test_serialization() {
        let original = IPCMessage::request(b"test payload".to_vec());
        let bytes = original.to_bytes().unwrap();
        let restored = IPCMessage::from_bytes(&bytes).unwrap();
        
        assert_eq!(original.id, restored.id);
        assert_eq!(original.message_type, restored.message_type);
        assert_eq!(original.payload, restored.payload);
        assert_eq!(original.correlation_id, restored.correlation_id);
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
    
    #[test]
    fn test_error_message() {
        let request_id = Uuid::new_v4();
        let error_msg = IPCMessage::error("Something went wrong".to_string(), Some(request_id));
        
        assert_eq!(error_msg.message_type, MessageType::Error);
        assert_eq!(error_msg.correlation_id, Some(request_id));
        assert_eq!(error_msg.payload, b"Something went wrong");
    }
}