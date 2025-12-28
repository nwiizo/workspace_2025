//! Chapter 8: Postgres for Generative AI
//!
//! 書籍の8章で説明されているベクトルデータベース機能。
//! - pgvector拡張
//! - ベクトル型（vector）
//! - Cosine Distance（<=>演算子）
//! - IVFFlatとHNSWインデックス
//! - RAG（Retrieval-Augmented Generation）のコンテキスト取得

use anyhow::Result;
use colored::Colorize;
use deadpool_postgres::Pool;

/// pgvectorのデモ
///
/// ベクトル検索とインデックスの効果を検証。
/// 実際のRAG実装ではOllamaなどの埋め込みモデルが必要だが、
/// ここではランダムベクトルで検索機能のみを検証。
pub async fn run(pool: &Pool) -> Result<()> {
    let client = pool.get().await?;

    println!("{}", "pgvector検証（Chapter 8: Generative AI）".bold());
    println!("{}", "-".repeat(50));

    // pgvector拡張を有効化
    println!("{}", "【pgvector拡張の有効化】".yellow());

    client
        .execute("CREATE EXTENSION IF NOT EXISTS vector", &[])
        .await?;

    println!("  {} pgvector拡張を有効化しました", "✓".green());

    // ベクトル型の確認
    let version = client
        .query_one(
            "SELECT extversion FROM pg_extension WHERE extname = 'vector'",
            &[],
        )
        .await?;
    let ver: String = version.get("extversion");
    println!("  pgvectorバージョン: {}", ver.cyan());

    // テスト用テーブルを作成（映画の埋め込み）
    println!("\n{}", "【ベクトルテーブルの作成】".yellow());

    // 既存のテーブルがあれば削除（他のデモとの競合を避ける）
    client
        .execute("DROP TABLE IF EXISTS pg_temp.movies_vector", &[])
        .await?;

    client
        .execute(
            "CREATE TEMP TABLE movies_vector (
                id SERIAL PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                -- 128次元のベクトル（実際のモデルは1024次元など）
                movie_embedding VECTOR(128)
            )",
            &[],
        )
        .await?;

    println!("  {} movies_vector テーブルを作成（VECTOR(128)型）", "✓".green());

    // サンプルデータを挿入（ランダムベクトル）
    // 実際のRAGではテキストを埋め込みモデルで変換する
    println!("\n{}", "【サンプルデータの挿入】".yellow());
    println!("{}", "  ※ 実際はOllamaなどの埋め込みモデルで生成".dimmed());

    // 映画データをランダムベクトルで挿入
    // ジャンルごとに似たベクトルを生成（シミュレーション）
    client
        .execute(
            r#"
            INSERT INTO movies_vector (name, description, movie_embedding) VALUES
            -- SF映画（似たベクトルパターン）
            ('The Matrix', 'A computer hacker learns about the true nature of reality.',
             (SELECT array_agg(0.8 + random() * 0.2)::vector(128) FROM generate_series(1, 128))),
            ('Blade Runner', 'A blade runner must pursue and terminate four replicants.',
             (SELECT array_agg(0.75 + random() * 0.25)::vector(128) FROM generate_series(1, 128))),
            ('Interstellar', 'A team of explorers travel through a wormhole in space.',
             (SELECT array_agg(0.7 + random() * 0.3)::vector(128) FROM generate_series(1, 128))),

            -- アニメ映画（別のベクトルパターン）
            ('Spirited Away', 'A young girl enters a world ruled by gods and spirits.',
             (SELECT array_agg(0.3 + random() * 0.2)::vector(128) FROM generate_series(1, 128))),
            ('Your Name', 'Two strangers find themselves linked in a bizarre way.',
             (SELECT array_agg(0.25 + random() * 0.25)::vector(128) FROM generate_series(1, 128))),
            ('Akira', 'A secret military project endangers Neo-Tokyo.',
             (SELECT array_agg(0.35 + random() * 0.25)::vector(128) FROM generate_series(1, 128))),

            -- アクション映画（さらに別のパターン）
            ('Mad Max: Fury Road', 'A woman rebels against a tyrannical ruler.',
             (SELECT array_agg(0.5 + random() * 0.2)::vector(128) FROM generate_series(1, 128))),
            ('John Wick', 'An ex-hit-man comes out of retirement.',
             (SELECT array_agg(0.55 + random() * 0.2)::vector(128) FROM generate_series(1, 128)))
            "#,
            &[],
        )
        .await?;

    let count = client
        .query_one("SELECT COUNT(*) FROM movies_vector", &[])
        .await?;
    let cnt: i64 = count.get(0);
    println!("  {} {} 件の映画データを挿入", "✓".green(), cnt);

    // ベクトル類似検索（インデックスなし）
    println!("\n{}", "【ベクトル類似検索】".yellow());
    println!("{}", "  Cosine Distance（<=>演算子）を使用".dimmed());

    // The Matrixに似た映画を検索
    let similar = client
        .query(
            r#"
            WITH target AS (
                SELECT movie_embedding FROM movies_vector WHERE name = 'The Matrix'
            )
            SELECT
                m.name,
                m.movie_embedding <=> t.movie_embedding AS distance
            FROM movies_vector m, target t
            WHERE m.name != 'The Matrix'
            ORDER BY distance
            LIMIT 3
            "#,
            &[],
        )
        .await?;

    println!("  「The Matrix」に類似した映画（Cosine Distance）:");
    for row in similar {
        let name: String = row.get("name");
        let distance: f64 = row.get("distance");
        println!("    {} (distance: {:.4})", name.cyan(), distance);
    }

    // 距離関数の比較
    println!("\n{}", "【距離関数の比較】".yellow());

    let distances = client
        .query_one(
            r#"
            WITH movies_pair AS (
                SELECT
                    (SELECT movie_embedding FROM movies_vector WHERE name = 'The Matrix') AS v1,
                    (SELECT movie_embedding FROM movies_vector WHERE name = 'Blade Runner') AS v2
            )
            SELECT
                v1 <=> v2 AS cosine_distance,
                v1 <-> v2 AS l2_distance,
                v1 <#> v2 AS inner_product
            FROM movies_pair
            "#,
            &[],
        )
        .await?;

    let cosine: f64 = distances.get("cosine_distance");
    let l2: f64 = distances.get("l2_distance");
    let inner: f64 = distances.get("inner_product");

    println!("  The Matrix vs Blade Runner:");
    println!("    {} Cosine Distance: {:.4}", "<=>".cyan(), cosine);
    println!("    {} L2 Distance:     {:.4}", "<->".cyan(), l2);
    println!("    {} Inner Product:   {:.4}", "<#>".cyan(), inner);

    // IVFFlatインデックス
    println!("\n{}", "【IVFFlatインデックス】".yellow());
    println!("{}", "  クラスタリングベースの近似検索".dimmed());

    client
        .execute(
            "CREATE INDEX idx_movies_vector_ivfflat ON movies_vector
             USING ivfflat (movie_embedding vector_cosine_ops)
             WITH (lists = 2)",
            &[],
        )
        .await?;

    println!("  {} IVFFlatインデックスを作成（lists=2）", "✓".green());

    // インデックスを使った検索
    let explain_ivf = client
        .query(
            r#"
            EXPLAIN (ANALYZE, COSTS OFF, TIMING ON)
            WITH target AS (
                SELECT movie_embedding FROM movies_vector WHERE name = 'Spirited Away'
            )
            SELECT m.name
            FROM movies_vector m, target t
            ORDER BY m.movie_embedding <=> t.movie_embedding
            LIMIT 3
            "#,
            &[],
        )
        .await?;

    for row in explain_ivf.iter().take(5) {
        let plan: String = row.get(0);
        if plan.contains("Index Scan") {
            println!("  {}", plan.green());
        } else {
            println!("  {}", plan.dimmed());
        }
    }

    // HNSWインデックス
    println!("\n{}", "【HNSWインデックス】".yellow());
    println!("{}", "  階層グラフによる高精度検索".dimmed());

    client
        .execute(
            "CREATE INDEX idx_movies_vector_hnsw ON movies_vector
             USING hnsw (movie_embedding vector_cosine_ops)
             WITH (m = 4, ef_construction = 8)",
            &[],
        )
        .await?;

    println!(
        "  {} HNSWインデックスを作成（m=4, ef_construction=8）",
        "✓".green()
    );

    // インデックスサイズの比較
    println!("\n{}", "【インデックスサイズの比較】".yellow());

    let sizes = client
        .query(
            "SELECT indexname, pg_size_pretty(pg_relation_size(indexname::regclass)) AS size
             FROM pg_indexes
             WHERE tablename = 'movies_vector' AND indexname LIKE 'idx_movies_vector%'
             ORDER BY indexname",
            &[],
        )
        .await?;

    for row in sizes {
        let name: String = row.get("indexname");
        let size: String = row.get("size");
        let idx_type = if name.contains("ivfflat") {
            "IVFFlat"
        } else {
            "HNSW"
        };
        println!("  {} ({}) - {}", name.cyan(), idx_type, size);
    }

    // probesパラメータの説明
    println!("\n{}", "【IVFFlat probesパラメータ】".yellow());
    println!("{}", "  検索するリスト数を増やすとRecallが向上".dimmed());

    println!(
        "  {}",
        "-- 単一トランザクション内でprobesを変更".dimmed()
    );
    println!("  {}", "BEGIN;".dimmed());
    println!("  {}", "SET LOCAL ivfflat.probes = 2;".dimmed());
    println!("  {}", "SELECT ... ORDER BY <=> LIMIT 3;".dimmed());
    println!("  {}", "COMMIT;".dimmed());

    // RAGのワークフロー説明
    println!("\n{}", "【RAGワークフロー（概念）】".yellow());
    println!("  1. ユーザーが質問を入力");
    println!("  2. 質問を埋め込みモデルでベクトル化");
    println!("  3. {} でコンテキストを検索", "Postgres + pgvector".cyan());
    println!("  4. 検索結果をLLMのプロンプトに追加");
    println!("  5. LLMが回答を生成");

    // 使い分けの説明
    println!("\n{}", "【IVFFlat vs HNSW】".yellow());
    println!("  {}:", "IVFFlat".cyan());
    println!("    - ビルド時間が短い");
    println!("    - メモリ使用量が少ない");
    println!("    - probesでRecall調整可能");
    println!("    - データ変更時に再構築推奨");

    println!("  {}:", "HNSW".cyan());
    println!("    - 検索が高速");
    println!("    - Recallが高い（安定）");
    println!("    - インクリメンタル更新可能");
    println!("    - メモリ使用量が大きい");

    println!(
        "\n{}",
        "💡 ポイント: 既にPostgresを使っているなら、pgvectorでRAGを実装可能。".dimmed()
    );
    println!(
        "{}",
        "   専用のVector Database（Pinecone等）を追加する前に、まずpgvectorで試す。".dimmed()
    );

    Ok(())
}
