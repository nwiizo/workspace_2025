//! Chapter 4: インデックスの検証
//!
//! 書籍の4章で説明されている各種インデックスの検証。
//! - B-tree vs Hash
//! - 複合インデックス
//! - Covering Index
//! - Partial Index
//! - Expression Index

use anyhow::Result;
use colored::Colorize;
use deadpool_postgres::Pool;

/// インデックスのデモ
///
/// EXPLAIN ANALYZEを使って各種インデックスの効果を検証する。
pub async fn run(pool: &Pool) -> Result<()> {
    let client = pool.get().await?;

    println!("{}", "インデックス戦略の検証（Chapter 4）".bold());
    println!("{}", "-".repeat(50));

    // テスト用テーブルを作成
    client
        .execute(
            "CREATE TEMP TABLE IF NOT EXISTS game_player_stats (
                id SERIAL PRIMARY KEY,
                username VARCHAR(50) NOT NULL,
                region VARCHAR(10) NOT NULL,
                score INTEGER NOT NULL,
                win_count INTEGER NOT NULL,
                loss_count INTEGER NOT NULL,
                play_time INTERVAL NOT NULL,
                champion_title VARCHAR(20)
            )",
            &[],
        )
        .await?;

    // テストデータを生成
    client
        .execute(
            "INSERT INTO game_player_stats (username, region, score, win_count, loss_count, play_time, champion_title)
            SELECT
                'player_' || id,
                (ARRAY['NA', 'EU', 'ASIA', 'SA'])[floor(random() * 4 + 1)::int],
                floor(random() * 10000)::int,
                floor(random() * 1000)::int,
                floor(random() * 500)::int,
                (random() * 500 || ' hours')::interval,
                (ARRAY['Bronze', 'Silver', 'Gold', 'Platinum', 'Diamond'])[floor(random() * 5 + 1)::int]
            FROM generate_series(1, 10000) AS id
            ON CONFLICT DO NOTHING",
            &[],
        )
        .await?;

    // ANALYZEでstatisticsを更新
    client.execute("ANALYZE game_player_stats", &[]).await?;

    println!("{}", "【B-tree vs Hash インデックス】".yellow());
    println!("{}", "等価検索（=）ではHashが最適".dimmed());

    // インデックスなしのクエリ
    let explain_no_idx = client
        .query(
            "EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF)
            SELECT * FROM game_player_stats WHERE champion_title = 'Diamond'",
            &[],
        )
        .await?;

    println!("\n  インデックスなし:");
    for row in explain_no_idx.iter().take(3) {
        let plan: String = row.get(0);
        println!("    {}", plan.dimmed());
    }

    // Hashインデックスを作成
    client
        .execute(
            "CREATE INDEX idx_champion_hash ON game_player_stats USING hash(champion_title)",
            &[],
        )
        .await?;

    let explain_hash = client
        .query(
            "EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF)
            SELECT * FROM game_player_stats WHERE champion_title = 'Diamond'",
            &[],
        )
        .await?;

    println!("\n  Hashインデックスあり:");
    for row in explain_hash.iter().take(3) {
        let plan: String = row.get(0);
        println!("    {}", plan.dimmed());
    }

    // 複合インデックス
    println!("\n{}", "【複合インデックス】".yellow());
    println!("{}", "先頭カラム（leading column）が重要".dimmed());

    client
        .execute(
            "CREATE INDEX idx_region_score ON game_player_stats (region, score DESC)",
            &[],
        )
        .await?;

    // 先頭カラムを含むクエリ
    let explain_leading = client
        .query(
            "EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF)
            SELECT * FROM game_player_stats WHERE region = 'NA' AND score > 5000",
            &[],
        )
        .await?;

    println!("\n  region + score（先頭カラムあり）:");
    for row in explain_leading.iter().take(3) {
        let plan: String = row.get(0);
        let is_index = plan.contains("Index");
        if is_index {
            println!("    {}", plan.green());
        } else {
            println!("    {}", plan.dimmed());
        }
    }

    // 先頭カラムを含まないクエリ
    let explain_no_leading = client
        .query(
            "EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF)
            SELECT * FROM game_player_stats WHERE score > 8000",
            &[],
        )
        .await?;

    println!("\n  scoreのみ（先頭カラムなし）:");
    for row in explain_no_leading.iter().take(3) {
        let plan: String = row.get(0);
        let is_seq = plan.contains("Seq Scan");
        if is_seq {
            println!("    {} (インデックス使われず)", plan.red());
        } else {
            println!("    {}", plan.dimmed());
        }
    }

    // Covering Index
    println!("\n{}", "【Covering Index（INCLUDE句）】".yellow());
    println!("{}", "テーブルアクセスをゼロに".dimmed());

    client
        .execute(
            "CREATE INDEX idx_covering ON game_player_stats (region, score DESC) INCLUDE (username)",
            &[],
        )
        .await?;

    let explain_covering = client
        .query(
            "EXPLAIN (ANALYZE, BUFFERS, COSTS OFF, TIMING OFF)
            SELECT username, score FROM game_player_stats WHERE region = 'EU' ORDER BY score DESC LIMIT 10",
            &[],
        )
        .await?;

    println!("\n  usernameをINCLUDEしたインデックス:");
    for row in &explain_covering {
        let plan: String = row.get(0);
        if plan.contains("Index Only Scan") {
            println!("    {} ← テーブルアクセスなし！", plan.green());
        } else if plan.contains("Heap Fetches: 0") {
            println!("    {} ← 完全にインデックスのみ", plan.green());
        } else {
            println!("    {}", plan.dimmed());
        }
    }

    // Partial Index
    println!("\n{}", "【Partial Index（WHERE句付き）】".yellow());
    println!("{}", "必要な部分だけインデックス化".dimmed());

    client
        .execute(
            "CREATE INDEX idx_casual_players ON game_player_stats (play_time) WHERE play_time <= '50 hours'",
            &[],
        )
        .await?;

    let explain_partial = client
        .query(
            "EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF)
            SELECT * FROM game_player_stats WHERE play_time <= '50 hours'",
            &[],
        )
        .await?;

    println!("\n  カジュアルプレイヤー（50時間以下）の検索:");
    for row in explain_partial.iter().take(3) {
        let plan: String = row.get(0);
        if plan.contains("idx_casual_players") {
            println!("    {}", plan.green());
        } else {
            println!("    {}", plan.dimmed());
        }
    }

    // Expression Index
    println!("\n{}", "【Expression Index】".yellow());
    println!("{}", "計算結果をインデックス化".dimmed());

    client
        .execute(
            "CREATE INDEX idx_perf_margin ON game_player_stats ((win_count - loss_count))",
            &[],
        )
        .await?;

    let explain_expr = client
        .query(
            "EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF)
            SELECT username, win_count, loss_count, (win_count - loss_count) AS margin
            FROM game_player_stats
            WHERE (win_count - loss_count) BETWEEN 300 AND 500",
            &[],
        )
        .await?;

    println!("\n  勝敗マージン（win - loss）での検索:");
    for row in explain_expr.iter().take(3) {
        let plan: String = row.get(0);
        if plan.contains("idx_perf_margin") {
            println!("    {}", plan.green());
        } else {
            println!("    {}", plan.dimmed());
        }
    }

    // インデックスサイズの確認
    println!("\n{}", "【インデックスサイズ】".yellow());

    let sizes = client
        .query(
            "SELECT indexname, pg_size_pretty(pg_relation_size(indexname::regclass)) AS size
            FROM pg_indexes
            WHERE tablename = 'game_player_stats'
            ORDER BY pg_relation_size(indexname::regclass) DESC",
            &[],
        )
        .await?;

    for row in sizes {
        let name: String = row.get("indexname");
        let size: String = row.get("size");
        println!("  {} - {}", name.cyan(), size);
    }

    // Over-Indexingの警告
    let index_count = client
        .query_one(
            "SELECT COUNT(*) FROM pg_indexes WHERE tablename = 'game_player_stats'",
            &[],
        )
        .await?;
    let count: i64 = index_count.get(0);

    println!("\n{}", format!("⚠️  現在のインデックス数: {}", count).yellow());
    if count > 5 {
        println!(
            "{}",
            "   Over-Indexingの可能性あり。更新コストに注意。".red()
        );
    }

    println!(
        "\n{}",
        "💡 ポイント: インデックスは万能薬ではない。".dimmed()
    );
    println!(
        "{}",
        "   EXPLAIN ANALYZEでボトルネックを特定してから作成。".dimmed()
    );
    println!(
        "{}",
        "   Covering/Partial/Expressionで最適化可能。Over-Indexingに注意。".dimmed()
    );

    Ok(())
}
