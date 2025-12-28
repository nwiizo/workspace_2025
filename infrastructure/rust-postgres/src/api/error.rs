//! エラーハンドリング
//!
//! `IntoResponse` を実装したカスタムエラー型を提供します。

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// アプリケーションエラー
#[derive(Debug)]
pub enum AppError {
    /// リソースが見つからない
    NotFound(String),
    /// リソースの競合（重複など）
    Conflict(String),
    /// 無効な入力
    InvalidInput(String),
    /// 認証が必要
    Unauthorized(String),
    /// アクセス権限がない
    Forbidden(String),
    /// データベースエラー
    DatabaseError(String),
    /// 内部エラー
    InternalError(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            AppError::InvalidInput(msg) => write!(f, "Invalid Input: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AppError::DatabaseError(msg) => write!(f, "Database Error: {}", msg),
            AppError::InternalError(msg) => write!(f, "Internal Error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.clone()),
            AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, "INVALID_INPUT", msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN", msg.clone()),
            AppError::DatabaseError(msg) => {
                // 内部エラーはログに出力し、クライアントには詳細を隠す
                eprintln!("Database error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "A database error occurred".to_string(),
                )
            }
            AppError::InternalError(msg) => {
                eprintln!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "An internal error occurred".to_string(),
                )
            }
        };

        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": message,
            }
        }));

        (status, body).into_response()
    }
}

// tokio-postgres エラーからの変換
impl From<tokio_postgres::Error> for AppError {
    fn from(err: tokio_postgres::Error) -> Self {
        // PostgreSQL エラーコードによる分類
        if let Some(db_err) = err.as_db_error() {
            match db_err.code().code() {
                "23505" => {
                    return AppError::Conflict("Resource already exists".to_string());
                }
                "23503" => {
                    return AppError::InvalidInput("Referenced resource not found".to_string());
                }
                _ => {}
            }
        }
        AppError::DatabaseError(err.to_string())
    }
}

// deadpool-postgres エラーからの変換
impl From<deadpool_postgres::PoolError> for AppError {
    fn from(err: deadpool_postgres::PoolError) -> Self {
        AppError::DatabaseError(format!("Pool error: {}", err))
    }
}
