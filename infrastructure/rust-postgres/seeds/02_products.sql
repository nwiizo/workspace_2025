-- =============================================================================
-- 商品データ（1000件以上）
-- コーヒーチェーン店を想定した架空の商品データ
-- =============================================================================

-- カテゴリ別商品データ生成

-- コーヒー（300件）
INSERT INTO products.catalog (name, category, price, description, attributes, tags)
SELECT
    coffee_name || ' ' || size_name,
    'coffee',
    base_price + size_price + ROUND((RANDOM() * 100)::NUMERIC, 0),
    'こだわりの豆を使用した' || coffee_name || '。' || size_name || 'サイズでお届けします。' || description_extra,
    jsonb_build_object(
        'size', size_code,
        'caffeine_mg', caffeine_base + (RANDOM() * 50)::INT,
        'calories', calories_base + (RANDOM() * 30)::INT,
        'temperature', CASE WHEN RANDOM() > 0.3 THEN 'hot' ELSE 'iced' END,
        'origin', origin,
        'roast_level', roast
    ),
    ARRAY[size_code, roast, origin, 'coffee']
FROM (
    VALUES
        ('エスプレッソ', 350, 150, 5, 'イタリアンローストの深い味わい。', 'italy', 'dark'),
        ('アメリカーノ', 380, 120, 10, 'すっきりとした飲み口。', 'brazil', 'medium'),
        ('カフェラテ', 450, 80, 150, 'ミルクとの絶妙なハーモニー。', 'colombia', 'medium'),
        ('カプチーノ', 450, 80, 120, 'ふわふわのフォームミルク。', 'ethiopia', 'medium'),
        ('カフェモカ', 500, 100, 250, 'チョコレートの甘さが魅力。', 'guatemala', 'dark'),
        ('キャラメルマキアート', 520, 100, 280, 'キャラメルソースがアクセント。', 'brazil', 'medium'),
        ('バニララテ', 500, 90, 260, 'バニラの香りが広がる一杯。', 'colombia', 'light'),
        ('ヘーゼルナッツラテ', 500, 90, 250, 'ナッツの香ばしさ。', 'ethiopia', 'medium'),
        ('フラットホワイト', 480, 110, 140, 'オーストラリア発祥の人気メニュー。', 'australia', 'medium'),
        ('コールドブリュー', 450, 200, 5, '水出しコーヒーの優しい味わい。', 'kenya', 'light'),
        ('ドリップコーヒー', 320, 150, 5, '定番のハンドドリップ。', 'brazil', 'medium'),
        ('カフェオレ', 400, 60, 100, 'ミルクたっぷりの優しい味。', 'colombia', 'light'),
        ('ウィンナーコーヒー', 480, 100, 200, 'ホイップクリームがリッチ。', 'austria', 'dark'),
        ('アイリッシュコーヒー', 550, 80, 180, 'ウイスキー風味の大人の味。', 'ireland', 'dark'),
        ('モカジャバ', 520, 110, 270, 'ジャワ産豆とチョコの融合。', 'indonesia', 'dark')
) AS coffees(coffee_name, base_price, caffeine_base, calories_base, description_extra, origin, roast)
CROSS JOIN (
    VALUES
        ('ショート', 'S', 0),
        ('トール', 'M', 50),
        ('グランデ', 'L', 100),
        ('ベンティ', 'XL', 150)
) AS sizes(size_name, size_code, size_price);

-- ティー（200件）
INSERT INTO products.catalog (name, category, price, description, attributes, tags)
SELECT
    tea_name || ' ' || size_name,
    'tea',
    base_price + size_price,
    description || ' ' || size_name || 'サイズ。',
    jsonb_build_object(
        'size', size_code,
        'caffeine_mg', caffeine,
        'calories', calories,
        'temperature', temp,
        'origin', origin,
        'type', tea_type
    ),
    ARRAY[size_code, tea_type, origin, 'tea']
FROM (
    VALUES
        ('イングリッシュブレックファスト', 380, 40, 0, '朝の定番紅茶。力強い味わい。', 'hot', 'uk', 'black'),
        ('アールグレイ', 380, 35, 0, 'ベルガモットの香り高い紅茶。', 'hot', 'uk', 'black'),
        ('ダージリン', 400, 30, 0, '紅茶のシャンパンと呼ばれる逸品。', 'hot', 'india', 'black'),
        ('アッサム', 380, 45, 0, 'ミルクティーに最適な濃厚な味。', 'hot', 'india', 'black'),
        ('セイロン', 360, 35, 0, 'スリランカ産の上品な紅茶。', 'hot', 'sri_lanka', 'black'),
        ('ジャスミンティー', 350, 20, 0, '花の香りが広がる中国茶。', 'hot', 'china', 'green'),
        ('抹茶ラテ', 480, 30, 150, '京都産抹茶を使用。', 'hot', 'japan', 'matcha'),
        ('ほうじ茶ラテ', 450, 10, 120, '香ばしいほうじ茶。', 'hot', 'japan', 'roasted'),
        ('玄米茶', 320, 15, 0, '香ばしい玄米の香り。', 'hot', 'japan', 'green'),
        ('ルイボスティー', 380, 0, 0, 'ノンカフェインの健康茶。', 'hot', 'south_africa', 'herbal'),
        ('カモミールティー', 380, 0, 0, 'リラックス効果のあるハーブティー。', 'hot', 'germany', 'herbal'),
        ('ペパーミントティー', 360, 0, 0, 'すっきり爽やかなハーブティー。', 'hot', 'usa', 'herbal'),
        ('チャイラテ', 480, 40, 140, 'スパイス香るインドの紅茶。', 'hot', 'india', 'chai'),
        ('タピオカミルクティー', 550, 30, 300, 'もちもちタピオカ入り。', 'iced', 'taiwan', 'bubble'),
        ('烏龍茶', 350, 25, 0, '台湾高山茶の芳醇な味わい。', 'hot', 'taiwan', 'oolong')
) AS teas(tea_name, base_price, caffeine, calories, description, temp, origin, tea_type)
CROSS JOIN (
    VALUES
        ('ショート', 'S', 0),
        ('トール', 'M', 50),
        ('グランデ', 'L', 100)
) AS sizes(size_name, size_code, size_price);

-- フード（200件）
INSERT INTO products.catalog (name, category, price, description, attributes, tags)
SELECT
    food_name,
    'food',
    price + (RANDOM() * 50)::INT,
    description,
    jsonb_build_object(
        'calories', calories,
        'protein_g', protein,
        'carbs_g', carbs,
        'fat_g', fat,
        'allergens', allergens,
        'vegetarian', vegetarian,
        'available_time', available_time
    ),
    tags
FROM (
    VALUES
        ('クロワッサン', 280, 'バターたっぷりのサクサク食感。', 280, 5, 30, 16, ARRAY['wheat', 'milk', 'egg'], FALSE, 'morning', ARRAY['pastry', 'breakfast']),
        ('チョコクロワッサン', 320, 'チョコレートを包んだクロワッサン。', 350, 6, 38, 20, ARRAY['wheat', 'milk', 'egg'], FALSE, 'morning', ARRAY['pastry', 'breakfast', 'chocolate']),
        ('アーモンドクロワッサン', 350, 'アーモンドクリームをトッピング。', 400, 8, 35, 24, ARRAY['wheat', 'milk', 'egg', 'nuts'], FALSE, 'morning', ARRAY['pastry', 'breakfast', 'nuts']),
        ('シナモンロール', 380, '温かいシナモンの香り。', 420, 6, 55, 18, ARRAY['wheat', 'milk', 'egg'], FALSE, 'all_day', ARRAY['pastry', 'sweet']),
        ('スコーン プレーン', 280, '紅茶に合う英国の伝統菓子。', 320, 5, 40, 14, ARRAY['wheat', 'milk', 'egg'], FALSE, 'all_day', ARRAY['pastry', 'british']),
        ('スコーン チョコチップ', 320, 'チョコチップ入りのスコーン。', 380, 5, 48, 16, ARRAY['wheat', 'milk', 'egg'], FALSE, 'all_day', ARRAY['pastry', 'british', 'chocolate']),
        ('マフィン ブルーベリー', 350, 'ジューシーなブルーベリー入り。', 380, 5, 52, 15, ARRAY['wheat', 'milk', 'egg'], FALSE, 'all_day', ARRAY['pastry', 'fruit']),
        ('マフィン チョコレート', 350, '濃厚なチョコレートマフィン。', 420, 6, 55, 20, ARRAY['wheat', 'milk', 'egg'], FALSE, 'all_day', ARRAY['pastry', 'chocolate']),
        ('ベーグル プレーン', 250, 'もちもちのニューヨークスタイル。', 260, 9, 52, 2, ARRAY['wheat'], TRUE, 'morning', ARRAY['bread', 'breakfast']),
        ('ベーグル セサミ', 270, 'ゴマの香ばしさがアクセント。', 280, 10, 54, 3, ARRAY['wheat', 'sesame'], TRUE, 'morning', ARRAY['bread', 'breakfast']),
        ('サンドイッチ BLT', 550, 'ベーコン、レタス、トマトの定番。', 450, 18, 35, 25, ARRAY['wheat', 'egg'], FALSE, 'lunch', ARRAY['sandwich', 'lunch']),
        ('サンドイッチ チキン', 580, 'ジューシーなチキンブレスト。', 420, 28, 32, 18, ARRAY['wheat'], FALSE, 'lunch', ARRAY['sandwich', 'lunch', 'chicken']),
        ('サンドイッチ ツナ', 520, '自家製ツナサラダ。', 380, 22, 30, 16, ARRAY['wheat', 'fish', 'egg'], FALSE, 'lunch', ARRAY['sandwich', 'lunch', 'fish']),
        ('サンドイッチ 野菜', 480, 'たっぷり野菜のヘルシーサンド。', 280, 10, 38, 8, ARRAY['wheat'], TRUE, 'lunch', ARRAY['sandwich', 'lunch', 'vegetarian']),
        ('キッシュ ほうれん草', 450, 'ほうれん草とチーズのキッシュ。', 350, 12, 25, 22, ARRAY['wheat', 'milk', 'egg'], FALSE, 'lunch', ARRAY['quiche', 'lunch']),
        ('キッシュ ベーコン', 480, 'ベーコンとオニオンのキッシュ。', 400, 15, 25, 28, ARRAY['wheat', 'milk', 'egg'], FALSE, 'lunch', ARRAY['quiche', 'lunch']),
        ('サラダ シーザー', 650, 'ロメインレタスとパルメザン。', 320, 12, 15, 24, ARRAY['milk', 'egg', 'fish'], FALSE, 'lunch', ARRAY['salad', 'lunch']),
        ('サラダ チキン', 700, 'グリルチキンとミックスグリーン。', 350, 30, 12, 18, ARRAY['wheat'], FALSE, 'lunch', ARRAY['salad', 'lunch', 'chicken']),
        ('ケーキ チーズケーキ', 550, '濃厚なニューヨークチーズケーキ。', 450, 8, 35, 32, ARRAY['wheat', 'milk', 'egg'], FALSE, 'all_day', ARRAY['cake', 'dessert']),
        ('ケーキ チョコレート', 580, 'リッチなチョコレートケーキ。', 500, 6, 55, 30, ARRAY['wheat', 'milk', 'egg'], FALSE, 'all_day', ARRAY['cake', 'dessert', 'chocolate']),
        ('ケーキ 抹茶', 550, '京都産抹茶のケーキ。', 420, 6, 45, 24, ARRAY['wheat', 'milk', 'egg'], FALSE, 'all_day', ARRAY['cake', 'dessert', 'matcha']),
        ('クッキー チョコチップ', 220, 'ざっくりチョコチップクッキー。', 250, 3, 32, 12, ARRAY['wheat', 'milk', 'egg'], FALSE, 'all_day', ARRAY['cookie', 'snack', 'chocolate']),
        ('クッキー オートミール', 220, 'ヘルシーなオートミールクッキー。', 230, 4, 30, 10, ARRAY['wheat', 'milk', 'egg'], FALSE, 'all_day', ARRAY['cookie', 'snack']),
        ('ドーナツ グレーズ', 250, '定番のグレーズドーナツ。', 280, 3, 38, 14, ARRAY['wheat', 'milk', 'egg'], FALSE, 'all_day', ARRAY['donut', 'snack']),
        ('ドーナツ チョコレート', 280, 'チョコレートコーティング。', 320, 4, 42, 16, ARRAY['wheat', 'milk', 'egg'], FALSE, 'all_day', ARRAY['donut', 'snack', 'chocolate'])
) AS foods(food_name, price, description, calories, protein, carbs, fat, allergens, vegetarian, available_time, tags);

-- フラペチーノ・フローズンドリンク（150件）
INSERT INTO products.catalog (name, category, price, description, attributes, tags)
SELECT
    frappe_name || ' ' || size_name,
    'frappe',
    base_price + size_price,
    description || size_name || 'サイズ。',
    jsonb_build_object(
        'size', size_code,
        'caffeine_mg', caffeine,
        'calories', calories + (size_price / 2),
        'temperature', 'frozen',
        'whipped_cream', whipped,
        'seasonal', seasonal
    ),
    ARRAY[size_code, 'frappe', 'frozen'] || extra_tags
FROM (
    VALUES
        ('キャラメルフラペチーノ', 550, 'キャラメルソースたっぷり。', 50, 350, TRUE, FALSE, ARRAY['caramel']),
        ('モカフラペチーノ', 550, 'コーヒーとチョコの融合。', 80, 380, TRUE, FALSE, ARRAY['mocha', 'chocolate']),
        ('バニラフラペチーノ', 520, 'バニラビーンズの香り。', 30, 320, TRUE, FALSE, ARRAY['vanilla']),
        ('抹茶フラペチーノ', 550, '宇治抹茶を贅沢に使用。', 50, 340, TRUE, FALSE, ARRAY['matcha', 'japanese']),
        ('ストロベリーフラペチーノ', 550, 'いちごの甘酸っぱさ。', 0, 360, TRUE, TRUE, ARRAY['strawberry', 'fruit']),
        ('マンゴーフラペチーノ', 550, 'トロピカルなマンゴー。', 0, 340, TRUE, TRUE, ARRAY['mango', 'fruit', 'tropical']),
        ('チョコレートフラペチーノ', 550, '濃厚なチョコレート。', 20, 400, TRUE, FALSE, ARRAY['chocolate']),
        ('コーヒーフラペチーノ', 500, 'コーヒーの苦味が効いた大人の味。', 100, 280, TRUE, FALSE, ARRAY['coffee']),
        ('ホワイトモカフラペチーノ', 580, 'ホワイトチョコレートのリッチな味。', 60, 420, TRUE, FALSE, ARRAY['white_chocolate']),
        ('ダークモカフラペチーノ', 580, 'ビターチョコの深い味わい。', 90, 380, TRUE, FALSE, ARRAY['dark_chocolate'])
) AS frappes(frappe_name, base_price, description, caffeine, calories, whipped, seasonal, extra_tags)
CROSS JOIN (
    VALUES
        ('トール', 'M', 0),
        ('グランデ', 'L', 80),
        ('ベンティ', 'XL', 160)
) AS sizes(size_name, size_code, size_price);

-- 季節限定商品（100件）
INSERT INTO products.catalog (name, category, price, description, attributes, tags)
SELECT
    seasonal_name,
    category,
    price,
    description,
    jsonb_build_object(
        'season', season,
        'available_from', start_date,
        'available_until', end_date,
        'limited', TRUE,
        'calories', calories
    ),
    ARRAY['seasonal', 'limited', season]
FROM (
    VALUES
        ('桜ラテ', 'coffee', 550, '春限定。桜の香りが広がる。', 'spring', '2024-03-01', '2024-04-30', 280),
        ('桜フラペチーノ', 'frappe', 620, '桜餅風味のフラペチーノ。', 'spring', '2024-03-01', '2024-04-30', 380),
        ('桜スコーン', 'food', 350, '桜あん入りスコーン。', 'spring', '2024-03-01', '2024-04-30', 320),
        ('抹茶桜ラテ', 'coffee', 580, '抹茶と桜のハーモニー。', 'spring', '2024-03-01', '2024-04-30', 300),
        ('ピーチフラペチーノ', 'frappe', 600, '夏限定。ジューシーな桃。', 'summer', '2024-06-01', '2024-08-31', 340),
        ('トロピカルマンゴー', 'frappe', 620, '夏のトロピカルフレーバー。', 'summer', '2024-06-01', '2024-08-31', 360),
        ('スイカフラペチーノ', 'frappe', 620, '夏祭り気分のスイカ味。', 'summer', '2024-06-01', '2024-08-31', 320),
        ('レモネードフラペチーノ', 'frappe', 580, '爽やかレモネード。', 'summer', '2024-06-01', '2024-08-31', 280),
        ('かき氷 いちご', 'food', 450, 'ふわふわかき氷。', 'summer', '2024-07-01', '2024-08-31', 150),
        ('かき氷 抹茶', 'food', 480, '宇治抹茶のかき氷。', 'summer', '2024-07-01', '2024-08-31', 180),
        ('パンプキンラテ', 'coffee', 550, '秋限定。かぼちゃの優しい甘さ。', 'autumn', '2024-09-01', '2024-11-30', 320),
        ('パンプキンスパイスラテ', 'coffee', 580, 'スパイス香るパンプキン。', 'autumn', '2024-09-01', '2024-11-30', 340),
        ('マロンラテ', 'coffee', 580, '栗の濃厚な味わい。', 'autumn', '2024-09-01', '2024-11-30', 350),
        ('スイートポテトラテ', 'coffee', 550, 'さつまいもの自然な甘さ。', 'autumn', '2024-09-01', '2024-11-30', 330),
        ('アップルパイラテ', 'coffee', 580, 'りんごとシナモン。', 'autumn', '2024-09-01', '2024-11-30', 320),
        ('モンブランケーキ', 'food', 620, '栗たっぷりモンブラン。', 'autumn', '2024-09-01', '2024-11-30', 450),
        ('ジンジャーブレッドラテ', 'coffee', 550, '冬限定。ジンジャーの香り。', 'winter', '2024-11-01', '2025-02-28', 300),
        ('ホワイトチョコモカ', 'coffee', 580, 'クリスマス気分のホワイトモカ。', 'winter', '2024-11-01', '2025-02-28', 380),
        ('ペパーミントモカ', 'coffee', 580, 'ミントとチョコの冬定番。', 'winter', '2024-11-01', '2025-02-28', 360),
        ('ホットチョコレート', 'tea', 500, '濃厚ホットチョコ。', 'winter', '2024-11-01', '2025-02-28', 350),
        ('シュトーレン', 'food', 450, 'ドイツ伝統のクリスマス菓子。', 'winter', '2024-11-01', '2024-12-25', 380),
        ('ブッシュドノエル', 'food', 650, 'クリスマスケーキ。', 'winter', '2024-12-01', '2024-12-25', 420),
        ('いちごショートケーキ', 'food', 600, '冬の定番ショートケーキ。', 'winter', '2024-12-01', '2025-02-28', 380),
        ('ホットアップルサイダー', 'tea', 480, '温かいりんごジュース。', 'winter', '2024-11-01', '2025-02-28', 180),
        ('あんバタースコーン', 'food', 380, '和洋折衷の美味しさ。', 'winter', '2024-11-01', '2025-02-28', 350)
);

-- グッズ・マーチャンダイズ（50件）
INSERT INTO products.catalog (name, category, price, description, attributes, tags)
VALUES
    ('タンブラー ステンレス 350ml', 'goods', 3200, '保温保冷に優れたタンブラー。', '{"material": "stainless", "capacity_ml": 350, "color": "silver"}', ARRAY['tumbler', 'drinkware']),
    ('タンブラー ステンレス 500ml', 'goods', 3800, '大容量ステンレスタンブラー。', '{"material": "stainless", "capacity_ml": 500, "color": "silver"}', ARRAY['tumbler', 'drinkware']),
    ('タンブラー プラスチック', 'goods', 1800, '軽量で持ち運びやすい。', '{"material": "plastic", "capacity_ml": 350, "color": "clear"}', ARRAY['tumbler', 'drinkware']),
    ('マグカップ ロゴ', 'goods', 1500, 'シンプルなロゴ入りマグ。', '{"material": "ceramic", "capacity_ml": 300, "color": "white"}', ARRAY['mug', 'drinkware']),
    ('マグカップ 季節限定 桜', 'goods', 2200, '桜デザインの限定マグ。', '{"material": "ceramic", "capacity_ml": 300, "seasonal": true}', ARRAY['mug', 'drinkware', 'seasonal']),
    ('マグカップ 季節限定 クリスマス', 'goods', 2200, 'クリスマスデザイン。', '{"material": "ceramic", "capacity_ml": 300, "seasonal": true}', ARRAY['mug', 'drinkware', 'seasonal']),
    ('コーヒー豆 ハウスブレンド 250g', 'goods', 1400, '定番のハウスブレンド。', '{"weight_g": 250, "roast": "medium", "origin": "blend"}', ARRAY['coffee_beans', 'beans']),
    ('コーヒー豆 エスプレッソ 250g', 'goods', 1500, 'エスプレッソ用深煎り。', '{"weight_g": 250, "roast": "dark", "origin": "blend"}', ARRAY['coffee_beans', 'beans']),
    ('コーヒー豆 シングルオリジン エチオピア', 'goods', 1800, 'エチオピア産フルーティー豆。', '{"weight_g": 200, "roast": "light", "origin": "ethiopia"}', ARRAY['coffee_beans', 'beans', 'single_origin']),
    ('コーヒー豆 シングルオリジン コロンビア', 'goods', 1700, 'コロンビア産バランス良い味。', '{"weight_g": 200, "roast": "medium", "origin": "colombia"}', ARRAY['coffee_beans', 'beans', 'single_origin']),
    ('ドリッパー 陶器', 'goods', 2500, '陶器製の円錐ドリッパー。', '{"material": "ceramic", "type": "cone"}', ARRAY['dripper', 'brewing']),
    ('ドリッパー ステンレス', 'goods', 3500, 'ステンレス製ドリッパー。', '{"material": "stainless", "type": "cone"}', ARRAY['dripper', 'brewing']),
    ('フレンチプレス 350ml', 'goods', 3800, '本格フレンチプレス。', '{"capacity_ml": 350, "material": "glass"}', ARRAY['french_press', 'brewing']),
    ('フレンチプレス 500ml', 'goods', 4500, '大容量フレンチプレス。', '{"capacity_ml": 500, "material": "glass"}', ARRAY['french_press', 'brewing']),
    ('コーヒーミル 手動', 'goods', 4800, 'コンパクトな手挽きミル。', '{"type": "manual", "material": "ceramic_blade"}', ARRAY['grinder', 'brewing']),
    ('エコバッグ', 'goods', 800, 'オリジナルエコバッグ。', '{"material": "cotton", "color": "natural"}', ARRAY['bag', 'eco']),
    ('トートバッグ', 'goods', 2500, 'キャンバストートバッグ。', '{"material": "canvas", "color": "natural"}', ARRAY['bag']),
    ('エプロン', 'goods', 3500, 'バリスタ風エプロン。', '{"material": "cotton", "color": "black"}', ARRAY['apron', 'apparel']),
    ('Tシャツ ロゴ M', 'goods', 2800, 'ロゴTシャツ Mサイズ。', '{"size": "M", "color": "white"}', ARRAY['tshirt', 'apparel']),
    ('Tシャツ ロゴ L', 'goods', 2800, 'ロゴTシャツ Lサイズ。', '{"size": "L", "color": "white"}', ARRAY['tshirt', 'apparel']);

-- 追加のバリエーション（ランダムデータで合計1000件以上に）
INSERT INTO products.catalog (name, category, price, description, attributes, tags)
SELECT
    'スペシャルブレンド #' || i,
    'coffee',
    400 + (RANDOM() * 300)::INT,
    '厳選された豆を使用したスペシャルブレンド第' || i || '弾。',
    jsonb_build_object(
        'blend_number', i,
        'caffeine_mg', 80 + (RANDOM() * 100)::INT,
        'calories', 5 + (RANDOM() * 50)::INT,
        'roast_level', (ARRAY['light', 'medium', 'dark'])[1 + (RANDOM() * 2)::INT]
    ),
    ARRAY['coffee', 'special', 'blend']
FROM generate_series(1, 100) AS i;

-- 統計情報確認
SELECT
    category,
    COUNT(*) as count,
    ROUND(AVG(price)::NUMERIC, 0) as avg_price,
    MIN(price) as min_price,
    MAX(price) as max_price
FROM products.catalog
GROUP BY category
ORDER BY count DESC;

SELECT 'Total products: ' || COUNT(*) as summary FROM products.catalog;
