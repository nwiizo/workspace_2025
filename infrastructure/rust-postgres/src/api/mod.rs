//! Axum Web API モジュール
//!
//! PostgreSQL の高度な機能を活用した REST API を提供します。
//!
//! ## エンドポイント
//!
//! - `GET /api/products/search` - 商品検索（全文検索 + JSONB）
//! - `GET /api/products/{id}` - 商品詳細
//! - `GET /api/products/{id}/similar` - 類似商品（pgvector）
//! - `GET /api/products/categories` - カテゴリ一覧
//! - `GET /api/metrics` - メトリクス取得（TimescaleDB）
//! - `GET /api/sensors` - センサー一覧
//! - `GET /api/stores` - 店舗一覧
//! - `GET /api/stores/nearby` - 近くの店舗検索（PostGIS）
//!
//! ## 認証エンドポイント
//!
//! - `GET /api/auth/google/login` - Google ログイン開始
//! - `GET /api/auth/google/callback` - Google コールバック
//! - `POST /api/auth/logout` - ログアウト
//! - `GET /api/auth/status` - 認証状態確認
//! - `GET /api/auth/me` - 現在のユーザー情報

pub mod auth;
pub mod error;
pub mod handlers;
pub mod routes;
pub mod state;

pub use auth::{OAuthConfig, AuthUser, get_user_from_session, require_auth};
pub use error::AppError;
pub use routes::create_router;
pub use state::AppState;
