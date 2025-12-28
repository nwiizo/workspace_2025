-- ==============================================================================
-- Just Use Postgres! - 技術検証用スキーマ
-- Chapter 2: Standard RDBMS capabilities - マルチスキーマ設計
-- ==============================================================================

-- スキーマの作成（マイクロサービス的な分割）
CREATE SCHEMA IF NOT EXISTS products;
CREATE SCHEMA IF NOT EXISTS customers;
CREATE SCHEMA IF NOT EXISTS sales;
CREATE SCHEMA IF NOT EXISTS streaming;

-- ==============================================================================
-- products スキーマ
-- ==============================================================================

CREATE TABLE products.catalog (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    category VARCHAR(50) NOT NULL,
    price DECIMAL(10, 2) NOT NULL CHECK (price > 0),
    stock_quantity INTEGER NOT NULL DEFAULT 0 CHECK (stock_quantity >= 0),
    description TEXT,
    tags JSONB DEFAULT '[]'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE products.reviews (
    id SERIAL PRIMARY KEY,
    product_id INTEGER NOT NULL REFERENCES products.catalog(id) ON DELETE CASCADE,
    customer_id INTEGER NOT NULL,
    rating INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 5),
    comment TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 全文検索用のインデックス
CREATE INDEX idx_catalog_description_fts ON products.catalog
    USING GIN (to_tsvector('english', COALESCE(description, '')));

-- JSONB用のインデックス
CREATE INDEX idx_catalog_tags ON products.catalog USING GIN (tags);

-- ==============================================================================
-- customers スキーマ
-- ==============================================================================

CREATE TABLE customers.accounts (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    preferences JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- ==============================================================================
-- sales スキーマ
-- ==============================================================================

CREATE TYPE sales.order_status AS ENUM ('pending', 'completed', 'cancelled');

CREATE TABLE sales.orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id INTEGER NOT NULL REFERENCES customers.accounts(id),
    status sales.order_status NOT NULL DEFAULT 'pending',
    total_amount DECIMAL(10, 2) NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP WITH TIME ZONE
);

CREATE TABLE sales.order_items (
    id SERIAL PRIMARY KEY,
    order_id UUID NOT NULL REFERENCES sales.orders(id) ON DELETE CASCADE,
    product_id INTEGER NOT NULL REFERENCES products.catalog(id),
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    price DECIMAL(10, 2) NOT NULL,
    UNIQUE (order_id, product_id)
);

-- ==============================================================================
-- streaming スキーマ（Chapter 3 Modern SQL用）
-- ==============================================================================

CREATE TABLE streaming.songs (
    id SERIAL PRIMARY KEY,
    title VARCHAR(200) NOT NULL,
    artist VARCHAR(100) NOT NULL,
    album VARCHAR(200),
    duration INTEGER NOT NULL CHECK (duration > 0), -- seconds
    genre VARCHAR(50),
    release_year INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE streaming.plays (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES customers.accounts(id),
    song_id INTEGER NOT NULL REFERENCES streaming.songs(id),
    play_start_time TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    play_duration INTEGER NOT NULL CHECK (play_duration >= 0), -- seconds
    played_after INTEGER REFERENCES streaming.plays(id), -- 連続再生のリンク
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- ==============================================================================
-- ビュー（Chapter 2.7）
-- ==============================================================================

CREATE VIEW sales.product_sales_summary AS
SELECT
    c.id AS product_id,
    c.name AS product_name,
    c.category,
    COALESCE(SUM(oi.quantity), 0) AS total_quantity_sold,
    COALESCE(SUM(oi.quantity * oi.price), 0) AS total_revenue
FROM products.catalog c
LEFT JOIN sales.order_items oi ON c.id = oi.product_id
LEFT JOIN sales.orders o ON oi.order_id = o.id AND o.status = 'completed'
GROUP BY c.id
ORDER BY total_quantity_sold DESC, total_revenue DESC;

-- ==============================================================================
-- 関数（Chapter 2.6）
-- ==============================================================================

-- カートに商品を追加する関数
CREATE OR REPLACE FUNCTION sales.order_add_item(
    customer_id_param INTEGER,
    product_id_param INTEGER,
    quantity_param INTEGER
)
RETURNS TABLE (
    out_order_id UUID,
    out_product_name VARCHAR,
    out_quantity INTEGER,
    out_unit_price DECIMAL,
    out_line_total DECIMAL
) AS $$
DECLARE
    pending_order_id UUID;
    product_price DECIMAL(10, 2);
BEGIN
    -- 商品の価格を取得
    SELECT price INTO product_price
    FROM products.catalog
    WHERE id = product_id_param;

    IF product_price IS NULL THEN
        RAISE EXCEPTION 'Product not found: %', product_id_param;
    END IF;

    -- 既存のpending orderを探す
    SELECT o.id INTO pending_order_id
    FROM sales.orders o
    WHERE o.customer_id = customer_id_param AND o.status = 'pending';

    -- なければ作る
    IF pending_order_id IS NULL THEN
        INSERT INTO sales.orders (customer_id, status)
        VALUES (customer_id_param, 'pending')
        RETURNING id INTO pending_order_id;
    END IF;

    -- 商品を追加または更新（MERGE文の代わりにINSERT ON CONFLICT）
    INSERT INTO sales.order_items (order_id, product_id, quantity, price)
    VALUES (pending_order_id, product_id_param, quantity_param, product_price)
    ON CONFLICT (order_id, product_id)
    DO UPDATE SET quantity = sales.order_items.quantity + quantity_param;

    -- 注文合計を更新
    UPDATE sales.orders o
    SET total_amount = (
        SELECT COALESCE(SUM(oi2.quantity * oi2.price), 0)
        FROM sales.order_items oi2
        WHERE oi2.order_id = pending_order_id
    )
    WHERE o.id = pending_order_id;

    -- 結果を返す
    RETURN QUERY
    SELECT
        oi.order_id AS out_order_id,
        c.name::VARCHAR AS out_product_name,
        oi.quantity AS out_quantity,
        oi.price AS out_unit_price,
        (oi.quantity * oi.price) AS out_line_total
    FROM sales.order_items oi
    JOIN products.catalog c ON c.id = oi.product_id
    WHERE oi.order_id = pending_order_id;
END;
$$ LANGUAGE plpgsql;

-- 注文を完了する関数
CREATE OR REPLACE FUNCTION sales.order_checkout(
    customer_id_param INTEGER
)
RETURNS TABLE (
    out_order_id UUID,
    out_total_amount DECIMAL,
    out_status TEXT,
    out_completed_at TIMESTAMP WITH TIME ZONE
) AS $$
DECLARE
    pending_order_id UUID;
BEGIN
    -- pending orderを探す
    SELECT o.id INTO pending_order_id
    FROM sales.orders o
    WHERE o.customer_id = customer_id_param AND o.status = 'pending';

    IF pending_order_id IS NULL THEN
        RAISE EXCEPTION 'No pending order found for customer: %', customer_id_param;
    END IF;

    -- 在庫を確認して減らす
    UPDATE products.catalog c
    SET stock_quantity = c.stock_quantity - oi.quantity
    FROM sales.order_items oi
    WHERE oi.order_id = pending_order_id
      AND c.id = oi.product_id;

    -- 注文を完了
    UPDATE sales.orders o2
    SET status = 'completed',
        completed_at = CURRENT_TIMESTAMP
    WHERE o2.id = pending_order_id;

    RETURN QUERY
    SELECT o.id AS out_order_id, o.total_amount AS out_total_amount,
           o.status::TEXT AS out_status, o.completed_at AS out_completed_at
    FROM sales.orders o
    WHERE o.id = pending_order_id;
END;
$$ LANGUAGE plpgsql;

-- トリガー関数：order_itemsの変更時に合計を更新
CREATE OR REPLACE FUNCTION sales.update_order_total()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE sales.orders
    SET total_amount = (
        SELECT COALESCE(SUM(quantity * price), 0)
        FROM sales.order_items
        WHERE order_id = COALESCE(NEW.order_id, OLD.order_id)
    )
    WHERE id = COALESCE(NEW.order_id, OLD.order_id);

    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_order_total
AFTER INSERT OR UPDATE OR DELETE ON sales.order_items
FOR EACH ROW
EXECUTE FUNCTION sales.update_order_total();
