//! Google OAuth 認証モジュール
//!
//! PostgreSQL をセッションストアとして使用した Google OAuth 認証を実装します。

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Json,
};
use axum_extra::extract::cookie::{Cookie, PrivateCookieJar, SameSite};
use oauth2::{
    AuthUrl, ClientId, ClientSecret, CsrfToken,
    PkceCodeChallenge, RedirectUrl, Scope, TokenUrl,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{AppError, AppState};

/// Cookie 名
const SESSION_COOKIE_NAME: &str = "session";
/// セッション有効期間（秒）
const SESSION_DURATION_SECS: i64 = 7 * 24 * 60 * 60; // 7日

/// Google OAuth エンドポイント
const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v3/userinfo";

/// OAuth 設定
#[derive(Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
    pub frontend_url: String,
}

impl OAuthConfig {
    /// 環境変数から設定を読み込む
    pub fn from_env() -> Result<Self, std::env::VarError> {
        Ok(Self {
            client_id: std::env::var("GOOGLE_CLIENT_ID")?,
            client_secret: std::env::var("GOOGLE_CLIENT_SECRET")?,
            redirect_url: std::env::var("OAUTH_REDIRECT_URL")
                .unwrap_or_else(|_| "http://localhost:3001/api/auth/google/callback".to_string()),
            frontend_url: std::env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
        })
    }
}

/// Google ログイン開始のクエリパラメータ
#[derive(Debug, Deserialize)]
pub struct LoginQuery {
    /// ログイン後のリダイレクト先
    pub redirect_to: Option<String>,
}

/// Google コールバックのクエリパラメータ
#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    pub state: String,
}

/// Google ユーザー情報
#[derive(Debug, Deserialize)]
pub struct GoogleUserInfo {
    pub sub: String,        // Google ID
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
    pub locale: Option<String>,
}

/// ログイン中のユーザー情報
#[derive(Debug, Clone, Serialize)]
pub struct AuthUser {
    pub id: i32,
    pub email: String,
    pub name: String,
    pub picture_url: Option<String>,
}

/// 認証状態レスポンス
#[derive(Debug, Serialize)]
pub struct AuthStatus {
    pub authenticated: bool,
    pub user: Option<AuthUser>,
}

/// セッショントークンを生成
fn generate_session_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    hex::encode(bytes)
}

/// トークンのハッシュを計算
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Google ログイン開始
///
/// CSRF 対策の state と PKCE を生成し、Google の認可エンドポイントにリダイレクトします。
pub async fn google_login(
    State(state): State<AppState>,
    Query(query): Query<LoginQuery>,
) -> Result<Response, AppError> {
    let oauth_config = state.oauth_config.as_ref().ok_or_else(|| {
        AppError::InternalError("OAuth not configured".to_string())
    })?;

    // OAuth2 クライアントを作成（型推論に任せる）
    let client = oauth2::basic::BasicClient::new(ClientId::new(oauth_config.client_id.clone()))
        .set_client_secret(ClientSecret::new(oauth_config.client_secret.clone()))
        .set_auth_uri(
            AuthUrl::new(GOOGLE_AUTH_URL.to_string())
                .map_err(|e| AppError::InternalError(format!("Invalid auth URL: {}", e)))?,
        )
        .set_token_uri(
            TokenUrl::new(GOOGLE_TOKEN_URL.to_string())
                .map_err(|e| AppError::InternalError(format!("Invalid token URL: {}", e)))?,
        )
        .set_redirect_uri(
            RedirectUrl::new(oauth_config.redirect_url.clone())
                .map_err(|e| AppError::InternalError(format!("Invalid redirect URL: {}", e)))?,
        );

    // PKCE チャレンジを生成
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // state を生成
    let (auth_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    // state と PKCE verifier を PostgreSQL に保存
    let db_client = state.db.get().await?;
    let redirect_url = query.redirect_to.unwrap_or_else(|| "/".to_string());

    db_client
        .execute(
            r#"
            INSERT INTO auth.oauth_states (state, pkce_verifier, redirect_url)
            VALUES ($1, $2, $3)
            "#,
            &[
                &csrf_state.secret(),
                &pkce_verifier.secret(),
                &redirect_url,
            ],
        )
        .await?;

    Ok(Redirect::temporary(auth_url.as_str()).into_response())
}

/// Google コールバック
///
/// Google からの認可コードを受け取り、アクセストークンを取得してユーザー情報を取得します。
pub async fn google_callback(
    State(state): State<AppState>,
    Query(query): Query<CallbackQuery>,
    jar: PrivateCookieJar,
) -> Result<(PrivateCookieJar, Response), AppError> {
    let oauth_config = state.oauth_config.as_ref().ok_or_else(|| {
        AppError::InternalError("OAuth not configured".to_string())
    })?;

    let db_client = state.db.get().await?;

    // state を検証し、PKCE verifier を取得
    let state_row = db_client
        .query_opt(
            r#"
            DELETE FROM auth.oauth_states
            WHERE state = $1 AND expires_at > NOW()
            RETURNING pkce_verifier, redirect_url
            "#,
            &[&query.state],
        )
        .await?
        .ok_or_else(|| AppError::InvalidInput("Invalid or expired state".to_string()))?;

    let pkce_verifier: String = state_row.get("pkce_verifier");
    let redirect_url: Option<String> = state_row.get("redirect_url");

    // トークンを取得（reqwest を直接使用）
    let http_client = reqwest::Client::new();

    let token_response = http_client
        .post(GOOGLE_TOKEN_URL)
        .form(&[
            ("code", query.code.as_str()),
            ("client_id", oauth_config.client_id.as_str()),
            ("client_secret", oauth_config.client_secret.as_str()),
            ("redirect_uri", oauth_config.redirect_url.as_str()),
            ("grant_type", "authorization_code"),
            ("code_verifier", pkce_verifier.as_str()),
        ])
        .send()
        .await
        .map_err(|e| AppError::InternalError(format!("Token request failed: {}", e)))?;

    if !token_response.status().is_success() {
        let error_text = token_response.text().await.unwrap_or_default();
        return Err(AppError::InternalError(format!(
            "Token exchange failed: {}",
            error_text
        )));
    }

    #[derive(Deserialize)]
    struct TokenResponse {
        access_token: String,
    }

    let token_data: TokenResponse = token_response
        .json()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to parse token response: {}", e)))?;

    // Google ユーザー情報を取得
    let user_info: GoogleUserInfo = http_client
        .get(GOOGLE_USERINFO_URL)
        .bearer_auth(&token_data.access_token)
        .send()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to get user info: {}", e)))?
        .json()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to parse user info: {}", e)))?;

    // ユーザーを作成/更新
    let user_row = db_client
        .query_one(
            r#"
            SELECT * FROM auth.upsert_google_user($1, $2, $3, $4, $5)
            "#,
            &[
                &user_info.sub,
                &user_info.email,
                &user_info.name,
                &user_info.picture,
                &user_info.locale,
            ],
        )
        .await?;

    let user_id: i32 = user_row.get("id");

    // セッショントークンを生成
    let session_token = generate_session_token();
    let token_hash = hash_token(&session_token);

    // セッションを作成
    db_client
        .execute(
            r#"
            SELECT auth.create_session($1, $2, NULL, NULL, INTERVAL '7 days')
            "#,
            &[&user_id, &token_hash],
        )
        .await?;

    // Cookie を設定
    let cookie = Cookie::build((SESSION_COOKIE_NAME, session_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::seconds(SESSION_DURATION_SECS))
        .build();

    let updated_jar = jar.add(cookie);

    // フロントエンドにリダイレクト
    let final_redirect = redirect_url.unwrap_or_else(|| "/".to_string());
    let redirect_target = format!("{}{}", oauth_config.frontend_url, final_redirect);

    Ok((updated_jar, Redirect::temporary(&redirect_target).into_response()))
}

/// ログアウト
pub async fn logout(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> Result<(PrivateCookieJar, Response), AppError> {
    // Cookie からセッショントークンを取得
    if let Some(cookie) = jar.get(SESSION_COOKIE_NAME) {
        let token_hash = hash_token(cookie.value());

        // セッションを削除
        let db_client = state.db.get().await?;
        db_client
            .execute(
                "DELETE FROM auth.sessions WHERE token_hash = $1",
                &[&token_hash],
            )
            .await?;
    }

    // Cookie を削除
    let updated_jar = jar.remove(Cookie::from(SESSION_COOKIE_NAME));

    Ok((updated_jar, StatusCode::OK.into_response()))
}

/// 認証状態を取得
pub async fn get_auth_status(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> Result<Json<AuthStatus>, AppError> {
    let Some(cookie) = jar.get(SESSION_COOKIE_NAME) else {
        return Ok(Json(AuthStatus {
            authenticated: false,
            user: None,
        }));
    };

    let token_hash = hash_token(cookie.value());
    let db_client = state.db.get().await?;

    let result = db_client
        .query_opt(
            r#"
            SELECT session_id, user_id, email, name, picture_url, is_valid
            FROM auth.validate_session($1)
            "#,
            &[&token_hash],
        )
        .await?;

    match result {
        Some(row) => {
            let is_valid: bool = row.get("is_valid");
            if is_valid {
                Ok(Json(AuthStatus {
                    authenticated: true,
                    user: Some(AuthUser {
                        id: row.get("user_id"),
                        email: row.get("email"),
                        name: row.get("name"),
                        picture_url: row.get("picture_url"),
                    }),
                }))
            } else {
                Ok(Json(AuthStatus {
                    authenticated: false,
                    user: None,
                }))
            }
        }
        None => Ok(Json(AuthStatus {
            authenticated: false,
            user: None,
        })),
    }
}

/// 現在のユーザー情報を取得（認証必須）
pub async fn get_current_user(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> Result<Json<AuthUser>, AppError> {
    let cookie = jar
        .get(SESSION_COOKIE_NAME)
        .ok_or_else(|| AppError::Unauthorized("Not authenticated".to_string()))?;

    let token_hash = hash_token(cookie.value());
    let db_client = state.db.get().await?;

    let row = db_client
        .query_opt(
            r#"
            SELECT user_id, email, name, picture_url, is_valid
            FROM auth.validate_session($1)
            "#,
            &[&token_hash],
        )
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid session".to_string()))?;

    let is_valid: bool = row.get("is_valid");
    if !is_valid {
        return Err(AppError::Unauthorized("Session expired".to_string()));
    }

    Ok(Json(AuthUser {
        id: row.get("user_id"),
        email: row.get("email"),
        name: row.get("name"),
        picture_url: row.get("picture_url"),
    }))
}

/// セッション Cookie からユーザー情報を取得するヘルパー関数
///
/// 認証されていない場合は `None` を返します。
pub async fn get_user_from_session(
    state: &AppState,
    jar: &PrivateCookieJar,
) -> Result<Option<AuthUser>, AppError> {
    let Some(cookie) = jar.get(SESSION_COOKIE_NAME) else {
        return Ok(None);
    };

    let token_hash = hash_token(cookie.value());
    let db_client = state.db.get().await?;

    let result = db_client
        .query_opt(
            r#"
            SELECT user_id, email, name, picture_url, is_valid
            FROM auth.validate_session($1)
            "#,
            &[&token_hash],
        )
        .await?;

    match result {
        Some(row) => {
            let is_valid: bool = row.get("is_valid");
            if is_valid {
                Ok(Some(AuthUser {
                    id: row.get("user_id"),
                    email: row.get("email"),
                    name: row.get("name"),
                    picture_url: row.get("picture_url"),
                }))
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

/// 認証を要求し、ユーザー情報を取得するヘルパー関数
///
/// 認証されていない場合は `Unauthorized` エラーを返します。
pub async fn require_auth(
    state: &AppState,
    jar: &PrivateCookieJar,
) -> Result<AuthUser, AppError> {
    get_user_from_session(state, jar)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))
}

// ============================================================
// 開発用モック認証（Google Cloud Console 不要）
// ============================================================

/// 開発用モックログイン
///
/// 本番環境では無効化すること。
/// 使用方法: GET /api/auth/dev/login?name=TestUser&email=test@example.com
#[derive(Debug, Deserialize)]
pub struct DevLoginQuery {
    pub name: Option<String>,
    pub email: Option<String>,
}

pub async fn dev_login(
    State(state): State<AppState>,
    Query(query): Query<DevLoginQuery>,
    jar: PrivateCookieJar,
) -> Result<(PrivateCookieJar, Response), AppError> {
    // 開発環境チェック（本番では無効化）
    if std::env::var("DEV_AUTH_ENABLED").unwrap_or_default() != "true" {
        return Err(AppError::NotFound("Dev auth is disabled".to_string()));
    }

    let db_client = state.db.get().await?;

    let name = query.name.unwrap_or_else(|| "Dev User".to_string());
    let email = query.email.unwrap_or_else(|| "dev@localhost".to_string());
    let google_id = format!("dev_{}", email.replace("@", "_").replace(".", "_"));

    // ユーザーを作成/更新
    let user_row = db_client
        .query_one(
            r#"
            SELECT (auth.upsert_google_user($1, $2, $3, $4, $5)).id
            "#,
            &[
                &google_id,
                &email,
                &name,
                &None::<String>, // picture
                &None::<String>, // locale
            ],
        )
        .await
        .map_err(|e| {
            eprintln!("Dev login - upsert_google_user error: {:?}", e);
            e
        })?;

    let user_id: i32 = user_row.get(0);

    // セッショントークンを生成
    let session_token = generate_session_token();
    let token_hash = hash_token(&session_token);

    // セッションを作成（dev_login用）
    db_client
        .execute(
            r#"
            SELECT auth.create_session($1, $2, NULL, NULL, INTERVAL '7 days')
            "#,
            &[&user_id, &token_hash],
        )
        .await
        .map_err(|e| {
            eprintln!("Dev login - create_session error: {:?}", e);
            e
        })?;

    // Cookie を設定
    let cookie = Cookie::build((SESSION_COOKIE_NAME, session_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::seconds(SESSION_DURATION_SECS))
        .build();

    let updated_jar = jar.add(cookie);

    // JSON レスポンスを返す（dev_login）
    let response = Json(serde_json::json!({
        "success": true,
        "user": {
            "id": user_id,
            "email": email,
            "name": name,
        },
        "message": "Dev login successful. This endpoint is for development only."
    }));

    Ok((updated_jar, response.into_response()))
}
