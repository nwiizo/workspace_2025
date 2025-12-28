-- ==============================================================================
-- Just Use Postgres! - サンプルデータ
-- Chapter 1.5: generate_seriesを使ったモックデータ生成
-- ==============================================================================

-- ==============================================================================
-- 顧客データ
-- ==============================================================================

INSERT INTO customers.accounts (email, name, preferences)
SELECT
    'user' || id || '@example.com' AS email,
    'User ' || id AS name,
    jsonb_build_object(
        'theme', (ARRAY['light', 'dark', 'auto'])[floor(random() * 3 + 1)::int],
        'notifications', random() > 0.5,
        'language', (ARRAY['en', 'ja', 'es', 'fr'])[floor(random() * 4 + 1)::int]
    ) AS preferences
FROM generate_series(1, 100) AS id;

-- ==============================================================================
-- 商品カタログ（コーヒーチェーンの例）
-- ==============================================================================

INSERT INTO products.catalog (name, category, price, stock_quantity, description, tags)
VALUES
    ('Espresso', 'coffee', 3.50, 1000, 'A concentrated form of coffee served in small, strong shots. Rich and bold flavor with a layer of crema on top.', '["hot", "classic", "caffeine"]'),
    ('Cappuccino', 'coffee', 4.50, 800, 'Espresso-based coffee drink prepared with steamed milk foam. Perfect balance of strong coffee and creamy milk.', '["hot", "milk", "popular"]'),
    ('Latte', 'coffee', 4.00, 900, 'Espresso with steamed milk and a light layer of foam. Smooth and creamy coffee experience.', '["hot", "milk", "mild"]'),
    ('Americano', 'coffee', 3.00, 1200, 'Espresso diluted with hot water. Similar strength to regular coffee but different flavor.', '["hot", "classic", "simple"]'),
    ('Cold Brew', 'coffee', 4.50, 600, 'Coffee brewed with cold water over 12-24 hours. Less acidic and naturally sweet taste.', '["cold", "smooth", "summer"]'),
    ('Mocha', 'coffee', 5.00, 700, 'Espresso with chocolate and steamed milk. A sweet treat for chocolate lovers.', '["hot", "chocolate", "sweet"]'),
    ('Flat White', 'coffee', 4.00, 750, 'Espresso with microfoam. Velvety texture with strong coffee flavor.', '["hot", "milk", "australian"]'),
    ('Matcha Latte', 'tea', 5.50, 500, 'Green tea powder with steamed milk. Earthy flavor with natural caffeine boost.', '["hot", "tea", "healthy"]'),
    ('Chai Latte', 'tea', 4.50, 550, 'Spiced tea with steamed milk. Warm and comforting with cinnamon and cardamom.', '["hot", "spiced", "comfort"]'),
    ('Iced Tea', 'tea', 3.00, 800, 'Refreshing cold tea. Perfect for hot summer days.', '["cold", "refreshing", "summer"]'),
    ('Croissant', 'pastry', 3.50, 200, 'Buttery French pastry with flaky layers. Freshly baked every morning.', '["breakfast", "butter", "french"]'),
    ('Chocolate Muffin', 'pastry', 4.00, 150, 'Rich chocolate muffin with chocolate chips. Moist and decadent.', '["sweet", "chocolate", "snack"]'),
    ('Blueberry Scone', 'pastry', 3.75, 180, 'Traditional scone with fresh blueberries. Light and crumbly texture.', '["breakfast", "fruit", "british"]'),
    ('Avocado Toast', 'food', 8.50, 100, 'Sourdough toast with smashed avocado, cherry tomatoes, and microgreens.', '["healthy", "breakfast", "trendy"]'),
    ('Bagel with Cream Cheese', 'food', 5.00, 120, 'Fresh bagel with cream cheese. Classic breakfast option.', '["breakfast", "classic", "filling"]');

-- ==============================================================================
-- 音楽ストリーミングデータ（Chapter 3用）
-- ==============================================================================

INSERT INTO streaming.songs (title, artist, album, duration, genre, release_year)
VALUES
    ('Bohemian Rhapsody', 'Queen', 'A Night at the Opera', 354, 'Rock', 1975),
    ('Stairway to Heaven', 'Led Zeppelin', 'Led Zeppelin IV', 482, 'Rock', 1971),
    ('Hotel California', 'Eagles', 'Hotel California', 390, 'Rock', 1977),
    ('Imagine', 'John Lennon', 'Imagine', 183, 'Pop', 1971),
    ('Billie Jean', 'Michael Jackson', 'Thriller', 294, 'Pop', 1982),
    ('Smells Like Teen Spirit', 'Nirvana', 'Nevermind', 301, 'Grunge', 1991),
    ('Like a Rolling Stone', 'Bob Dylan', 'Highway 61 Revisited', 369, 'Rock', 1965),
    ('Hey Jude', 'The Beatles', 'Non-album single', 431, 'Rock', 1968),
    ('Purple Rain', 'Prince', 'Purple Rain', 520, 'Pop', 1984),
    ('Wonderwall', 'Oasis', '(Whats the Story) Morning Glory?', 258, 'Rock', 1995),
    ('Lose Yourself', 'Eminem', '8 Mile Soundtrack', 326, 'Hip Hop', 2002),
    ('Rolling in the Deep', 'Adele', '21', 228, 'Pop', 2010),
    ('Shape of You', 'Ed Sheeran', 'Divide', 233, 'Pop', 2017),
    ('Blinding Lights', 'The Weeknd', 'After Hours', 200, 'Synth-pop', 2019),
    ('Bad Guy', 'Billie Eilish', 'When We All Fall Asleep', 194, 'Pop', 2019);

-- 再生履歴を生成（連続再生のリンク付き）
DO $$
DECLARE
    user_id_var INTEGER;
    song_id_var INTEGER;
    prev_play_id INTEGER := NULL;
    play_time TIMESTAMP WITH TIME ZONE;
    play_dur INTEGER;
    song_dur INTEGER;
    session_count INTEGER;
    plays_in_session INTEGER;
BEGIN
    -- 各ユーザーについて再生履歴を生成
    FOR user_id_var IN 1..20 LOOP
        -- 3-7個のセッションを生成
        session_count := floor(random() * 5 + 3)::int;

        FOR session_idx IN 1..session_count LOOP
            -- セッションの開始時刻（過去30日のランダムな時点）
            play_time := NOW() - (random() * interval '30 days');
            prev_play_id := NULL;

            -- 各セッションで2-8曲再生
            plays_in_session := floor(random() * 7 + 2)::int;

            FOR play_idx IN 1..plays_in_session LOOP
                -- ランダムな曲を選択
                song_id_var := floor(random() * 15 + 1)::int;

                -- 曲の長さを取得
                SELECT duration INTO song_dur FROM streaming.songs WHERE id = song_id_var;

                -- 再生時間（途中で止めることもある）
                play_dur := floor(random() * song_dur * 1.1)::int;
                IF play_dur > song_dur THEN
                    play_dur := song_dur;
                END IF;

                -- 再生履歴を挿入
                INSERT INTO streaming.plays (user_id, song_id, play_start_time, play_duration, played_after)
                VALUES (user_id_var, song_id_var, play_time, play_dur, prev_play_id)
                RETURNING id INTO prev_play_id;

                -- 次の曲の開始時刻
                play_time := play_time + (play_dur || ' seconds')::interval + (random() * interval '30 seconds');
            END LOOP;
        END LOOP;
    END LOOP;
END $$;

-- ==============================================================================
-- レビューデータ
-- ==============================================================================

INSERT INTO products.reviews (product_id, customer_id, rating, comment)
SELECT
    floor(random() * 15 + 1)::int AS product_id,
    floor(random() * 100 + 1)::int AS customer_id,
    floor(random() * 5 + 1)::int AS rating,
    CASE floor(random() * 5)::int
        WHEN 0 THEN 'Great product! Would recommend.'
        WHEN 1 THEN 'Good quality, fast delivery.'
        WHEN 2 THEN 'Average. Expected more for the price.'
        WHEN 3 THEN 'Excellent! Will buy again.'
        WHEN 4 THEN 'Not bad, but could be better.'
    END AS comment
FROM generate_series(1, 200);

-- ==============================================================================
-- 注文データ（Chapter 1の株取引データに相当するもの）
-- ==============================================================================

-- いくつかの注文を作成
DO $$
DECLARE
    customer_id_var INTEGER;
    product_id_var INTEGER;
    order_id_var UUID;
    qty INTEGER;
BEGIN
    FOR i IN 1..50 LOOP
        customer_id_var := floor(random() * 100 + 1)::int;

        -- 新しい注文を作成
        INSERT INTO sales.orders (customer_id, status, total_amount)
        VALUES (customer_id_var, 'completed', 0)
        RETURNING id INTO order_id_var;

        -- 1-5個の商品を追加
        FOR j IN 1..floor(random() * 5 + 1)::int LOOP
            product_id_var := floor(random() * 15 + 1)::int;
            qty := floor(random() * 3 + 1)::int;

            INSERT INTO sales.order_items (order_id, product_id, quantity, price)
            SELECT order_id_var, product_id_var, qty, price
            FROM products.catalog WHERE id = product_id_var
            ON CONFLICT (order_id, product_id) DO NOTHING;
        END LOOP;

        -- 合計を更新
        UPDATE sales.orders
        SET total_amount = (
            SELECT COALESCE(SUM(quantity * price), 0)
            FROM sales.order_items WHERE order_id = order_id_var
        ),
        completed_at = NOW() - (random() * interval '30 days')
        WHERE id = order_id_var;
    END LOOP;
END $$;

-- 統計情報を表示
SELECT 'customers.accounts' AS table_name, COUNT(*) AS row_count FROM customers.accounts
UNION ALL
SELECT 'products.catalog', COUNT(*) FROM products.catalog
UNION ALL
SELECT 'products.reviews', COUNT(*) FROM products.reviews
UNION ALL
SELECT 'streaming.songs', COUNT(*) FROM streaming.songs
UNION ALL
SELECT 'streaming.plays', COUNT(*) FROM streaming.plays
UNION ALL
SELECT 'sales.orders', COUNT(*) FROM sales.orders
UNION ALL
SELECT 'sales.order_items', COUNT(*) FROM sales.order_items;
