-- =============================================================================
-- スキーマ作成
-- =============================================================================

-- 商品スキーマ
CREATE SCHEMA IF NOT EXISTS products;

-- 店舗スキーマ（PostGIS用）
CREATE SCHEMA IF NOT EXISTS stores;

-- メトリクススキーマ（TimescaleDB用）
CREATE SCHEMA IF NOT EXISTS metrics;

-- 拡張機能の有効化（利用可能な場合）
DO $$
BEGIN
    -- PostGIS
    IF EXISTS (SELECT 1 FROM pg_available_extensions WHERE name = 'postgis') THEN
        CREATE EXTENSION IF NOT EXISTS postgis;
        RAISE NOTICE 'PostGIS extension enabled';
    ELSE
        RAISE NOTICE 'PostGIS extension not available, skipping...';
    END IF;

    -- pgvector
    IF EXISTS (SELECT 1 FROM pg_available_extensions WHERE name = 'vector') THEN
        CREATE EXTENSION IF NOT EXISTS vector;
        RAISE NOTICE 'pgvector extension enabled';
    ELSE
        RAISE NOTICE 'pgvector extension not available, skipping...';
    END IF;

    -- pg_trgm (全文検索用)
    CREATE EXTENSION IF NOT EXISTS pg_trgm;
    RAISE NOTICE 'pg_trgm extension enabled';
END $$;

-- =============================================================================
-- 商品テーブル
-- =============================================================================

DROP TABLE IF EXISTS products.catalog CASCADE;
CREATE TABLE products.catalog (
    id SERIAL PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    category VARCHAR(50) NOT NULL,
    price NUMERIC(10, 2) NOT NULL,
    description TEXT,
    attributes JSONB DEFAULT '{}',
    tags TEXT[],
    search_vector TSVECTOR,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 全文検索用インデックス
CREATE INDEX idx_products_search_vector ON products.catalog USING GIN(search_vector);
CREATE INDEX idx_products_name_trgm ON products.catalog USING GIN(name gin_trgm_ops);
CREATE INDEX idx_products_category ON products.catalog(category);
CREATE INDEX idx_products_price ON products.catalog(price);
CREATE INDEX idx_products_attributes ON products.catalog USING GIN(attributes);
CREATE INDEX idx_products_tags ON products.catalog USING GIN(tags);

-- 検索ベクトル自動更新トリガー
CREATE OR REPLACE FUNCTION products.update_search_vector()
RETURNS TRIGGER AS $$
BEGIN
    NEW.search_vector :=
        setweight(to_tsvector('english', COALESCE(NEW.name, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(NEW.description, '')), 'B') ||
        setweight(to_tsvector('english', COALESCE(NEW.category, '')), 'C');
    NEW.updated_at := NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_search_vector
    BEFORE INSERT OR UPDATE ON products.catalog
    FOR EACH ROW EXECUTE FUNCTION products.update_search_vector();

-- =============================================================================
-- 店舗テーブル
-- =============================================================================

DROP TABLE IF EXISTS stores.locations CASCADE;
CREATE TABLE stores.locations (
    id SERIAL PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    address VARCHAR(500) NOT NULL,
    city VARCHAR(100) NOT NULL,
    prefecture VARCHAR(50) NOT NULL,
    postal_code VARCHAR(10),
    phone VARCHAR(20),
    lat DOUBLE PRECISION NOT NULL,
    lon DOUBLE PRECISION NOT NULL,
    store_type VARCHAR(50) DEFAULT 'standard',
    opening_hours JSONB DEFAULT '{}',
    facilities TEXT[],
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 位置情報カラム（PostGISが有効な場合）
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'postgis') THEN
        ALTER TABLE stores.locations ADD COLUMN IF NOT EXISTS location GEOGRAPHY(POINT, 4326);
        CREATE INDEX IF NOT EXISTS idx_stores_location ON stores.locations USING GIST(location);
        RAISE NOTICE 'PostGIS location column added';
    END IF;
END $$;

CREATE INDEX idx_stores_city ON stores.locations(city);
CREATE INDEX idx_stores_prefecture ON stores.locations(prefecture);
CREATE INDEX idx_stores_type ON stores.locations(store_type);
CREATE INDEX idx_stores_lat_lon ON stores.locations(lat, lon);

-- =============================================================================
-- センサー・メトリクステーブル
-- =============================================================================

DROP TABLE IF EXISTS metrics.sensors CASCADE;
CREATE TABLE metrics.sensors (
    id SERIAL PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    sensor_type VARCHAR(50) NOT NULL,
    location VARCHAR(200),
    unit VARCHAR(20),
    min_value DOUBLE PRECISION,
    max_value DOUBLE PRECISION,
    is_active BOOLEAN DEFAULT TRUE,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

DROP TABLE IF EXISTS metrics.measurements CASCADE;
CREATE TABLE metrics.measurements (
    id BIGSERIAL,
    sensor_id INTEGER NOT NULL REFERENCES metrics.sensors(id),
    value DOUBLE PRECISION NOT NULL,
    quality VARCHAR(20) DEFAULT 'good',
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, recorded_at)
);

-- TimescaleDBが有効な場合はハイパーテーブルに変換
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'timescaledb') THEN
        PERFORM create_hypertable('metrics.measurements', 'recorded_at', if_not_exists => TRUE);
        RAISE NOTICE 'TimescaleDB hypertable created';
    ELSE
        RAISE NOTICE 'TimescaleDB not available, using standard table';
    END IF;
END $$;

CREATE INDEX idx_measurements_sensor_time ON metrics.measurements(sensor_id, recorded_at DESC);

-- =============================================================================
-- ユーザーアクティビティテーブル（分析用）
-- =============================================================================

DROP TABLE IF EXISTS metrics.user_activities CASCADE;
CREATE TABLE metrics.user_activities (
    id BIGSERIAL,
    user_id VARCHAR(50) NOT NULL,
    activity_type VARCHAR(50) NOT NULL,
    product_id INTEGER,
    store_id INTEGER,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, created_at)
);

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'timescaledb') THEN
        PERFORM create_hypertable('metrics.user_activities', 'created_at', if_not_exists => TRUE);
        RAISE NOTICE 'User activities hypertable created';
    END IF;
END $$;

CREATE INDEX idx_user_activities_user ON metrics.user_activities(user_id, created_at DESC);
CREATE INDEX idx_user_activities_type ON metrics.user_activities(activity_type);

RAISE NOTICE 'Schema creation completed!';
