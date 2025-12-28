//! Chapter 6: 全文検索の詳細検証
//!
//! 書籍の6章で説明されている全文検索機能の詳細。
//! - tsvector と tsquery
//! - plainto_tsquery vs to_tsquery
//! - setweight による重み付け
//! - ts_headline によるハイライト
//! - GIN vs GiST インデックス

use anyhow::Result;
use colored::Colorize;
use deadpool_postgres::Pool;

/// 全文検索の詳細デモ
pub async fn run(pool: &Pool) -> Result<()> {
    let client = pool.get().await?;

    println!("{}", "全文検索の詳細検証（Chapter 6）".bold());
    println!("{}", "-".repeat(50));

    // テスト用テーブルを作成
    client
        .execute(
            "CREATE TEMP TABLE IF NOT EXISTS movies (
                id SERIAL PRIMARY KEY,
                name VARCHAR(200) NOT NULL,
                description TEXT,
                vote_average NUMERIC(3,1),
                lexemes TSVECTOR GENERATED ALWAYS AS (
                    setweight(to_tsvector('english', coalesce(name, '')), 'A') ||
                    setweight(to_tsvector('english', coalesce(description, '')), 'B')
                ) STORED
            )",
            &[],
        )
        .await?;

    // サンプルデータを挿入
    client
        .execute(
            r#"
            INSERT INTO movies (name, description, vote_average) VALUES
            ('The Matrix', 'A computer hacker learns about the true nature of reality and his role in the war against its controllers. Neo discovers the matrix.', 8.7),
            ('Ghost', 'A murdered man''s love for his partner enables him to remain on Earth as a ghost. He helps protect her from danger.', 7.2),
            ('Ghost in the Shell', 'A cyborg policewoman and her partner hunt a mysterious hacker called the Puppet Master in a futuristic Japan.', 8.0),
            ('Ghostbusters', 'Three parapsychologists start a ghost-catching business in New York City after being fired from Columbia University.', 7.8),
            ('Pirates of the Caribbean', 'Captain Jack Sparrow and blacksmith Will Turner team up to rescue Elizabeth from pirates who are cursed.', 8.0),
            ('The Lion King', 'A lion prince flees his kingdom after the murder of his father and learns about responsibility and bravery.', 8.5),
            ('Finding Nemo', 'A clownfish father named Marlin searches the ocean for his captured son Nemo with the help of a forgetful fish.', 8.1),
            ('Shrek', 'An ogre agrees to rescue a princess for a lord, but finds love along the way. A donkey becomes his unlikely companion.', 7.9)
            "#,
            &[],
        )
        .await?;

    // GINインデックスを作成
    client
        .execute(
            "CREATE INDEX idx_movies_lexemes_gin ON movies USING GIN(lexemes)",
            &[],
        )
        .await?;

    // tsvectorの内部構造
    println!("{}", "【tsvectorの内部構造】".yellow());

    let tsvec = client
        .query_one(
            "SELECT name, lexemes::text FROM movies WHERE name = 'The Matrix'",
            &[],
        )
        .await?;

    let name: String = tsvec.get("name");
    let lexemes: String = tsvec.get("lexemes");
    println!("  映画: {}", name.cyan());
    println!("  lexemes: {}", lexemes.dimmed());
    println!("  → 'A'ラベル（タイトル由来）と'B'ラベル（説明由来）が付与");

    // plainto_tsquery vs to_tsquery
    println!("\n{}", "【plainto_tsquery vs to_tsquery】".yellow());

    let plain = client
        .query_one(
            "SELECT plainto_tsquery('english', 'ghost in shell')::text AS query",
            &[],
        )
        .await?;
    let plain_q: String = plain.get("query");
    println!("  plainto_tsquery('ghost in shell'): {}", plain_q.cyan());
    println!("  → 自然文を自動的にAND検索に変換");

    let advanced = client
        .query_one(
            "SELECT to_tsquery('english', 'ghost & (shell | busters)')::text AS query",
            &[],
        )
        .await?;
    let adv_q: String = advanced.get("query");
    println!("\n  to_tsquery('ghost & (shell | busters)'): {}", adv_q.cyan());
    println!("  → AND/OR/NOTを直接指定可能");

    // 検索の実行
    println!("\n{}", "【全文検索の実行】".yellow());

    let results = client
        .query(
            "SELECT name, ts_rank(lexemes, to_tsquery('english', 'ghost')) AS rank
            FROM movies
            WHERE lexemes @@ to_tsquery('english', 'ghost')
            ORDER BY rank DESC",
            &[],
        )
        .await?;

    println!("  「ghost」で検索:");
    for row in results {
        let movie_name: String = row.get("name");
        let rank: f32 = row.get("rank");
        println!("    {} (rank: {:.4})", movie_name.cyan(), rank);
    }

    // setweightの効果
    println!("\n{}", "【setweightによる重み付け】".yellow());
    println!("{}", "タイトルに含まれる単語を優先".dimmed());

    let weighted = client
        .query(
            "SELECT name, ts_rank(lexemes, to_tsquery('english', 'lion')) AS rank
            FROM movies
            WHERE lexemes @@ to_tsquery('english', 'lion')
            ORDER BY rank DESC",
            &[],
        )
        .await?;

    println!("  「lion」で検索:");
    for row in weighted {
        let movie_name: String = row.get("name");
        let rank: f32 = row.get("rank");
        let is_title = movie_name.to_lowercase().contains("lion");
        if is_title {
            println!(
                "    {} (rank: {:.4}) ← タイトルに含まれるので高ランク",
                movie_name.green(),
                rank
            );
        } else {
            println!("    {} (rank: {:.4})", movie_name.cyan(), rank);
        }
    }

    // ts_headline
    println!("\n{}", "【ts_headlineによるハイライト】".yellow());

    let headline = client
        .query_one(
            "SELECT name, ts_headline('english', description, to_tsquery('english', 'pirate'),
                'StartSel=<<, StopSel=>>, MaxWords=15, MinWords=5') AS highlighted
            FROM movies
            WHERE lexemes @@ to_tsquery('english', 'pirate')
            LIMIT 1",
            &[],
        )
        .await?;

    let hl_name: String = headline.get("name");
    let highlighted: String = headline.get("highlighted");
    println!("  映画: {}", hl_name.cyan());
    let colored_hl = highlighted.replace("<<", "\x1b[33m").replace(">>", "\x1b[0m");
    println!("  ハイライト: {}", colored_hl);

    // NOT検索
    println!("\n{}", "【NOT検索】".yellow());

    let not_search = client
        .query(
            "SELECT name FROM movies
            WHERE lexemes @@ to_tsquery('english', 'ghost & !shell')
            ORDER BY name",
            &[],
        )
        .await?;

    println!("  「ghost」を含むが「shell」を含まない:");
    for row in not_search {
        let movie_name: String = row.get("name");
        println!("    {}", movie_name.cyan());
    }

    // フレーズ検索 (<->)
    println!("\n{}", "【フレーズ検索 (FOLLOWED BY)】".yellow());

    let phrase = client
        .query(
            "SELECT name FROM movies
            WHERE lexemes @@ to_tsquery('english', 'computer <-> hacker')",
            &[],
        )
        .await?;

    println!("  「computer hacker」（隣接する単語）:");
    for row in phrase {
        let movie_name: String = row.get("name");
        println!("    {}", movie_name.cyan());
    }

    // GINインデックスの効果
    println!("\n{}", "【GINインデックスの効果】".yellow());

    let explain = client
        .query(
            "EXPLAIN (ANALYZE, COSTS OFF, TIMING ON)
            SELECT name FROM movies WHERE lexemes @@ to_tsquery('english', 'ghost')",
            &[],
        )
        .await?;

    for row in explain.iter().take(3) {
        let plan: String = row.get(0);
        if plan.contains("Index Scan") || plan.contains("Bitmap") {
            println!("  {}", plan.green());
        } else {
            println!("  {}", plan.dimmed());
        }
    }

    // Elasticsearchとの比較
    println!("\n{}", "【Postgres全文検索の特徴】".yellow());
    println!("  ✓ 追加インフラ不要（既存Postgresで完結）");
    println!("  ✓ トランザクション対応（データ整合性保証）");
    println!("  ✓ setweightで重み付けランキング");
    println!("  ✓ GINインデックスで高速検索");
    println!("  ⚠ 日本語は形態素解析が別途必要");
    println!("  ⚠ 大規模データ（数億件）では専用検索エンジンを検討");

    println!(
        "\n{}",
        "💡 ポイント: 多くのケースでPostgresの全文検索で十分。".dimmed()
    );
    println!(
        "{}",
        "   Elasticsearchを追加する前に、まずPostgresで試す。".dimmed()
    );

    Ok(())
}
