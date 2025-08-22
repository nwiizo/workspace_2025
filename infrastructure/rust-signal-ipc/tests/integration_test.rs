/// 統合テスト
///
/// IPCメッセージとエラー処理の統合テスト

use rust_signal_ipc::ipc::{IPCMessage, MessageType};
use rust_signal_ipc::errors::IPCError;
use uuid::Uuid;

#[test]
fn test_message_round_trip() {
    let original = IPCMessage::request(b"test payload".to_vec());
    let bytes = original.to_bytes().unwrap();
    let restored = IPCMessage::from_bytes(&bytes).unwrap();
    
    assert_eq!(original.payload, restored.payload);
    assert_eq!(original.message_type, restored.message_type);
}

#[test]
fn test_message_types() {
    let request = IPCMessage::request(b"req".to_vec());
    assert_eq!(request.message_type, MessageType::Request);
    
    let request_id = Uuid::new_v4();
    let response = IPCMessage::response(b"res".to_vec(), request_id);
    assert_eq!(response.message_type, MessageType::Response);
    assert_eq!(response.correlation_id, Some(request_id));
    
    let notification = IPCMessage::notification(b"notif".to_vec());
    assert_eq!(notification.message_type, MessageType::Notification);
}

#[test]
fn test_error_types() {
    // Protocol error (fatal, not retryable)
    let err = IPCError::protocol("invalid version");
    assert!(!err.is_retryable());
    assert!(err.is_fatal());
    
    // IO error - ConnectionRefused (retryable)
    let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "test");
    let err = IPCError::from(io_err);
    assert!(err.is_retryable());
    assert!(!err.is_fatal());
    
    // IO error - PermissionDenied (not retryable)
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test");
    let err = IPCError::from(io_err);
    assert!(!err.is_retryable());
    assert!(!err.is_fatal());
    
    // Connection error (retryable)
    let err = IPCError::connection("connection lost");
    assert!(err.is_retryable());
    assert!(!err.is_fatal());
}

#[test]
fn test_payload_size_limit() {
    // 正常なサイズ
    let normal_payload = vec![0u8; 1024];
    let msg = IPCMessage::request(normal_payload);
    assert!(msg.validate().is_ok());
    
    // 最大サイズぎりぎり
    let max_payload = vec![0u8; IPCMessage::MAX_PAYLOAD_SIZE];
    let msg = IPCMessage::request(max_payload);
    assert!(msg.validate().is_ok());
    
    // サイズ超過
    let oversized_payload = vec![0u8; IPCMessage::MAX_PAYLOAD_SIZE + 1];
    let msg = IPCMessage::request(oversized_payload);
    assert!(msg.validate().is_err());
}

#[test]
fn test_timestamp_validation() {
    use chrono::{Utc, Duration};
    
    // 現在のタイムスタンプ（正常）
    let msg = IPCMessage::request(b"test".to_vec());
    assert!(msg.validate().is_ok());
    
    // 古すぎるタイムスタンプ
    let mut old_msg = IPCMessage::request(b"test".to_vec());
    old_msg.timestamp = Utc::now() - Duration::hours(2);
    assert!(old_msg.validate().is_err());
    
    // 未来のタイムスタンプ（許容範囲内）
    let mut future_msg = IPCMessage::request(b"test".to_vec());
    future_msg.timestamp = Utc::now() + Duration::minutes(3);
    assert!(future_msg.validate().is_ok());
    
    // 未来すぎるタイムスタンプ
    let mut far_future_msg = IPCMessage::request(b"test".to_vec());
    far_future_msg.timestamp = Utc::now() + Duration::minutes(10);
    assert!(far_future_msg.validate().is_err());
}