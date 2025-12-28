-- =============================================================================
-- 注文・カート・ユーザー関連スキーマ
-- =============================================================================

CREATE SCHEMA IF NOT EXISTS customers;
CREATE SCHEMA IF NOT EXISTS orders;

-- =============================================================================
-- ユーザーテーブル
-- =============================================================================

DROP TABLE IF EXISTS customers.users CASCADE;
CREATE TABLE customers.users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(100) NOT NULL,
    phone VARCHAR(20),
    membership_tier VARCHAR(20) DEFAULT 'bronze',
    points INTEGER DEFAULT 0,
    preferences JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_users_email ON customers.users(email);
CREATE INDEX idx_users_membership ON customers.users(membership_tier);

-- =============================================================================
-- お気に入りテーブル
-- =============================================================================

DROP TABLE IF EXISTS customers.favorites CASCADE;
CREATE TABLE customers.favorites (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES customers.users(id) ON DELETE CASCADE,
    product_id INTEGER NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, product_id)
);

CREATE INDEX idx_favorites_user ON customers.favorites(user_id);

-- =============================================================================
-- カートテーブル
-- =============================================================================

DROP TABLE IF EXISTS orders.carts CASCADE;
CREATE TABLE orders.carts (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES customers.users(id) ON DELETE CASCADE,
    session_id VARCHAR(100),
    store_id INTEGER,
    status VARCHAR(20) DEFAULT 'active',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ DEFAULT NOW() + INTERVAL '24 hours'
);

CREATE INDEX idx_carts_user ON orders.carts(user_id);
CREATE INDEX idx_carts_session ON orders.carts(session_id);
CREATE INDEX idx_carts_status ON orders.carts(status);

DROP TABLE IF EXISTS orders.cart_items CASCADE;
CREATE TABLE orders.cart_items (
    id SERIAL PRIMARY KEY,
    cart_id INTEGER NOT NULL REFERENCES orders.carts(id) ON DELETE CASCADE,
    product_id INTEGER NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1,
    unit_price NUMERIC(10, 2) NOT NULL,
    customizations JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_cart_items_cart ON orders.cart_items(cart_id);

-- =============================================================================
-- 注文テーブル
-- =============================================================================

DROP TABLE IF EXISTS orders.order_items CASCADE;
DROP TABLE IF EXISTS orders.orders CASCADE;

CREATE TABLE orders.orders (
    id SERIAL PRIMARY KEY,
    order_number VARCHAR(20) UNIQUE NOT NULL,
    user_id INTEGER REFERENCES customers.users(id),
    store_id INTEGER NOT NULL,
    status VARCHAR(30) DEFAULT 'pending',
    subtotal NUMERIC(10, 2) NOT NULL,
    tax NUMERIC(10, 2) NOT NULL,
    total NUMERIC(10, 2) NOT NULL,
    points_earned INTEGER DEFAULT 0,
    points_used INTEGER DEFAULT 0,
    payment_method VARCHAR(30),
    payment_status VARCHAR(20) DEFAULT 'pending',
    notes TEXT,
    pickup_time TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_orders_user ON orders.orders(user_id);
CREATE INDEX idx_orders_store ON orders.orders(store_id);
CREATE INDEX idx_orders_status ON orders.orders(status);
CREATE INDEX idx_orders_created ON orders.orders(created_at DESC);
CREATE INDEX idx_orders_number ON orders.orders(order_number);

CREATE TABLE orders.order_items (
    id SERIAL PRIMARY KEY,
    order_id INTEGER NOT NULL REFERENCES orders.orders(id) ON DELETE CASCADE,
    product_id INTEGER NOT NULL,
    product_name VARCHAR(200) NOT NULL,
    quantity INTEGER NOT NULL,
    unit_price NUMERIC(10, 2) NOT NULL,
    subtotal NUMERIC(10, 2) NOT NULL,
    customizations JSONB DEFAULT '{}'
);

CREATE INDEX idx_order_items_order ON orders.order_items(order_id);

-- =============================================================================
-- レビューテーブル
-- =============================================================================

DROP TABLE IF EXISTS customers.reviews CASCADE;
CREATE TABLE customers.reviews (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES customers.users(id) ON DELETE CASCADE,
    product_id INTEGER,
    store_id INTEGER,
    rating INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 5),
    title VARCHAR(200),
    content TEXT,
    helpful_count INTEGER DEFAULT 0,
    verified_purchase BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_reviews_product ON customers.reviews(product_id);
CREATE INDEX idx_reviews_store ON customers.reviews(store_id);
CREATE INDEX idx_reviews_user ON customers.reviews(user_id);
CREATE INDEX idx_reviews_rating ON customers.reviews(rating);

-- =============================================================================
-- サンプルユーザーデータ
-- =============================================================================

INSERT INTO customers.users (email, name, phone, membership_tier, points, preferences)
VALUES
    ('tanaka@example.com', '田中 太郎', '090-1234-5678', 'gold', 2500, '{"favorite_drink": "latte", "size_preference": "M", "notifications": true}'),
    ('suzuki@example.com', '鈴木 花子', '090-2345-6789', 'platinum', 8500, '{"favorite_drink": "matcha", "size_preference": "L", "notifications": true}'),
    ('yamamoto@example.com', '山本 健一', '090-3456-7890', 'bronze', 350, '{"favorite_drink": "espresso", "size_preference": "S", "notifications": false}'),
    ('sato@example.com', '佐藤 美咲', '090-4567-8901', 'silver', 1200, '{"favorite_drink": "cappuccino", "size_preference": "M", "notifications": true}'),
    ('watanabe@example.com', '渡辺 翔太', '090-5678-9012', 'gold', 3800, '{"favorite_drink": "cold_brew", "size_preference": "L", "notifications": true}'),
    ('ito@example.com', '伊藤 あかり', '090-6789-0123', 'bronze', 150, '{"favorite_drink": "tea", "size_preference": "M", "notifications": false}'),
    ('nakamura@example.com', '中村 大輔', '090-7890-1234', 'silver', 980, '{"favorite_drink": "mocha", "size_preference": "M", "notifications": true}'),
    ('kobayashi@example.com', '小林 さくら', '090-8901-2345', 'platinum', 12000, '{"favorite_drink": "frappe", "size_preference": "L", "notifications": true}'),
    ('kato@example.com', '加藤 隆', '090-9012-3456', 'gold', 4200, '{"favorite_drink": "americano", "size_preference": "M", "notifications": false}'),
    ('yoshida@example.com', '吉田 真理', '090-0123-4567', 'bronze', 50, '{"favorite_drink": "latte", "size_preference": "S", "notifications": true}');

-- =============================================================================
-- サンプルお気に入りデータ
-- =============================================================================

INSERT INTO customers.favorites (user_id, product_id)
SELECT
    (RANDOM() * 9 + 1)::INT,
    (RANDOM() * 160 + 1)::INT
FROM generate_series(1, 50)
ON CONFLICT (user_id, product_id) DO NOTHING;

-- =============================================================================
-- サンプル注文データ（過去90日分）
-- =============================================================================

-- 注文を生成する関数
CREATE OR REPLACE FUNCTION orders.generate_order_number()
RETURNS VARCHAR(20) AS $$
BEGIN
    RETURN 'ORD-' || TO_CHAR(NOW(), 'YYYYMMDD') || '-' || LPAD((RANDOM() * 99999)::INT::TEXT, 5, '0');
END;
$$ LANGUAGE plpgsql;

-- 注文データを挿入
DO $$
DECLARE
    v_order_id INTEGER;
    v_subtotal NUMERIC(10, 2);
    v_tax NUMERIC(10, 2);
    v_total NUMERIC(10, 2);
    v_user_id INTEGER;
    v_store_id INTEGER;
    v_order_date TIMESTAMPTZ;
    v_status VARCHAR(30);
    v_item_count INTEGER;
    i INTEGER;
    j INTEGER;
BEGIN
    FOR i IN 1..500 LOOP
        -- ランダムな値を設定
        v_user_id := (RANDOM() * 9 + 1)::INT;
        v_store_id := (RANDOM() * 125 + 1)::INT;
        v_order_date := NOW() - (RANDOM() * 90)::INT * INTERVAL '1 day'
                      - (RANDOM() * 12)::INT * INTERVAL '1 hour';
        v_status := (ARRAY['completed', 'completed', 'completed', 'completed', 'cancelled', 'refunded'])[1 + (RANDOM() * 5)::INT];
        v_item_count := (RANDOM() * 4 + 1)::INT;

        -- 注文を作成
        INSERT INTO orders.orders (
            order_number, user_id, store_id, status,
            subtotal, tax, total,
            points_earned, payment_method, payment_status,
            pickup_time, completed_at, created_at
        ) VALUES (
            'ORD-' || TO_CHAR(v_order_date, 'YYYYMMDD') || '-' || LPAD(i::TEXT, 5, '0'),
            v_user_id,
            v_store_id,
            v_status,
            0, 0, 0,  -- 後で更新
            0,
            (ARRAY['credit_card', 'mobile_pay', 'cash', 'points'])[1 + (RANDOM() * 3)::INT],
            CASE WHEN v_status = 'cancelled' THEN 'refunded' ELSE 'completed' END,
            v_order_date + INTERVAL '15 minutes',
            CASE WHEN v_status IN ('completed', 'cancelled', 'refunded') THEN v_order_date + INTERVAL '20 minutes' ELSE NULL END,
            v_order_date
        ) RETURNING id INTO v_order_id;

        -- 注文アイテムを追加
        v_subtotal := 0;
        FOR j IN 1..v_item_count LOOP
            DECLARE
                v_product_id INTEGER;
                v_product_name VARCHAR(200);
                v_unit_price NUMERIC(10, 2);
                v_quantity INTEGER;
                v_item_subtotal NUMERIC(10, 2);
            BEGIN
                v_product_id := (RANDOM() * 159 + 1)::INT;
                v_quantity := (RANDOM() * 2 + 1)::INT;

                SELECT name, price INTO v_product_name, v_unit_price
                FROM products.catalog WHERE id = v_product_id;

                IF v_product_name IS NULL THEN
                    v_product_name := 'カフェラテ';
                    v_unit_price := 450;
                END IF;

                v_item_subtotal := v_unit_price * v_quantity;
                v_subtotal := v_subtotal + v_item_subtotal;

                INSERT INTO orders.order_items (
                    order_id, product_id, product_name,
                    quantity, unit_price, subtotal,
                    customizations
                ) VALUES (
                    v_order_id,
                    v_product_id,
                    v_product_name,
                    v_quantity,
                    v_unit_price,
                    v_item_subtotal,
                    CASE WHEN RANDOM() > 0.7 THEN
                        jsonb_build_object(
                            'size', (ARRAY['S', 'M', 'L'])[1 + (RANDOM() * 2)::INT],
                            'extra_shot', RANDOM() > 0.8,
                            'sugar', (ARRAY['none', 'half', 'normal'])[1 + (RANDOM() * 2)::INT]
                        )
                    ELSE '{}'::JSONB
                    END
                );
            END;
        END LOOP;

        -- 注文の合計を更新
        v_tax := ROUND(v_subtotal * 0.1, 0);
        v_total := v_subtotal + v_tax;

        UPDATE orders.orders
        SET subtotal = v_subtotal,
            tax = v_tax,
            total = v_total,
            points_earned = FLOOR(v_total / 100)::INT
        WHERE id = v_order_id;
    END LOOP;
END $$;

-- =============================================================================
-- サンプルレビューデータ
-- =============================================================================

INSERT INTO customers.reviews (user_id, product_id, store_id, rating, title, content, helpful_count, verified_purchase, created_at)
SELECT
    (RANDOM() * 9 + 1)::INT,
    CASE WHEN RANDOM() > 0.3 THEN (RANDOM() * 159 + 1)::INT ELSE NULL END,
    CASE WHEN RANDOM() > 0.5 THEN (RANDOM() * 125 + 1)::INT ELSE NULL END,
    (RANDOM() * 2 + 3)::INT,  -- 3-5の評価が多い
    (ARRAY[
        '美味しい！',
        '毎日飲んでます',
        'お気に入り',
        'リピート確定',
        '香りが最高',
        'コスパ良し',
        '期待以上',
        '少し甘め',
        'さっぱりして良い',
        '接客が丁寧'
    ])[1 + (RANDOM() * 9)::INT],
    (ARRAY[
        '毎朝の楽しみになっています。苦味と酸味のバランスが絶妙で、何度でも飲みたくなる味です。',
        '仕事の合間に立ち寄っています。スタッフの方も親切で、居心地が良いです。',
        '新商品を試しましたが、期待以上の美味しさでした。季節限定なのが残念です。',
        '友人に勧められて来ました。確かに美味しいですが、少し高めかな。',
        'テイクアウトで利用しましたが、温度もちょうど良く、最後まで美味しく飲めました。',
        '店内が綺麗で落ち着きます。Wi-Fiも使えるので作業にも最適。',
        'カスタマイズの種類が豊富で嬉しい。自分好みの一杯が作れます。',
        'モバイルオーダーを使いましたが、待ち時間なしでスムーズに受け取れました。',
        '甘さ控えめにしてもらいましたが、しっかり風味が残っていて満足です。',
        'ポイントが貯まりやすいので、よく利用しています。ゴールド会員目指します！'
    ])[1 + (RANDOM() * 9)::INT],
    (RANDOM() * 20)::INT,
    RANDOM() > 0.2,
    NOW() - (RANDOM() * 180)::INT * INTERVAL '1 day'
FROM generate_series(1, 200);

-- =============================================================================
-- 統計情報
-- =============================================================================

SELECT 'Users: ' || COUNT(*) FROM customers.users;
SELECT 'Favorites: ' || COUNT(*) FROM customers.favorites;
SELECT 'Orders: ' || COUNT(*) FROM orders.orders;
SELECT 'Order Items: ' || COUNT(*) FROM orders.order_items;
SELECT 'Reviews: ' || COUNT(*) FROM customers.reviews;

-- 注文統計
SELECT
    status,
    COUNT(*) as count,
    ROUND(AVG(total)::NUMERIC, 0) as avg_total,
    ROUND(SUM(total)::NUMERIC, 0) as sum_total
FROM orders.orders
GROUP BY status
ORDER BY count DESC;
