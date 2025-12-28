-- =============================================================================
-- 認証・セッション管理スキーマ
-- PostgreSQL をセッションストアとして使用
-- =============================================================================

-- 認証スキーマ
CREATE SCHEMA IF NOT EXISTS auth;

-- =============================================================================
-- ユーザーテーブル（Google OAuth 連携）
-- =============================================================================

DROP TABLE IF EXISTS auth.users CASCADE;
CREATE TABLE auth.users (
    id SERIAL PRIMARY KEY,
    google_id VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(200) NOT NULL,
    picture_url TEXT,
    locale VARCHAR(10),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_login_at TIMESTAMPTZ
);

CREATE INDEX idx_auth_users_google_id ON auth.users(google_id);
CREATE INDEX idx_auth_users_email ON auth.users(email);

-- =============================================================================
-- セッションテーブル
-- PostgreSQL をセッションストアとして使用（Redis 不要）
-- =============================================================================

DROP TABLE IF EXISTS auth.sessions CASCADE;
CREATE TABLE auth.sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id INTEGER NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    token_hash VARCHAR(64) NOT NULL UNIQUE,  -- SHA-256 ハッシュ
    user_agent TEXT,
    ip_address INET,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    last_activity_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_auth_sessions_user_id ON auth.sessions(user_id);
CREATE INDEX idx_auth_sessions_token_hash ON auth.sessions(token_hash);
CREATE INDEX idx_auth_sessions_expires_at ON auth.sessions(expires_at);

-- =============================================================================
-- OAuth 状態管理テーブル（CSRF 対策）
-- =============================================================================

DROP TABLE IF EXISTS auth.oauth_states CASCADE;
CREATE TABLE auth.oauth_states (
    state VARCHAR(64) PRIMARY KEY,
    pkce_verifier VARCHAR(128),  -- PKCE code_verifier
    nonce VARCHAR(64),           -- OIDC nonce
    redirect_url TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '10 minutes')
);

CREATE INDEX idx_oauth_states_expires_at ON auth.oauth_states(expires_at);

-- =============================================================================
-- 期限切れセッション自動削除関数
-- =============================================================================

CREATE OR REPLACE FUNCTION auth.cleanup_expired_sessions()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM auth.sessions WHERE expires_at < NOW();
    GET DIAGNOSTICS deleted_count = ROW_COUNT;

    DELETE FROM auth.oauth_states WHERE expires_at < NOW();

    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- =============================================================================
-- セッション作成関数
-- =============================================================================

CREATE OR REPLACE FUNCTION auth.create_session(
    p_user_id INTEGER,
    p_token_hash VARCHAR(64),
    p_user_agent TEXT DEFAULT NULL,
    p_ip_address INET DEFAULT NULL,
    p_expires_in INTERVAL DEFAULT INTERVAL '7 days'
)
RETURNS auth.sessions AS $$
DECLARE
    new_session auth.sessions;
BEGIN
    INSERT INTO auth.sessions (user_id, token_hash, user_agent, ip_address, expires_at)
    VALUES (p_user_id, p_token_hash, p_user_agent, p_ip_address, NOW() + p_expires_in)
    RETURNING * INTO new_session;

    -- ユーザーの最終ログイン時刻を更新
    UPDATE auth.users SET last_login_at = NOW() WHERE id = p_user_id;

    RETURN new_session;
END;
$$ LANGUAGE plpgsql;

-- =============================================================================
-- セッション検証関数
-- =============================================================================

CREATE OR REPLACE FUNCTION auth.validate_session(p_token_hash VARCHAR(64))
RETURNS TABLE (
    session_id UUID,
    user_id INTEGER,
    email VARCHAR(255),
    name VARCHAR(200),
    picture_url TEXT,
    is_valid BOOLEAN
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        s.id AS session_id,
        u.id AS user_id,
        u.email,
        u.name,
        u.picture_url,
        (s.expires_at > NOW()) AS is_valid
    FROM auth.sessions s
    JOIN auth.users u ON s.user_id = u.id
    WHERE s.token_hash = p_token_hash;

    -- セッションが有効なら最終アクティビティ時刻を更新
    UPDATE auth.sessions
    SET last_activity_at = NOW()
    WHERE token_hash = p_token_hash AND expires_at > NOW();
END;
$$ LANGUAGE plpgsql;

-- =============================================================================
-- ユーザー作成/更新関数（Google OAuth コールバック用）
-- =============================================================================

CREATE OR REPLACE FUNCTION auth.upsert_google_user(
    p_google_id VARCHAR(255),
    p_email VARCHAR(255),
    p_name VARCHAR(200),
    p_picture_url TEXT DEFAULT NULL,
    p_locale VARCHAR(10) DEFAULT NULL
)
RETURNS auth.users AS $$
DECLARE
    result_user auth.users;
BEGIN
    INSERT INTO auth.users (google_id, email, name, picture_url, locale, last_login_at)
    VALUES (p_google_id, p_email, p_name, p_picture_url, p_locale, NOW())
    ON CONFLICT (google_id) DO UPDATE SET
        email = EXCLUDED.email,
        name = EXCLUDED.name,
        picture_url = EXCLUDED.picture_url,
        locale = EXCLUDED.locale,
        updated_at = NOW(),
        last_login_at = NOW()
    RETURNING * INTO result_user;

    RETURN result_user;
END;
$$ LANGUAGE plpgsql;

-- =============================================================================
-- トリガー：updated_at 自動更新
-- =============================================================================

CREATE OR REPLACE FUNCTION auth.update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_users_updated_at
    BEFORE UPDATE ON auth.users
    FOR EACH ROW EXECUTE FUNCTION auth.update_timestamp();

RAISE NOTICE 'Auth schema created successfully!';
