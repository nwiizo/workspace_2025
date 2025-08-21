/// エラー処理モジュール
///
/// IPCおよびシグナル処理で発生するエラーを定義

use thiserror::Error;

/// IPCおよびシグナル処理で発生するエラーの定義
#[derive(Error, Debug)]
pub enum IPCError {
    /// I/O操作でのエラー
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    
    /// シリアライゼーション/デシリアライゼーションエラー
    #[error("serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    
    /// プロトコルエラー
    #[error("protocol error: {0}")]
    Protocol(String),
    
    /// 接続エラー
    #[error("connection error: {0}")]
    Connection(String),
    
    /// その他のエラー
    #[error("{0}")]
    Other(String),
}

impl IPCError {
    /// カスタムプロトコルエラーを作成
    pub fn protocol<S: Into<String>>(msg: S) -> Self {
        Self::Protocol(msg.into())
    }
    
    /// カスタム接続エラーを作成
    pub fn connection<S: Into<String>>(msg: S) -> Self {
        Self::Connection(msg.into())
    }
    
    /// エラーが再試行可能かどうかを判定
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Io(_) | Self::Connection(_)
        )
    }
}

/// 結果型のエイリアス
pub type Result<T> = std::result::Result<T, IPCError>;