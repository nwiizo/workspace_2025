//! アプリケーション状態
//!
//! Axum の Router に渡す共有状態を定義します。

use deadpool_postgres::Pool;

use super::auth::OAuthConfig;

/// アプリケーション状態
#[derive(Clone)]
pub struct AppState {
    /// データベース接続プール
    pub db: Pool,
    /// OAuth 設定（オプション）
    pub oauth_config: Option<OAuthConfig>,
    /// Cookie 署名用の秘密鍵
    pub cookie_key: axum_extra::extract::cookie::Key,
}

impl AppState {
    /// 新しい AppState を作成
    pub fn new(db: Pool) -> Self {
        Self {
            db,
            oauth_config: None,
            cookie_key: axum_extra::extract::cookie::Key::generate(),
        }
    }

    /// OAuth 設定を追加
    pub fn with_oauth(mut self, config: OAuthConfig) -> Self {
        self.oauth_config = Some(config);
        self
    }

    /// Cookie 署名用の鍵を設定
    pub fn with_cookie_key(mut self, key: axum_extra::extract::cookie::Key) -> Self {
        self.cookie_key = key;
        self
    }
}

