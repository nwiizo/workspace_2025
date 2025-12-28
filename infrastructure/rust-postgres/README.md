# Just Use Postgres! 技術検証

「Just Use Postgres!」書籍で紹介されている各機能を、DockerとRustで実際に検証するプロジェクト。

## クイックスタート（フルスタック）

```bash
# 1. PostgreSQL を起動
docker compose up -d

# 2. サンプルデータを投入（初回のみ）
./seeds/seed.sh

# 3. API サーバーを起動（別ターミナル）
cargo run --bin api-server

# 4. フロントエンドを起動（別ターミナル）
cd frontend
npm install
npm run dev

# 5. ブラウザで開く
open http://localhost:3000
```

## サンプルデータ投入

`seeds/` ディレクトリに大量の架空データを投入するスクリプトがあります。

### データ量

| テーブル | 件数 | 内容 |
|----------|------|------|
| products.catalog | 1,000+ | コーヒー、ティー、フード、フラペチーノ、季節限定、グッズ |
| stores.locations | 200+ | 日本全国の店舗（東京、大阪、京都、福岡、北海道など） |
| metrics.sensors | 50 | 温度、湿度、CO2、電力、心拍センサー |
| metrics.measurements | 1,500,000+ | 30日分の1分間隔時系列データ |
| metrics.user_activities | 110,000+ | 商品閲覧、購入、検索、来店アクティビティ |

### 使い方

```bash
# 全データ投入（初回推奨）
./seeds/seed.sh

# スキーマのみ作成
./seeds/seed.sh --schema

# 各データを個別に投入
./seeds/seed.sh --products   # 商品データのみ
./seeds/seed.sh --stores     # 店舗データのみ
./seeds/seed.sh --metrics    # メトリクスデータのみ（時間がかかります）

# データベースをリセットして全投入
./seeds/seed.sh --reset

# カスタム接続設定
PGHOST=db.example.com PGPASSWORD=secret ./seeds/seed.sh
```

### 環境変数

| 変数 | デフォルト | 説明 |
|------|-----------|------|
| PGHOST | localhost | PostgreSQL ホスト |
| PGPORT | 5432 | PostgreSQL ポート |
| PGUSER | postgres | PostgreSQL ユーザー |
| PGPASSWORD | password | PostgreSQL パスワード |
| PGDATABASE | coffee_chain | データベース名 |

### 画面構成

| 画面 | URL | PostgreSQL機能 |
|------|-----|----------------|
| ホーム | `/` | アーキテクチャ概要 |
| 商品検索 | `/products` | 全文検索 (tsvector) + JSONB |
| ダッシュボード | `/dashboard` | TimescaleDB (time_bucket) |
| 店舗検索 | `/stores` | PostGIS (ST_DWithin) |

### API エンドポイント

| Method | Path | 説明 | PostgreSQL機能 |
|--------|------|------|----------------|
| GET | `/api/health` | ヘルスチェック | - |
| GET | `/api/products/search` | 商品検索 | 全文検索 + JSONB |
| GET | `/api/products/{id}` | 商品詳細 | - |
| GET | `/api/products/{id}/similar` | 類似商品 | pgvector |
| GET | `/api/products/categories` | カテゴリ一覧 | - |
| GET | `/api/metrics` | メトリクス取得 | TimescaleDB |
| GET | `/api/sensors` | センサー一覧 | - |
| GET | `/api/stores` | 店舗一覧 | - |
| GET | `/api/stores/nearby` | 近くの店舗 | PostGIS |

---

## 検証内容

### Chapter 1: Meeting Postgres

`generate_series`を使ったモックデータ生成。外部ツールなしでテストデータを作成できる。

```sql
-- 株取引のモックデータを生成
SELECT
    n AS id,
    (random() * 10 + 1)::INT AS buyer_id,
    (ARRAY['AAPL', 'GOOGL', 'MSFT', 'AMZN', 'META'])[floor(random() * 5 + 1)::INT] AS symbol,
    (random() * 100 + 1)::INT AS order_quantity,
    round((random() * 500 + 100)::NUMERIC, 2) AS bid_price
FROM generate_series(1, 20) AS n;
```

**実装時の気づき**:
- `generate_series`は日付生成にも使える: `generate_series('2025-01-01'::date, '2025-12-31', '1 day')`
- `random()`は毎回異なる値を返すため、再現可能なテストには`setseed()`を使う
- 配列のインデックスは1始まり（0始まりではない）

**応用例**:
- カレンダーテーブルの生成（休日判定用）
- 時系列データの欠損値補完（LEFT JOINと組み合わせ）
- 負荷テスト用の大量データ生成

**限界**:
- 複雑なリレーションを持つデータの生成には向かない
- 実際のデータ分布を再現するのは困難

---

### Chapter 2: Standard RDBMS capabilities

**スキーマ構造**: Database → Schema → Table の三層構造でマイクロサービス的なモジュール分離。

```sql
-- スキーマ間のJOINも可能
SELECT c.name, p.name AS product, oi.quantity
FROM customers.accounts c
JOIN sales.orders o ON c.id = o.customer_id
JOIN sales.order_items oi ON o.id = oi.order_id
JOIN products.catalog p ON oi.product_id = p.id;
```

**データベース関数**: PL/pgSQLでカート操作をアトミックに実行。

```sql
-- カートに商品を追加する関数
CREATE OR REPLACE FUNCTION sales.add_to_cart(
    p_customer_id INT,
    p_product_id INT,
    p_quantity INT
) RETURNS UUID AS $$
DECLARE
    v_order_id UUID;
    v_price NUMERIC;
BEGIN
    -- 既存のカートを取得または作成
    SELECT id INTO v_order_id FROM sales.orders
    WHERE customer_id = p_customer_id AND status = 'cart'
    LIMIT 1;

    IF v_order_id IS NULL THEN
        INSERT INTO sales.orders (customer_id, status)
        VALUES (p_customer_id, 'cart')
        RETURNING id INTO v_order_id;
    END IF;

    -- 商品価格を取得
    SELECT price INTO v_price FROM products.catalog WHERE id = p_product_id;

    -- カートに追加（既存なら数量を加算）
    INSERT INTO sales.order_items (order_id, product_id, quantity, price)
    VALUES (v_order_id, p_product_id, p_quantity, v_price)
    ON CONFLICT (order_id, product_id) DO UPDATE
    SET quantity = order_items.quantity + EXCLUDED.quantity;

    RETURN v_order_id;
END;
$$ LANGUAGE plpgsql;
```

**トリガー**: 注文合計の自動更新。

```sql
-- order_itemsの変更時にorders.total_amountを自動更新
CREATE OR REPLACE FUNCTION sales.update_order_total()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE sales.orders
    SET total_amount = (
        SELECT COALESCE(SUM(quantity * price), 0)
        FROM sales.order_items WHERE order_id = NEW.order_id
    )
    WHERE id = NEW.order_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_order_total
AFTER INSERT OR UPDATE OR DELETE ON sales.order_items
FOR EACH ROW EXECUTE FUNCTION sales.update_order_total();
```

**実装時の気づき**:
- PL/pgSQL関数内で`SELECT ... INTO`を使う際、結果が0行だと変数は`NULL`になる（エラーにはならない）
- `ON CONFLICT DO UPDATE`でUPSERTを実現できるが、`EXCLUDED`で新しい値を参照する
- トリガーの`NEW`はINSERT/UPDATEで使用可能、`OLD`はUPDATE/DELETEで使用可能
- DELETEトリガーでは`NEW`がNULLになるため、`COALESCE(NEW.order_id, OLD.order_id)`のような対応が必要

**応用例**:
- 監査ログの自動記録（変更前後の値をJSONBで保存）
- 在庫の自動減算と在庫切れチェック
- キャッシュテーブルの自動更新

**限界**:
- トリガーが増えると処理の流れが追いにくくなる（「どこで更新されている？」問題）
- 大量の行を更新する場合、行ごとにトリガーが発火してパフォーマンス低下
- デバッグが困難（`RAISE NOTICE`でログを出力するしかない）

---

### Chapter 3: Modern SQL

**CTE (Common Table Expressions)**: Data-modifying CTEでUPDATEの結果を即座に活用。

```sql
-- 半分未満しか再生されなかった曲を「スキップ」としてマーク
WITH skipped AS (
    UPDATE streaming.plays
    SET skipped = true
    WHERE play_duration < (
        SELECT duration / 2 FROM streaming.songs WHERE id = plays.song_id
    )
    RETURNING *
)
SELECT s.title, sk.play_duration, s.duration
FROM skipped sk
JOIN streaming.songs s ON sk.song_id = s.id;
```

**Window Functions**: カテゴリ内ランキングと累計計算。

```sql
-- カテゴリ内での価格ランキング
SELECT name, category, price,
    ROW_NUMBER() OVER (PARTITION BY category ORDER BY price DESC) AS rank
FROM products.catalog;

-- 顧客ごとの累計購入額
SELECT customer_id, product_name, amount,
    SUM(amount) OVER (PARTITION BY customer_id ORDER BY created_at) AS running_total
FROM sales.order_details;
```

**Recursive Query**: リンクリストをたどる連続再生履歴。

```sql
-- 再生ID 175から始まる連続再生シーケンス
WITH RECURSIVE play_sequence AS (
    -- 起点（非再帰項）
    SELECT id, song_id, user_id, played_after, 1 AS depth, ARRAY[id] AS path
    FROM streaming.plays WHERE id = 175

    UNION ALL

    -- 再帰部分
    SELECT p.id, p.song_id, p.user_id, p.played_after, ps.depth + 1, ps.path || p.id
    FROM streaming.plays p
    JOIN play_sequence ps ON p.played_after = ps.id
    WHERE NOT p.id = ANY(ps.path)  -- 循環検出
)
SELECT ps.depth, s.title, p.play_duration
FROM play_sequence ps
JOIN streaming.plays p ON ps.id = p.id
JOIN streaming.songs s ON p.song_id = s.id
ORDER BY ps.depth;
```

**実装時の気づき**:
- PostgresはCTEをインライン展開して最適化する（以前は「最適化の壁」だった）
- `ROW_NUMBER()` vs `RANK()` vs `DENSE_RANK()`: 同順位の扱いが異なる
- Recursive CTEの終了条件は「新しい行が生成されなくなるまで」
- 循環検出には配列で訪問済みノードを追跡する方法が有効

**応用例**:
- 組織階層の展開（部署→課→チーム）
- BOM（部品表）の展開
- グラフ構造の探索（友達の友達を検索）
- 移動平均・累積和の計算

**限界**:
- Recursive CTEは深い階層（数千レベル）でパフォーマンスが低下
- グラフDBほど柔軟なグラフ探索はできない（最短経路など）
- Window Functionsは結果セット全体をメモリに保持する必要がある場合がある

---

### Chapter 4: Indexes

**B-tree vs Hash**: 等価検索にはHashが最適。

```sql
-- Hashインデックス（等価検索専用）
CREATE INDEX idx_champion_hash ON game_player_stats USING HASH (champion_title);

-- 等価検索でHashインデックスが使われる
SELECT * FROM game_player_stats WHERE champion_title = 'Diamond';
```

**Covering Index (INCLUDE句)**: テーブルアクセスをゼロに。

```sql
-- usernameをINCLUDEしてIndex-Only Scanを実現
CREATE INDEX idx_covering ON game_player_stats (region, score DESC)
INCLUDE (username);

-- テーブルにアクセスせずインデックスのみで完結
SELECT username, score FROM game_player_stats
WHERE region = 'EU' ORDER BY score DESC LIMIT 10;
```

**Partial Index**: 必要な部分だけインデックス化。

```sql
-- カジュアルプレイヤー（50時間以下）のみインデックス化
CREATE INDEX idx_casual_players ON game_player_stats (play_time)
WHERE play_time <= '50 hours';
```

**Expression Index**: 計算結果をインデックス化。

```sql
-- 勝敗マージン（win - loss）をインデックス化
CREATE INDEX idx_perf_margin ON game_player_stats ((win_count - loss_count));

-- 計算式での検索が高速に
SELECT * FROM game_player_stats
WHERE (win_count - loss_count) BETWEEN 300 AND 500;
```

**実装時の気づき**:
- `EXPLAIN ANALYZE`の出力で`Index Scan`と`Index Only Scan`の違いを確認することが重要
- 複合インデックスの先頭カラム（leading column）がWHERE句にないとインデックスが使われない
- Partial Indexは`WHERE`句が完全に一致する場合のみ使用される
- Hashインデックスは範囲検索（`<`, `>`, `BETWEEN`）には使えない

**応用例**:
- `LOWER(email)`のExpression Indexで大文字小文字を無視した検索
- `status = 'active'`のPartial Indexでアクティブユーザーのみ高速検索
- `(created_at::date)`のExpression Indexで日付検索を高速化

**限界**:
- インデックスが多すぎると更新性能が低下（Over-Indexing問題）
- 小さなテーブルではSeq Scanの方が速い場合がある
- 統計情報が古いと最適なインデックスが選ばれない（`ANALYZE`が必要）

**実装で遭遇したエラー**:
```
-- インデックスサイズの比較で実際に確認した結果
idx_champion_hash - 696 kB
idx_covering - 416 kB
idx_region_score - 248 kB
idx_perf_margin - 120 kB
idx_casual_players - 48 kB  -- Partial Indexは最小
```

---

### Chapter 5: JSON

**JSON演算子**: `->` (JSON型), `->>` (TEXT型), `@>` (包含), `?` (キー存在)。

```sql
-- ネストしたJSONへのアクセス
SELECT
    order_data->>'type' AS drink_type,
    order_data->'extras'->>'syrup' AS syrup,
    order_data->'extras'->'shots' AS shots  -- JSON型のまま
FROM coffee_orders
WHERE order_data->'extras' ? 'syrup';  -- syrupキーが存在するか

-- 包含演算子で検索
SELECT * FROM coffee_orders
WHERE order_data @> '{"milk": "oat", "size": "large"}';
```

**JSON Path**: より柔軟なクエリ。

```sql
-- 2ショット以上の注文を検索
SELECT order_data->>'type'
FROM coffee_orders
WHERE jsonb_path_exists(order_data, '$.extras.shots ? (@ >= 2)');
```

**jsonb_set**: 部分更新。

```sql
-- シロップだけを変更（他のフィールドは保持）
UPDATE coffee_orders
SET order_data = jsonb_set(order_data, '{extras,syrup}', '"hazelnut"')
WHERE id = 1;
```

**GINインデックス**: `default` vs `jsonb_path_ops`。

```sql
-- default: @>, ?, @?, @@ をサポート
CREATE INDEX idx_default ON table USING GIN (data);

-- jsonb_path_ops: @>のみだがサイズ小・高速
CREATE INDEX idx_pathops ON table USING GIN (data jsonb_path_ops);
```

**実装時の気づき**:
- `->` と `->>` の違いを間違えると型エラーになる（特にRustからのデシリアライズ時）
- `pg_typeof()`で型を確認する際は`::TEXT`キャストが必要（Rustのtokio-postgresで）
- `@>`演算子は配列にも使える: `tags @> '["hot", "milk"]'`
- `jsonb_set`のパスが存在しない場合、新しいキーが作成される

**応用例**:
- ユーザー設定の柔軟な保存（スキーマ変更なしで新設定を追加）
- APIレスポンスのキャッシュ（構造が変わる可能性があるデータ）
- イベントソーシングのペイロード保存

**限界**:
- スキーマレスによる型安全性の低下（アプリ側でのバリデーションが必須）
- 複雑なJSONBクエリは可読性が低い
- 深くネストしたJSONの部分更新は`jsonb_set`のネストが必要で煩雑
- GINインデックスはB-treeより更新コストが高い

**実装で遭遇したエラー**:
```rust
// pg_typeof()の結果をRustで取得しようとしてエラー
// 解決策: ::TEXTでキャスト
let row = client.query_one(
    "SELECT pg_typeof(data)::TEXT AS type FROM table",
    &[]
).await?;
```

---

### Chapter 6: Full-Text Search

**tsvector/tsquery**: 全文検索の基本。

```sql
-- tsvectorの作成（重み付け）
SELECT
    setweight(to_tsvector('english', title), 'A') ||
    setweight(to_tsvector('english', description), 'B') AS lexemes
FROM movies;

-- 検索とランキング
SELECT title, ts_rank(lexemes, query) AS rank
FROM movies, plainto_tsquery('english', 'computer hacker') AS query
WHERE lexemes @@ query
ORDER BY rank DESC;
```

**ts_headline**: 検索結果のハイライト。

```sql
SELECT title,
    ts_headline('english', description, plainto_tsquery('ghost'),
        'StartSel=<b>, StopSel=</b>, MaxWords=20') AS highlighted
FROM movies
WHERE lexemes @@ plainto_tsquery('ghost');
```

**フレーズ検索**: `<->` 演算子で隣接する単語を検索。

```sql
-- "computer hacker"という連続したフレーズを検索
SELECT title FROM movies
WHERE lexemes @@ to_tsquery('english', 'computer <-> hacker');
```

**実装時の気づき**:
- `plainto_tsquery`は自然文をAND検索に変換、`to_tsquery`は構文を直接指定
- ステミングにより`running`, `runs`, `runner`はすべて`run`にマッチ
- `setweight`の重みはA > B > C > D（デフォルトはD）
- `ts_headline`は検索結果の表示用で、検索自体には使わない

**応用例**:
- ブログ記事の検索（タイトルに高い重み）
- 製品カタログの検索（名前、説明、タグを統合）
- ドキュメント検索システム

**限界**:
- 日本語は形態素解析が別途必要（`pg_bigm`や`pgroonga`を使用）
- 数億件規模ではElasticsearchの検討が必要
- ファセット検索、ハイライト位置の取得などはElasticsearchの方が充実

**実装で確認した動作**:
```sql
-- ステミングの確認
SELECT to_tsvector('english', 'running runs runner');
-- → 'run':1,2 'runner':3

SELECT plainto_tsquery('english', 'ghost in shell');
-- → 'ghost' & 'shell'  （'in'はストップワードとして除去）
```

---

### Chapter 7: Extensions

**pgcrypto**: パスワードハッシュ化とAES暗号化。

```sql
-- bcryptでパスワードをハッシュ化
SELECT crypt('user_password', gen_salt('bf', 8)) AS hashed;

-- パスワード検証
SELECT (stored_hash = crypt('input_password', stored_hash)) AS valid;

-- AES暗号化/復号化
SELECT encrypt('sensitive data', 'secret_key', 'aes') AS encrypted;
SELECT convert_from(decrypt(encrypted_data, 'secret_key', 'aes'), 'UTF8');
```

**pg_trgm**: ファジー検索とタイプミス許容。

```sql
-- 類似度計算
SELECT similarity('PostgreSQL', 'Postgres');  -- 0.67

-- タイプミスを許容した検索
SELECT name FROM products
WHERE name % 'Capuchino'  -- Cappuccinoがヒット
ORDER BY similarity(name, 'Capuchino') DESC;

-- LIKE検索の高速化（GINインデックス使用）
CREATE INDEX idx_trgm ON products USING GIN (name gin_trgm_ops);
SELECT * FROM products WHERE name ILIKE '%capp%';
```

**citext**: 大文字小文字を無視。

```sql
CREATE TABLE users (email CITEXT PRIMARY KEY);
INSERT INTO users VALUES ('User@Example.COM');

-- 大文字小文字を無視して検索
SELECT * FROM users WHERE email = 'user@example.com';  -- ヒット
```

**実装時の気づき**:
- bcryptのコスト（8〜12が推奨）が高いほどハッシュ計算に時間がかかる
- `gen_random_bytes(16)`でランダムなソルトを生成できる
- `pg_trgm`の類似度閾値はデフォルト0.3（`set_limit()`で変更可能）
- `citext`は内部的にLOWER()を適用しているため、インデックスも自動的に対応

**応用例**:
- パスワードリセットトークンの安全な保存
- クレジットカード番号の暗号化（PCI DSS対応）
- 検索サジェスト機能（タイプミス許容）
- メールアドレスの重複チェック

**限界**:
- 暗号化キーの管理はPostgres外で行う必要がある
- `pg_trgm`は3文字未満の文字列では効果が低い
- `citext`は照合順序（collation）に依存する場合がある

---

### Chapter 8: Postgres for Generative AI

**pgvector**: ベクトル型と類似検索。

```sql
-- pgvector拡張を有効化
CREATE EXTENSION vector;

-- ベクトル型のテーブル
CREATE TABLE movies_vector (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    movie_embedding VECTOR(128)  -- 128次元ベクトル
);

-- Cosine Distance（<=>）で類似検索
SELECT name, movie_embedding <=> target_embedding AS distance
FROM movies_vector
ORDER BY distance
LIMIT 5;
```

**距離関数の比較**:

```sql
SELECT
    v1 <=> v2 AS cosine_distance,   -- コサイン距離（正規化ベクトル向け）
    v1 <-> v2 AS l2_distance,       -- ユークリッド距離
    v1 <#> v2 AS inner_product      -- 内積（負の値）
FROM vectors;
```

**IVFFlat vs HNSW インデックス**:

```sql
-- IVFFlat: ビルド高速、メモリ少、probesで精度調整
CREATE INDEX idx_ivfflat ON movies_vector
USING ivfflat (movie_embedding vector_cosine_ops)
WITH (lists = 100);

-- HNSW: 検索高速、高精度、メモリ多
CREATE INDEX idx_hnsw ON movies_vector
USING hnsw (movie_embedding vector_cosine_ops)
WITH (m = 16, ef_construction = 64);
```

**実装時の気づき**:
- ベクトルはランダム生成でも類似検索の動作確認は可能（ジャンルごとにパターンを変える）
- `<#>`（内積）は負の値を返すため、`ORDER BY`で昇順にすると最も類似したものが先頭に
- IVFFlatの`lists`パラメータはデータ量の平方根が目安
- HNSWの`m`パラメータは精度とメモリのトレードオフ

**応用例**:
- RAG（Retrieval-Augmented Generation）のコンテキスト検索
- 類似画像検索
- レコメンデーションシステム
- 異常検知（正常ベクトルからの距離）

**限界**:
- 数億ベクトル規模ではPinecone、Milvusなどの専用Vector DBを検討
- 埋め込みモデル（Ollama等）との連携はアプリ層で実装が必要
- インデックス構築に時間がかかる（特にHNSW）

**実装で遭遇した問題**:
```sql
-- 他のデモで作成したTEMP TABLE 'movies' との名前衝突
-- 解決策: テーブル名を 'movies_vector' に変更
DROP TABLE IF EXISTS pg_temp.movies_vector;
CREATE TEMP TABLE movies_vector (...);
```

---

### Chapter 9: Postgres for Time Series

**TimescaleDB**: 時系列データ専用の拡張機能。

```sql
-- TimescaleDB拡張を有効化
CREATE EXTENSION timescaledb;

-- 通常のテーブルを作成
CREATE TABLE heart_rate_measurements (
    watch_id INT NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL,
    heart_rate INT NOT NULL,
    activity TEXT NOT NULL
);

-- Hypertableに変換（自動パーティショニング）
SELECT create_hypertable(
    relation => 'heart_rate_measurements',
    time_column_name => 'recorded_at',
    chunk_time_interval => INTERVAL '1 day'
);
```

**time_bucket関数**: 時系列データの集計。

```sql
-- 1時間ごとの平均心拍数
SELECT
    time_bucket('1 hour', recorded_at) AS hour,
    watch_id,
    AVG(heart_rate)::INT AS avg_hr,
    MAX(heart_rate) AS max_hr
FROM heart_rate_measurements
WHERE recorded_at >= '2025-01-01' AND recorded_at < '2025-01-02'
GROUP BY hour, watch_id
ORDER BY hour;

-- 日別サマリー
SELECT
    time_bucket('1 day', recorded_at) AS day,
    COUNT(*) AS total_measurements,
    AVG(heart_rate)::INT AS avg_hr
FROM heart_rate_measurements
GROUP BY day ORDER BY day;
```

**チャンク（パーティション）管理**:

```sql
-- 自動生成されたチャンクを確認
SELECT chunk_name, range_start, range_end
FROM timescaledb_information.chunks
WHERE hypertable_name = 'heart_rate_measurements';

-- パーティションプルーニング（必要なチャンクのみスキャン）
EXPLAIN SELECT * FROM heart_rate_measurements
WHERE recorded_at >= '2025-01-03' AND recorded_at < '2025-01-04';
-- → _hyper_1_3_chunk のみスキャン
```

**データ保持ポリシー**:

```sql
-- 30日より古いデータを自動削除
SELECT add_retention_policy('heart_rate_measurements', INTERVAL '30 days');
```

**実装時の気づき**:
- Hypertable化すると通常のテーブルと同じようにクエリできる（透過的）
- チャンク間隔は「1日に挿入されるデータ量 × 1週間」程度が目安
- `timescaledb_information.chunks`でチャンクのメタデータを確認できる
- BRINインデックスは時系列データと相性が良い（時間順に挿入されるため）

**応用例**:
- IoTセンサーデータの収集と分析
- サーバーメトリクス（CPU、メモリ使用率）の監視
- 金融データ（株価、為替レート）の時系列分析
- ログデータの集約と可視化

**限界**:
- Hyperscale（分散TimescaleDB）は有料版のみ
- 単一ノードでの垂直スケーリングには限界がある
- Continuous Aggregates（マテリアライズドビュー）はリアルタイムではない

**実装で確認した動作**:
```
【チャンク（パーティション）の確認】
  自動生成されたチャンク（先頭5件）:
    _hyper_1_1_chunk | 2025-01-01 - 2025-01-02
    _hyper_1_2_chunk | 2025-01-02 - 2025-01-03
    _hyper_1_3_chunk | 2025-01-03 - 2025-01-04
    _hyper_1_4_chunk | 2025-01-04 - 2025-01-05
    _hyper_1_5_chunk | 2025-01-05 - 2025-01-06
```

---

### Chapter 10: Postgres for Geospatial Data

**PostGIS**: 地理空間データ専用の拡張機能。

```sql
-- PostGIS拡張を有効化
CREATE EXTENSION postgis;

-- geometry型（平面座標系）とgeography型（球面座標系）
CREATE TABLE cities (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    -- SRID 4326: WGS 84（GPS座標系）
    boundaries geometry(Polygon, 4326)
);

CREATE TABLE points_of_interest (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    location geometry(Point, 4326)
);
```

**ポイントとポリゴンの作成**:

```sql
-- ポイントの作成
INSERT INTO points_of_interest (name, location) VALUES
('Magic Kingdom', ST_SetSRID(ST_MakePoint(-81.5812, 28.4177), 4326));

-- ポリゴンの作成（都市境界）
INSERT INTO cities (name, boundaries) VALUES
('Tampa', ST_GeomFromText(
    'POLYGON((-82.55 27.85, -82.35 27.85, -82.35 28.05, -82.55 28.05, -82.55 27.85))',
    4326
));
```

**ST_Distance**: 距離計算（メートル）。

```sql
-- geography型にキャストして正確な距離を計算
SELECT
    p1.name AS from_place,
    p2.name AS to_place,
    round(ST_Distance(
        p1.location::geography,
        p2.location::geography
    ))::INT AS distance_meters
FROM points_of_interest p1, points_of_interest p2
WHERE p1.name = 'Hotel' AND p2.amenity = 'restaurant';
```

**ST_DWithin**: 範囲検索（指定距離内）。

```sql
-- ホテルから1km以内の施設を検索
SELECT p.name, p.amenity
FROM points_of_interest p, points_of_interest hotel
WHERE hotel.name = 'Roost Apartment Hotel'
  AND ST_DWithin(
      p.location::geography,
      hotel.location::geography,
      1000  -- 1000メートル以内
  );
```

**ST_Within / ST_Contains**: 包含関係。

```sql
-- 各都市内の観光スポットを検索
SELECT c.name AS city, p.name AS poi
FROM points_of_interest p
JOIN cities c ON ST_Within(p.location, c.boundaries);
```

**ST_Transform**: 座標系変換。

```sql
-- WGS 84（SRID 4326）からWeb Mercator（SRID 3857）へ変換
SELECT
    name,
    ST_X(location) AS lon_wgs84,
    ST_Y(location) AS lat_wgs84,
    ST_X(ST_Transform(location, 3857)) AS x_mercator,
    ST_Y(ST_Transform(location, 3857)) AS y_mercator
FROM points_of_interest;
```

**GiSTインデックス**: 空間検索の高速化。

```sql
-- GiSTインデックスを作成
CREATE INDEX idx_poi_location ON points_of_interest USING GIST (location);
CREATE INDEX idx_cities_boundaries ON cities USING GIST (boundaries);

-- インデックスを使った高速な空間検索
SELECT p.name FROM points_of_interest p
JOIN cities c ON ST_Within(p.location, c.boundaries)
WHERE c.name = 'Miami';
```

**実装時の気づき**:
- `geometry`は平面座標系で高速だが大距離で誤差、`geography`は球面座標系で正確だが遅い
- `ST_Distance`に`geometry`型を渡すと単位は「度」、`geography`型を渡すと「メートル」
- `ST_DWithin`は`ST_Distance`より効率的（インデックスを活用できる）
- ポリゴンは必ず閉じる（最初と最後の点が同じ）

**応用例**:
- 店舗検索（現在地から最寄りの店舗）
- 配達範囲の判定（ポリゴン内にあるか）
- ジオフェンシング（特定エリアへの出入りを検知）
- ルート計算（pgRoutingと組み合わせ）

**限界**:
- 大規模な地図タイル配信には専用のタイルサーバーが必要
- リアルタイム車両追跡（毎秒更新）は更新負荷が高い
- 高度な地図レンダリングはMapboxやLeafletなどと連携が必要

**実装で確認した動作**:
```
【都市間の距離】
  Orlando → Tampa: 121 km
  Miami → Orlando: 318 km
  Miami → Tampa: 325 km

【座標変換（ST_Transform）】
  Walt Disney World:
    WGS 84 (SRID 4326): 経度 -81.5639, 緯度 28.3852
    Web Mercator (SRID 3857): X -9079651.82, Y 3297626.07
```

---

### Chapter 11: Postgres as a Message Queue

**カスタムキュー実装**: FOR UPDATE SKIP LOCKEDで並行処理。

```sql
-- キューテーブル
CREATE TABLE mq.queue (
    id BIGSERIAL PRIMARY KEY,
    message JSON NOT NULL,
    status mq_status NOT NULL DEFAULT 'new',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    worker_id TEXT
);

-- dequeue関数（複数ワーカーで競合なし）
CREATE FUNCTION mq.dequeue(worker TEXT, batch_size INT DEFAULT 1)
RETURNS TABLE (message_id BIGINT, payload JSON) AS $$
BEGIN
    RETURN QUERY
    WITH locked_messages AS (
        SELECT id FROM mq.queue
        WHERE status = 'new'
        ORDER BY created_at
        FOR UPDATE SKIP LOCKED  -- ロック済みの行をスキップ
        LIMIT batch_size
    ),
    updated AS (
        UPDATE mq.queue q
        SET status = 'processing', worker_id = worker
        FROM locked_messages lm WHERE q.id = lm.id
        RETURNING q.id, q.message
    )
    SELECT updated.id, updated.message FROM updated;
END;
$$ LANGUAGE plpgsql;
```

**LISTEN/NOTIFY**: リアルタイム通知。

```sql
-- enqueue時に自動通知
CREATE FUNCTION mq.enqueue(payload JSON) RETURNS BIGINT AS $$
DECLARE new_id BIGINT;
BEGIN
    INSERT INTO mq.queue (message) VALUES (payload) RETURNING id INTO new_id;
    PERFORM pg_notify('queue_new_message', 'new_message');
    RETURN new_id;
END;
$$ LANGUAGE plpgsql;

-- ワーカー側でLISTEN
LISTEN queue_new_message;
```

**トランザクショナル一貫性**: ビジネスロジックとメッセージ送信を1トランザクションで。

```sql
BEGIN;
-- 決済処理
UPDATE orders SET status = 'paid' WHERE id = 123;

-- メール送信キューに追加（同じトランザクション内）
SELECT mq.enqueue('{"type": "send_email", "order_id": 123}');

COMMIT;  -- 両方成功するか、両方ロールバック
```

**実装時の気づき**:
- `FOR UPDATE SKIP LOCKED`は行ロックの取得を待たずにスキップするため、複数ワーカーが同時にdequeueしても競合しない
- `LISTEN/NOTIFY`はコネクションごとに状態を持つため、コネクションプールとの相性に注意
- 通知ペイロードは8000バイト制限がある（大きなデータはキュー側で保持）
- `pg_notify`はトランザクション終了時に送信される

**応用例**:
- 非同期メール送信
- バックグラウンドジョブ（レポート生成、データ集計）
- イベント駆動アーキテクチャ
- マイクロサービス間の疎結合な通信

**限界**:
- 超高スループット（秒間数万メッセージ）には専用MQが必要
- 複雑なルーティング（トピック、パターンマッチング）はRabbitMQ等が得意
- メッセージの永続化と再配信はアプリ側で実装が必要
- `LISTEN`は接続後の通知のみ受信可能（過去の通知は取得できない）

**実装で確認した動作**:
```
【Worker 1: メッセージを取得（2件）】
  Worker 1 がメッセージ 1 を取得: {"service":"registration","visitor":"Alice"}
  Worker 1 がメッセージ 2 を取得: {"service":"license_renewal","visitor":"Bob"}

【Worker 2: メッセージを取得（SKIP LOCKEDで競合回避）】
  Worker 2 がメッセージ 3 を取得: {"service":"registration","visitor":"Charlie"}
  Worker 2 がメッセージ 4 を取得: {"service":"id_card","visitor":"Diana"}

→ Worker 1とWorker 2が同時にdequeueしても、異なるメッセージを取得
```

---

## クイックスタート

### 1. PostgreSQL を起動

```bash
cd verification
docker compose up -d
```

**TimescaleDB + PostGIS + pgvector入りの PostgreSQL 17** が `localhost:5432` で起動します。

初回起動時に `init/` 配下のSQLが自動実行され、スキーマとサンプルデータが作成されます。

> **Note**: Dockerイメージに `timescale/timescaledb-ha:pg17` を使用しています。このイメージには以下の拡張機能が含まれています：
> - **TimescaleDB**: 時系列データ（Chapter 9）
> - **PostGIS**: 地理空間データ（Chapter 10）
> - **pgvector**: ベクトル検索（Chapter 8）

### 2. Rust プログラムをビルド

```bash
cargo build --release
```

### 3. デモを実行

```bash
# すべてのデモを実行
cargo run --release -- all

# 個別のデモを実行
cargo run --release -- generate-series      # Chapter 1: generate_series
cargo run --release -- show-schemas         # Chapter 2: スキーマ構造
cargo run --release -- transaction          # Chapter 2: トランザクション
cargo run --release -- cart-demo            # Chapter 2: DB関数
cargo run --release -- view-demo            # Chapter 2: View
cargo run --release -- cte-demo             # Chapter 3: CTE
cargo run --release -- window-functions     # Chapter 3: Window Functions
cargo run --release -- recursive-query      # Chapter 3: Recursive Query
cargo run --release -- index-demo           # Chapter 4: インデックス戦略
cargo run --release -- jsonb-demo           # Chapter 5: JSONB基本
cargo run --release -- json-advanced        # Chapter 5: JSON詳細
cargo run --release -- full-text-search "coffee espresso"  # Chapter 6: 全文検索基本
cargo run --release -- full-text-advanced   # Chapter 6: 全文検索詳細
cargo run --release -- extensions           # Chapter 7: 拡張機能
cargo run --release -- pgvector             # Chapter 8: ベクトル検索
cargo run --release -- timescaledb          # Chapter 9: 時系列データ
cargo run --release -- postgis              # Chapter 10: 地理空間データ
cargo run --release -- message-queue        # Chapter 11: メッセージキュー
```

---

## Rust実装の詳細

### 非同期PostgreSQL接続

```rust
use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;

async fn create_pool() -> Result<Pool> {
    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.port = Some(5432);
    cfg.user = Some("postgres".to_string());
    cfg.password = Some("password".to_string());
    cfg.dbname = Some("coffee_chain".to_string());

    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
    Ok(pool)
}
```

### クエリ実行パターン

```rust
// 単一行の取得
let row = client.query_one(
    "SELECT name, price FROM products WHERE id = $1",
    &[&product_id]
).await?;
let name: String = row.get("name");
let price: Decimal = row.get("price");

// 複数行の取得
let rows = client.query(
    "SELECT * FROM products WHERE category = $1",
    &[&category]
).await?;
for row in rows {
    let name: String = row.get("name");
    println!("{}", name);
}

// 更新系クエリ
client.execute(
    "UPDATE products SET price = $1 WHERE id = $2",
    &[&new_price, &product_id]
).await?;
```

### Rustでの型マッピングの注意点

```rust
// pg_typeof()の結果を取得する際は::TEXTキャストが必要
let row = client.query_one(
    "SELECT pg_typeof(data)::TEXT AS type FROM table",
    &[]
).await?;

// ENUM型はStringとして取得（カスタムデシリアライザが必要な場合も）
let status: String = row.get("status");

// tsvector型は直接デシリアライズできない
// → ::TEXT にキャストするか、ts_rank()などの結果を使う
```

### JSONBの扱い

```rust
use serde_json::Value;

// JSONBカラムの取得
let row = client.query_one("SELECT settings FROM users WHERE id = $1", &[&1]).await?;
let settings: Value = row.get("settings");

// JSONフィールドへのアクセス
if let Some(theme) = settings.get("theme") {
    println!("Theme: {}", theme);
}
```

### 日付・時刻の扱い（TimescaleDB）

```rust
use chrono::{DateTime, Utc};

let row = client.query_one(
    "SELECT time_bucket('1 hour', recorded_at) AS hour, AVG(heart_rate) AS avg
     FROM heart_rate_measurements GROUP BY hour",
    &[]
).await?;

let hour: DateTime<Utc> = row.get("hour");
println!("Hour: {}", hour.format("%Y-%m-%d %H:%M"));
```

---

## 実装中に遭遇した主なエラーと解決策

### 1. TEMP TABLE名の衝突

**問題**: `full_text_advanced.rs`で作成した`movies`テーブルと`pgvector.rs`の`movies`テーブルが衝突。

**解決策**: テーブル名を`movies_vector`に変更。

```rust
// Before
client.execute("CREATE TEMP TABLE movies ...", &[]).await?;

// After
client.execute("DROP TABLE IF EXISTS pg_temp.movies_vector", &[]).await?;
client.execute("CREATE TEMP TABLE movies_vector ...", &[]).await?;
```

### 2. pg_typeof()のデシリアライズエラー

**問題**: `pg_typeof()`の戻り値をRustで取得しようとしてエラー。

**解決策**: `::TEXT`でキャスト。

```sql
-- Before
SELECT pg_typeof(data) AS type FROM table

-- After
SELECT pg_typeof(data)::TEXT AS type FROM table
```

### 3. PL/pgSQL変数名とカラム名の衝突

**問題**: 関数内の変数名がテーブルのカラム名と同じでエラー。

**解決策**: 変数名にプレフィックス（`v_`や`p_`）を付ける。

```sql
-- Before
CREATE FUNCTION test(order_id UUID) ...
SELECT * FROM orders WHERE order_id = order_id;  -- あいまい

-- After
CREATE FUNCTION test(p_order_id UUID) ...
SELECT * FROM orders WHERE order_id = p_order_id;  -- 明確
```

### 4. ENUM型のデシリアライズ

**問題**: PostgreSQLのENUM型をRustで直接取得できない。

**解決策**: `::TEXT`でキャストするか、文字列として取得。

```rust
let status: String = row.get("status");  // ENUMをStringとして取得
```

---

## ディレクトリ構成

```
verification/
├── Cargo.toml              # Rust プロジェクト設定
├── docker-compose.yml      # PostgreSQL コンテナ設定（TimescaleDB + PostGIS + pgvector入り）
├── init/                   # DB初期化SQL
│   ├── 01_schema.sql       # スキーマ・テーブル・関数定義
│   └── 02_seed_data.sql    # サンプルデータ
└── src/
    ├── main.rs             # エントリポイント・CLI定義
    └── examples/           # 各機能のデモ
        ├── mod.rs
        ├── generate_series.rs    # Chapter 1
        ├── schemas.rs            # Chapter 2
        ├── transaction.rs        # Chapter 2
        ├── cart.rs               # Chapter 2
        ├── view.rs               # Chapter 2
        ├── cte.rs                # Chapter 3
        ├── window_functions.rs   # Chapter 3
        ├── recursive.rs          # Chapter 3
        ├── indexes.rs            # Chapter 4
        ├── jsonb.rs              # Chapter 5
        ├── json_advanced.rs      # Chapter 5
        ├── full_text_search.rs   # Chapter 6
        ├── full_text_advanced.rs # Chapter 6
        ├── extensions.rs         # Chapter 7
        ├── pgvector.rs           # Chapter 8
        ├── timescaledb.rs        # Chapter 9
        ├── postgis.rs            # Chapter 10
        └── message_queue.rs      # Chapter 11
```

---

## データベース構成

```
coffee_chain (database)
├── products (schema)
│   ├── catalog (table)     # 商品カタログ
│   └── reviews (table)     # レビュー
├── customers (schema)
│   └── accounts (table)    # 顧客アカウント
├── sales (schema)
│   ├── orders (table)      # 注文
│   ├── order_items (table) # 注文明細
│   └── product_sales_summary (view)
├── streaming (schema)      # Chapter 3 Modern SQL用
│   ├── songs (table)       # 曲
│   └── plays (table)       # 再生履歴（連続再生のリンク付き）
├── watch (schema)          # Chapter 9 TimescaleDB用
│   └── heart_rate_measurements (hypertable)  # 心拍数計測
├── florida (schema)        # Chapter 10 PostGIS用
│   ├── cities (table)      # 都市（Polygon）
│   └── points_of_interest (table)  # 観光スポット（Point）
└── mq (schema)             # Chapter 11 メッセージキュー用
    └── queue (table)       # メッセージキュー
```

---

## 主な学び

### 「Just Use Postgres」の本当の意味

> 「新規プロジェクトならとりあえずPostgres」ではない。
>
> 「既にPostgresを使っているチームが、新しいユースケースが発生したとき、
> 別のデータベースを追加する前に、まずPostgresで解決できるか確認してみよう」

### 各機能の位置づけ

| ユースケース | 従来の選択肢 | Postgresでの代替 |
|------------|------------|----------------|
| 全文検索 | Elasticsearch | GIN + tsvector |
| ドキュメントDB | MongoDB | JSONB |
| 時系列データ | InfluxDB | TimescaleDB（Hypertable + time_bucket） |
| グラフ構造 | Neo4j | Recursive CTE |
| キューイング | Redis/RabbitMQ | LISTEN/NOTIFY + FOR UPDATE SKIP LOCKED |
| ファジー検索 | Elasticsearch | pg_trgm + GIN |
| ベクトル検索 | Pinecone/Milvus | pgvector |
| 地理空間 | MongoDB | PostGIS（geometry/geography + GiST） |

### 判断基準：いつPostgresを使い、いつ専用ツールを使うか

| 機能 | Postgresで十分 | 専用ツールを検討 |
|-----|--------------|----------------|
| 全文検索 | 数百万件以下、シンプルな検索 | 数億件、複雑なファセット、日本語形態素解析 |
| JSONB | 構造が可変なメタデータ | ドキュメント中心のデータモデル全体 |
| TimescaleDB | 単一ノードで処理可能な規模 | 分散処理が必要な大規模データ |
| pgvector | 数百万ベクトル以下 | 数億ベクトル、リアルタイム更新 |
| PostGIS | 店舗検索、配達範囲判定 | 大規模タイル配信、リアルタイム追跡 |
| Message Queue | 秒間数百メッセージ | 秒間数万メッセージ、複雑なルーティング |

### 注意点

- すべてをPostgresで解決すべきではない
- 各機能の限界を理解した上で、適切な場面で活用する
- 「Postgresで試した？」と聞き返せるようになることが目標
- 運用の複雑さを減らすことにも価値がある（1つのDBで完結できる利点）

---

## Axum Webフレームワークとの統合

本プロジェクトで検証したPostgreSQL機能をAxum Webアプリケーションに統合する方法を解説します。

### アーキテクチャ概要

```
┌─────────────────────────────────────────────────────────────┐
│                      Axum Application                        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │   Router    │  │  Handlers   │  │     Middleware      │ │
│  │             │  │             │  │  (Auth, Logging)    │ │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘ │
│         │                │                     │            │
│  ┌──────▼─────────────────▼─────────────────────▼─────────┐ │
│  │                    AppState                             │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ │ │
│  │  │  PgPool     │  │   Config    │  │  Repositories   │ │ │
│  │  │ (deadpool)  │  │             │  │  (trait objects)│ │ │
│  │  └──────┬──────┘  └─────────────┘  └────────┬────────┘ │ │
│  └─────────┼───────────────────────────────────┼──────────┘ │
└────────────┼───────────────────────────────────┼────────────┘
             │                                   │
             ▼                                   ▼
┌─────────────────────────────────────────────────────────────┐
│                     PostgreSQL                               │
│  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌─────────────┐ │
│  │ JSONB     │ │ Full-Text │ │ pgvector  │ │ TimescaleDB │ │
│  │           │ │  Search   │ │           │ │             │ │
│  └───────────┘ └───────────┘ └───────────┘ └─────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 接続プールの選択: SQLx vs deadpool-postgres

| 観点 | SQLx | deadpool-postgres |
|------|------|-------------------|
| コンパイル時クエリチェック | ✅ あり | ❌ なし |
| ステートメントキャッシュ | ✅ 内蔵 | 🔧 手動設定 |
| ランタイムプールサイズ変更 | ❌ 制限あり | ✅ 可能 |
| バックグラウンドタスク | あり | なし |
| 起動時の挙動 | 失敗する可能性あり | 失敗しない |
| 推奨ユースケース | 型安全性重視 | 動的スケーリング |

**本プロジェクトではdeadpool-postgresを採用**: tokio-postgresとの直接的な統合により、PostgreSQL固有の機能（LISTEN/NOTIFY、COPY、拡張型）を活用しやすい。

### AppState設計パターン

```rust
use axum::extract::FromRef;
use deadpool_postgres::Pool;
use std::sync::Arc;

/// アプリケーション全体の共有状態
#[derive(Clone, FromRef)]
pub struct AppState {
    /// データベース接続プール
    pub db: Pool,
    /// 設定情報
    pub config: AppConfig,
    /// リポジトリ（オプション: DIパターン）
    pub user_repo: Arc<dyn UserRepository>,
}

#[derive(Clone)]
pub struct AppConfig {
    pub jwt_secret: String,
    pub bcrypt_cost: u32,
}
```

### エラーハンドリング

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Conflict(String),
    InvalidInput(String),
    DatabaseError(String),
    Unauthorized,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::DatabaseError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "Unauthorized".to_string(),
            ),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

// tokio_postgres::Errorからの自動変換
impl From<tokio_postgres::Error> for AppError {
    fn from(err: tokio_postgres::Error) -> Self {
        AppError::DatabaseError(err.to_string())
    }
}

// deadpool::PoolErrorからの自動変換
impl From<deadpool_postgres::PoolError> for AppError {
    fn from(err: deadpool_postgres::PoolError) -> Self {
        AppError::DatabaseError(err.to_string())
    }
}
```

### ハンドラ実装例

**商品検索（JSONB + 全文検索）**:

```rust
use axum::{extract::{State, Query}, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SearchParams {
    q: Option<String>,
    category: Option<String>,
    min_price: Option<f64>,
    max_price: Option<f64>,
}

#[derive(Serialize)]
pub struct Product {
    id: i32,
    name: String,
    category: String,
    price: f64,
    metadata: serde_json::Value,
    rank: Option<f32>,
}

pub async fn search_products(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<Product>>, AppError> {
    let client = state.db.get().await?;

    // 全文検索 + JSONB + 価格フィルタの複合クエリ
    let rows = client.query(
        r#"
        SELECT
            id, name, category, price, metadata,
            ts_rank(search_vector, plainto_tsquery('english', $1)) AS rank
        FROM products.catalog
        WHERE ($1 IS NULL OR search_vector @@ plainto_tsquery('english', $1))
          AND ($2 IS NULL OR category = $2)
          AND ($3 IS NULL OR price >= $3)
          AND ($4 IS NULL OR price <= $4)
        ORDER BY rank DESC NULLS LAST, name
        LIMIT 50
        "#,
        &[&params.q, &params.category, &params.min_price, &params.max_price],
    ).await?;

    let products: Vec<Product> = rows.iter().map(|row| Product {
        id: row.get("id"),
        name: row.get("name"),
        category: row.get("category"),
        price: row.get("price"),
        metadata: row.get("metadata"),
        rank: row.try_get("rank").ok(),
    }).collect();

    Ok(Json(products))
}
```

**類似検索（pgvector）**:

```rust
pub async fn similar_products(
    State(state): State<AppState>,
    Path(product_id): Path<i32>,
) -> Result<Json<Vec<Product>>, AppError> {
    let client = state.db.get().await?;

    let rows = client.query(
        r#"
        SELECT p.id, p.name, p.category, p.price,
               1 - (p.embedding <=> target.embedding) AS similarity
        FROM products.catalog p,
             (SELECT embedding FROM products.catalog WHERE id = $1) target
        WHERE p.id != $1
        ORDER BY p.embedding <=> target.embedding
        LIMIT 10
        "#,
        &[&product_id],
    ).await?;

    // ... マッピング処理
    Ok(Json(products))
}
```

### トランザクション処理

```rust
pub async fn create_order(
    State(state): State<AppState>,
    Json(payload): Json<CreateOrderRequest>,
) -> Result<Json<Order>, AppError> {
    let mut client = state.db.get().await?;

    // トランザクション開始
    let tx = client.transaction().await?;

    // 注文作成
    let order_row = tx.query_one(
        "INSERT INTO sales.orders (customer_id, status)
         VALUES ($1, 'pending') RETURNING id",
        &[&payload.customer_id],
    ).await?;
    let order_id: uuid::Uuid = order_row.get("id");

    // 注文明細を一括挿入
    for item in &payload.items {
        tx.execute(
            "INSERT INTO sales.order_items (order_id, product_id, quantity, price)
             SELECT $1, $2, $3, price FROM products.catalog WHERE id = $2",
            &[&order_id, &item.product_id, &item.quantity],
        ).await?;
    }

    // 合計金額を更新（トリガーで自動計算される場合は不要）
    tx.execute(
        "UPDATE sales.orders SET total_amount = (
            SELECT SUM(quantity * price) FROM sales.order_items WHERE order_id = $1
        ) WHERE id = $1",
        &[&order_id],
    ).await?;

    // コミット
    tx.commit().await?;

    Ok(Json(Order { id: order_id, /* ... */ }))
}
```

### リポジトリパターン（テスト容易性向上）

```rust
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<User, AppError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn create(&self, user: CreateUserRequest) -> Result<User, AppError>;
}

pub struct PostgresUserRepository {
    pool: Pool,
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: i32) -> Result<User, AppError> {
        let client = self.pool.get().await?;
        let row = client.query_opt(
            "SELECT id, name, email, created_at FROM customers.accounts WHERE id = $1",
            &[&id],
        ).await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

        Ok(User {
            id: row.get("id"),
            name: row.get("name"),
            email: row.get("email"),
            created_at: row.get("created_at"),
        })
    }

    // ... 他のメソッド
}

// テスト用のモック実装
#[cfg(test)]
pub struct MockUserRepository {
    users: std::sync::RwLock<std::collections::HashMap<i32, User>>,
}
```

### JWT認証ミドルウェア

```rust
use axum::{
    middleware::Next,
    extract::Request,
    http::header::AUTHORIZATION,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // ユーザーID
    pub exp: i64,     // 有効期限
    pub role: String, // ロール
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    ).map_err(|_| AppError::Unauthorized)?;

    // リクエスト拡張にクレームを追加
    req.extensions_mut().insert(token_data.claims);

    Ok(next.run(req).await)
}
```

### ルーター構成

```rust
use axum::{
    routing::{get, post, put, delete},
    Router,
    middleware,
};

pub fn create_router(state: AppState) -> Router {
    // 認証不要のルート
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route("/products/search", get(search_products));

    // 認証必要のルート
    let protected_routes = Router::new()
        .route("/users/me", get(get_current_user))
        .route("/orders", post(create_order))
        .route("/orders/{id}", get(get_order))
        .route("/cart", get(get_cart).post(add_to_cart))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    // 管理者専用ルート
    let admin_routes = Router::new()
        .route("/admin/users", get(list_users))
        .route("/admin/products", post(create_product))
        .layer(middleware::from_fn(require_admin));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .nest("/api/v1", admin_routes)
        .with_state(state)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // データベースプール作成
    let pool = create_pool().await?;

    // AppState構築
    let state = AppState {
        db: pool.clone(),
        config: AppConfig { /* ... */ },
        user_repo: Arc::new(PostgresUserRepository { pool }),
    };

    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

### PostgreSQL機能別の統合パターン

| PostgreSQL機能 | Axumでの活用パターン |
|---------------|---------------------|
| JSONB | 柔軟なフィルタリング、メタデータ検索 |
| 全文検索 | 検索API（`ts_rank`でスコア順） |
| pgvector | レコメンデーションAPI |
| PostGIS | 位置検索API（最寄り店舗など） |
| TimescaleDB | メトリクスAPI（時系列集計） |
| LISTEN/NOTIFY | WebSocket連携（リアルタイム更新） |
| FOR UPDATE SKIP LOCKED | バックグラウンドジョブ |

### テスト戦略

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;

    async fn create_test_app() -> TestServer {
        let pool = create_test_pool().await.unwrap();
        let state = AppState { /* テスト用設定 */ };
        let app = create_router(state);
        TestServer::new(app).unwrap()
    }

    #[tokio::test]
    async fn test_search_products() {
        let server = create_test_app().await;

        let response = server
            .get("/products/search")
            .add_query_param("q", "coffee")
            .await;

        response.assert_status_ok();
        let products: Vec<Product> = response.json();
        assert!(!products.is_empty());
    }

    #[tokio::test]
    async fn test_create_order_unauthorized() {
        let server = create_test_app().await;

        let response = server
            .post("/orders")
            .json(&CreateOrderRequest { /* ... */ })
            .await;

        response.assert_status_unauthorized();
    }
}
```

---

## 参考

- [Just Use Postgres!](https://www.manning.com/books/just-use-postgres) - Manning Publications
- [Modern SQL](https://modern-sql.com/) - Markus Winand
- [失敗から学ぶRDBの正しい歩き方](https://gihyo.jp/book/2019/978-4-297-10408-5) - そーだい
- [pgvector](https://github.com/pgvector/pgvector) - Open-source vector similarity search for Postgres
- [TimescaleDB Documentation](https://docs.timescale.com/)
- [PostGIS Documentation](https://postgis.net/documentation/)
- [Axum Documentation](https://docs.rs/axum/latest/axum/)
- [tokio-postgres Documentation](https://docs.rs/tokio-postgres/latest/tokio_postgres/)
- [deadpool-postgres Documentation](https://docs.rs/deadpool-postgres/latest/deadpool_postgres/)
