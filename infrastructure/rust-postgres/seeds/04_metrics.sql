-- =============================================================================
-- センサー・メトリクスデータ（100万件以上の時系列データ）
-- =============================================================================

-- センサーマスターデータ（50台）
INSERT INTO metrics.sensors (name, sensor_type, location, unit, min_value, max_value, metadata)
VALUES
    -- 温度センサー（20台）
    ('温度センサー 1F-ロビー', 'temperature', '1F ロビー', '℃', -10, 50, '{"manufacturer": "Sensirion", "model": "SHT31", "accuracy": 0.3}'),
    ('温度センサー 1F-カフェ', 'temperature', '1F カフェエリア', '℃', -10, 50, '{"manufacturer": "Sensirion", "model": "SHT31", "accuracy": 0.3}'),
    ('温度センサー 2F-オフィス', 'temperature', '2F オフィス', '℃', -10, 50, '{"manufacturer": "Sensirion", "model": "SHT31", "accuracy": 0.3}'),
    ('温度センサー 2F-会議室A', 'temperature', '2F 会議室A', '℃', -10, 50, '{"manufacturer": "Sensirion", "model": "SHT31", "accuracy": 0.3}'),
    ('温度センサー 2F-会議室B', 'temperature', '2F 会議室B', '℃', -10, 50, '{"manufacturer": "Sensirion", "model": "SHT31", "accuracy": 0.3}'),
    ('温度センサー 3F-オフィス', 'temperature', '3F オフィス', '℃', -10, 50, '{"manufacturer": "Sensirion", "model": "SHT31", "accuracy": 0.3}'),
    ('温度センサー 3F-サーバールーム', 'temperature', '3F サーバールーム', '℃', -10, 50, '{"manufacturer": "Sensirion", "model": "SHT31", "accuracy": 0.3}'),
    ('温度センサー B1-倉庫', 'temperature', 'B1 倉庫', '℃', -10, 50, '{"manufacturer": "Sensirion", "model": "SHT31", "accuracy": 0.3}'),
    ('温度センサー 屋外-北側', 'temperature', '屋外 北側', '℃', -20, 50, '{"manufacturer": "Sensirion", "model": "SHT40", "accuracy": 0.2}'),
    ('温度センサー 屋外-南側', 'temperature', '屋外 南側', '℃', -20, 50, '{"manufacturer": "Sensirion", "model": "SHT40", "accuracy": 0.2}'),

    -- 湿度センサー（10台）
    ('湿度センサー 1F-ロビー', 'humidity', '1F ロビー', '%', 0, 100, '{"manufacturer": "Sensirion", "model": "SHT31", "accuracy": 2}'),
    ('湿度センサー 1F-カフェ', 'humidity', '1F カフェエリア', '%', 0, 100, '{"manufacturer": "Sensirion", "model": "SHT31", "accuracy": 2}'),
    ('湿度センサー 2F-オフィス', 'humidity', '2F オフィス', '%', 0, 100, '{"manufacturer": "Sensirion", "model": "SHT31", "accuracy": 2}'),
    ('湿度センサー 3F-オフィス', 'humidity', '3F オフィス', '%', 0, 100, '{"manufacturer": "Sensirion", "model": "SHT31", "accuracy": 2}'),
    ('湿度センサー 3F-サーバールーム', 'humidity', '3F サーバールーム', '%', 0, 100, '{"manufacturer": "Sensirion", "model": "SHT31", "accuracy": 2}'),

    -- 電力メーター（5台）
    ('電力メーター 全館', 'power', 'メインブレーカー', 'kW', 0, 500, '{"manufacturer": "Panasonic", "model": "KW9M", "phase": "3"}'),
    ('電力メーター 1F', 'power', '1F 分電盤', 'kW', 0, 100, '{"manufacturer": "Panasonic", "model": "KW8M", "phase": "3"}'),
    ('電力メーター 2F', 'power', '2F 分電盤', 'kW', 0, 100, '{"manufacturer": "Panasonic", "model": "KW8M", "phase": "3"}'),
    ('電力メーター 3F', 'power', '3F 分電盤', 'kW', 0, 100, '{"manufacturer": "Panasonic", "model": "KW8M", "phase": "3"}'),
    ('電力メーター サーバールーム', 'power', '3F サーバールーム', 'kW', 0, 50, '{"manufacturer": "Panasonic", "model": "KW8M", "phase": "1"}'),

    -- CO2センサー（5台）
    ('CO2センサー 1F-ロビー', 'co2', '1F ロビー', 'ppm', 300, 5000, '{"manufacturer": "Senseair", "model": "S8", "accuracy": 40}'),
    ('CO2センサー 2F-オフィス', 'co2', '2F オフィス', 'ppm', 300, 5000, '{"manufacturer": "Senseair", "model": "S8", "accuracy": 40}'),
    ('CO2センサー 2F-会議室A', 'co2', '2F 会議室A', 'ppm', 300, 5000, '{"manufacturer": "Senseair", "model": "S8", "accuracy": 40}'),
    ('CO2センサー 3F-オフィス', 'co2', '3F オフィス', 'ppm', 300, 5000, '{"manufacturer": "Senseair", "model": "S8", "accuracy": 40}'),
    ('CO2センサー 3F-サーバールーム', 'co2', '3F サーバールーム', 'ppm', 300, 5000, '{"manufacturer": "Senseair", "model": "S8", "accuracy": 40}'),

    -- 心拍センサー（ウェアラブル 10台）
    ('心拍センサー User001', 'heartRate', 'ウェアラブル', 'bpm', 40, 200, '{"manufacturer": "Polar", "model": "H10", "user_id": "user001"}'),
    ('心拍センサー User002', 'heartRate', 'ウェアラブル', 'bpm', 40, 200, '{"manufacturer": "Polar", "model": "H10", "user_id": "user002"}'),
    ('心拍センサー User003', 'heartRate', 'ウェアラブル', 'bpm', 40, 200, '{"manufacturer": "Polar", "model": "H10", "user_id": "user003"}'),
    ('心拍センサー User004', 'heartRate', 'ウェアラブル', 'bpm', 40, 200, '{"manufacturer": "Polar", "model": "H10", "user_id": "user004"}'),
    ('心拍センサー User005', 'heartRate', 'ウェアラブル', 'bpm', 40, 200, '{"manufacturer": "Polar", "model": "H10", "user_id": "user005"}'),
    ('心拍センサー User006', 'heartRate', 'ウェアラブル', 'bpm', 40, 200, '{"manufacturer": "Garmin", "model": "HRM-Pro", "user_id": "user006"}'),
    ('心拍センサー User007', 'heartRate', 'ウェアラブル', 'bpm', 40, 200, '{"manufacturer": "Garmin", "model": "HRM-Pro", "user_id": "user007"}'),
    ('心拍センサー User008', 'heartRate', 'ウェアラブル', 'bpm', 40, 200, '{"manufacturer": "Garmin", "model": "HRM-Pro", "user_id": "user008"}'),
    ('心拍センサー User009', 'heartRate', 'ウェアラブル', 'bpm', 40, 200, '{"manufacturer": "Garmin", "model": "HRM-Pro", "user_id": "user009"}'),
    ('心拍センサー User010', 'heartRate', 'ウェアラブル', 'bpm', 40, 200, '{"manufacturer": "Garmin", "model": "HRM-Pro", "user_id": "user010"}');

-- =============================================================================
-- 時系列データ生成（過去30日分、1分間隔）
-- 約50センサー × 30日 × 24時間 × 60分 = 約216万レコード
-- =============================================================================

-- 温度センサーデータ（10台 × 30日 × 1440分 = 432,000件）
INSERT INTO metrics.measurements (sensor_id, value, quality, recorded_at)
SELECT
    s.id,
    -- 時間帯と日による温度変動をシミュレート
    CASE
        WHEN s.location LIKE '%屋外%' THEN
            15 + 10 * SIN(2 * PI() * (EXTRACT(HOUR FROM t) - 6) / 24)  -- 屋外: 5-25℃
            + 5 * SIN(2 * PI() * EXTRACT(DOY FROM t) / 365)  -- 季節変動
            + (RANDOM() - 0.5) * 3  -- ノイズ
        WHEN s.location LIKE '%サーバー%' THEN
            22 + (RANDOM() - 0.5) * 2  -- サーバールーム: 21-23℃（安定）
        ELSE
            23 + 3 * SIN(2 * PI() * (EXTRACT(HOUR FROM t) - 14) / 24)  -- 室内: 20-26℃
            + (RANDOM() - 0.5) * 1.5
    END,
    CASE WHEN RANDOM() > 0.001 THEN 'good' ELSE 'suspect' END,
    t
FROM metrics.sensors s
CROSS JOIN generate_series(
    NOW() - INTERVAL '30 days',
    NOW(),
    INTERVAL '1 minute'
) AS t
WHERE s.sensor_type = 'temperature'
  AND s.id <= 10;  -- 最初の10台のみ

-- 湿度センサーデータ（5台 × 30日 × 1440分 = 216,000件）
INSERT INTO metrics.measurements (sensor_id, value, quality, recorded_at)
SELECT
    s.id,
    CASE
        WHEN s.location LIKE '%サーバー%' THEN
            45 + (RANDOM() - 0.5) * 5  -- サーバールーム: 42-48%
        ELSE
            50 + 15 * SIN(2 * PI() * EXTRACT(DOY FROM t) / 365)  -- 季節で35-65%
            + 10 * SIN(2 * PI() * (EXTRACT(HOUR FROM t) - 6) / 24)  -- 日内変動
            + (RANDOM() - 0.5) * 8
    END,
    CASE WHEN RANDOM() > 0.002 THEN 'good' ELSE 'suspect' END,
    t
FROM metrics.sensors s
CROSS JOIN generate_series(
    NOW() - INTERVAL '30 days',
    NOW(),
    INTERVAL '1 minute'
) AS t
WHERE s.sensor_type = 'humidity';

-- 電力データ（5台 × 30日 × 1440分 = 216,000件）
INSERT INTO metrics.measurements (sensor_id, value, quality, recorded_at)
SELECT
    s.id,
    CASE
        WHEN s.location = 'メインブレーカー' THEN
            -- 全館: ビジネスアワーで高負荷
            CASE
                WHEN EXTRACT(DOW FROM t) IN (0, 6) THEN 80 + RANDOM() * 40  -- 週末
                WHEN EXTRACT(HOUR FROM t) BETWEEN 9 AND 18 THEN 200 + RANDOM() * 150  -- 営業時間
                WHEN EXTRACT(HOUR FROM t) BETWEEN 19 AND 22 THEN 150 + RANDOM() * 80  -- 夕方
                ELSE 60 + RANDOM() * 30  -- 深夜
            END
        WHEN s.location LIKE '%サーバールーム%' THEN
            25 + RANDOM() * 10  -- サーバールーム: 常時稼働
        ELSE
            -- 各フロア
            CASE
                WHEN EXTRACT(DOW FROM t) IN (0, 6) THEN 15 + RANDOM() * 10
                WHEN EXTRACT(HOUR FROM t) BETWEEN 9 AND 18 THEN 40 + RANDOM() * 30
                ELSE 10 + RANDOM() * 10
            END
    END,
    'good',
    t
FROM metrics.sensors s
CROSS JOIN generate_series(
    NOW() - INTERVAL '30 days',
    NOW(),
    INTERVAL '1 minute'
) AS t
WHERE s.sensor_type = 'power';

-- CO2データ（5台 × 30日 × 1440分 = 216,000件）
INSERT INTO metrics.measurements (sensor_id, value, quality, recorded_at)
SELECT
    s.id,
    CASE
        WHEN s.location LIKE '%会議室%' THEN
            -- 会議室: 会議時間帯で急上昇
            CASE
                WHEN EXTRACT(HOUR FROM t) IN (10, 11, 14, 15, 16) AND RANDOM() > 0.3
                THEN 800 + RANDOM() * 1000  -- 会議中
                ELSE 450 + RANDOM() * 100  -- 空き
            END
        WHEN s.location LIKE '%サーバー%' THEN
            420 + RANDOM() * 50  -- サーバールーム: 安定
        ELSE
            -- オフィス: 人の増減に応じて変動
            CASE
                WHEN EXTRACT(DOW FROM t) IN (0, 6) THEN 400 + RANDOM() * 50
                WHEN EXTRACT(HOUR FROM t) BETWEEN 9 AND 18 THEN 600 + RANDOM() * 400
                ELSE 420 + RANDOM() * 80
            END
    END,
    CASE WHEN RANDOM() > 0.003 THEN 'good' ELSE 'suspect' END,
    t
FROM metrics.sensors s
CROSS JOIN generate_series(
    NOW() - INTERVAL '30 days',
    NOW(),
    INTERVAL '1 minute'
) AS t
WHERE s.sensor_type = 'co2';

-- 心拍データ（10台 × 30日 × 1440分 = 432,000件）
-- ただし、活動時間帯のみ（6:00-23:00）
INSERT INTO metrics.measurements (sensor_id, value, quality, recorded_at)
SELECT
    s.id,
    CASE
        -- 運動時間帯（朝と夕方）
        WHEN EXTRACT(HOUR FROM t) BETWEEN 6 AND 7 THEN 100 + RANDOM() * 60  -- 朝の運動
        WHEN EXTRACT(HOUR FROM t) BETWEEN 18 AND 19 THEN 110 + RANDOM() * 70  -- 夕方の運動
        -- 仕事時間帯
        WHEN EXTRACT(HOUR FROM t) BETWEEN 9 AND 12 THEN 70 + RANDOM() * 20
        WHEN EXTRACT(HOUR FROM t) BETWEEN 13 AND 17 THEN 72 + RANDOM() * 25
        -- 休憩・食事
        WHEN EXTRACT(HOUR FROM t) = 12 THEN 65 + RANDOM() * 15
        -- 夜
        WHEN EXTRACT(HOUR FROM t) BETWEEN 20 AND 22 THEN 60 + RANDOM() * 15
        -- 睡眠時間
        ELSE 55 + RANDOM() * 10
    END + (s.id - 26) * 3,  -- ユーザーごとの基礎心拍数の違い
    'good',
    t
FROM metrics.sensors s
CROSS JOIN generate_series(
    NOW() - INTERVAL '30 days',
    NOW(),
    INTERVAL '1 minute'
) AS t
WHERE s.sensor_type = 'heartRate'
  AND EXTRACT(HOUR FROM t) BETWEEN 6 AND 23;

-- =============================================================================
-- ユーザーアクティビティデータ（過去30日分）
-- =============================================================================

-- 商品閲覧アクティビティ
INSERT INTO metrics.user_activities (user_id, activity_type, product_id, store_id, metadata, created_at)
SELECT
    'user' || LPAD((RANDOM() * 1000)::INT::TEXT, 4, '0'),
    'view_product',
    (RANDOM() * 1000)::INT + 1,
    NULL,
    jsonb_build_object(
        'source', (ARRAY['web', 'mobile_app', 'tablet'])[1 + (RANDOM() * 2)::INT],
        'duration_sec', (RANDOM() * 120)::INT + 5,
        'referrer', (ARRAY['search', 'category', 'recommendation', 'direct'])[1 + (RANDOM() * 3)::INT]
    ),
    NOW() - (RANDOM() * 30)::INT * INTERVAL '1 day'
        - (RANDOM() * 24)::INT * INTERVAL '1 hour'
        - (RANDOM() * 60)::INT * INTERVAL '1 minute'
FROM generate_series(1, 50000);

-- 商品購入アクティビティ
INSERT INTO metrics.user_activities (user_id, activity_type, product_id, store_id, metadata, created_at)
SELECT
    'user' || LPAD((RANDOM() * 1000)::INT::TEXT, 4, '0'),
    'purchase',
    (RANDOM() * 1000)::INT + 1,
    (RANDOM() * 200)::INT + 1,
    jsonb_build_object(
        'quantity', (RANDOM() * 3)::INT + 1,
        'payment_method', (ARRAY['credit_card', 'mobile_pay', 'cash', 'points'])[1 + (RANDOM() * 3)::INT],
        'is_member', RANDOM() > 0.3
    ),
    NOW() - (RANDOM() * 30)::INT * INTERVAL '1 day'
        - (RANDOM() * 24)::INT * INTERVAL '1 hour'
        - (RANDOM() * 60)::INT * INTERVAL '1 minute'
FROM generate_series(1, 10000);

-- 店舗来店アクティビティ
INSERT INTO metrics.user_activities (user_id, activity_type, product_id, store_id, metadata, created_at)
SELECT
    'user' || LPAD((RANDOM() * 1000)::INT::TEXT, 4, '0'),
    'store_visit',
    NULL,
    (RANDOM() * 200)::INT + 1,
    jsonb_build_object(
        'visit_duration_min', (RANDOM() * 60)::INT + 5,
        'items_purchased', (RANDOM() * 5)::INT
    ),
    NOW() - (RANDOM() * 30)::INT * INTERVAL '1 day'
        - (RANDOM() * 24)::INT * INTERVAL '1 hour'
        - (RANDOM() * 60)::INT * INTERVAL '1 minute'
FROM generate_series(1, 20000);

-- 商品検索アクティビティ
INSERT INTO metrics.user_activities (user_id, activity_type, product_id, store_id, metadata, created_at)
SELECT
    'user' || LPAD((RANDOM() * 1000)::INT::TEXT, 4, '0'),
    'search',
    NULL,
    NULL,
    jsonb_build_object(
        'query', (ARRAY['コーヒー', 'ラテ', 'フラペチーノ', 'スコーン', '抹茶', 'チョコ', 'キャラメル', 'モカ', 'アイス', 'ホット'])[1 + (RANDOM() * 9)::INT],
        'results_count', (RANDOM() * 50)::INT + 1,
        'clicked_position', CASE WHEN RANDOM() > 0.3 THEN (RANDOM() * 10)::INT + 1 ELSE NULL END
    ),
    NOW() - (RANDOM() * 30)::INT * INTERVAL '1 day'
        - (RANDOM() * 24)::INT * INTERVAL '1 hour'
        - (RANDOM() * 60)::INT * INTERVAL '1 minute'
FROM generate_series(1, 30000);

-- =============================================================================
-- 統計情報
-- =============================================================================

SELECT
    sensor_type,
    COUNT(*) as sensor_count
FROM metrics.sensors
GROUP BY sensor_type
ORDER BY sensor_count DESC;

SELECT 'Total sensors: ' || COUNT(*) as summary FROM metrics.sensors;

SELECT
    s.sensor_type,
    COUNT(m.*) as measurement_count,
    ROUND(AVG(m.value)::NUMERIC, 2) as avg_value,
    ROUND(MIN(m.value)::NUMERIC, 2) as min_value,
    ROUND(MAX(m.value)::NUMERIC, 2) as max_value
FROM metrics.sensors s
LEFT JOIN metrics.measurements m ON s.id = m.sensor_id
GROUP BY s.sensor_type
ORDER BY measurement_count DESC;

SELECT 'Total measurements: ' || COUNT(*) as summary FROM metrics.measurements;

SELECT
    activity_type,
    COUNT(*) as count
FROM metrics.user_activities
GROUP BY activity_type
ORDER BY count DESC;

SELECT 'Total user activities: ' || COUNT(*) as summary FROM metrics.user_activities;
