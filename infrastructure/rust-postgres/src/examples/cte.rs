//! Chapter 3: CTEの検証
//!
//! 書籍の3.2節で説明されているCTE（Common Table Expressions）の検証。
//! CTEは「読みやすいサブクエリ」以上の価値がある。

use anyhow::Result;
use colored::Colorize;
use deadpool_postgres::Pool;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct SkippedSong {
    title: String,
    duration: String,
    min_play: String,
    play_count: i64,
}

/// CTEのデモ
///
/// 書籍の例では「半分以下の時間しか再生されなかった曲」を
/// ランキングしていた。同様のクエリを実行する。
pub async fn run(pool: &Pool) -> Result<()> {
    let client = pool.get().await?;

    println!("{}", "CTE（Common Table Expressions）の検証".bold());
    println!("{}", "-".repeat(50));

    // CTEを使った複雑なクエリ
    println!("{}", "【基本的なCTE】".yellow());
    println!("{}", "「半分以下しか再生されなかった曲」を検索".dimmed());

    let rows = client
        .query(
            "WITH plays_cte AS (
                SELECT s.title, s.duration, p.play_duration, p.user_id
                FROM streaming.plays p
                JOIN streaming.songs s ON p.song_id = s.id
                WHERE p.play_duration < (s.duration / 2)
            ),
            user_play_counts AS (
                SELECT
                    title,
                    duration,
                    COUNT(DISTINCT user_id) AS user_count,
                    MIN(play_duration) AS min_play_duration,
                    COUNT(*) AS total_play_count
                FROM plays_cte
                GROUP BY title, duration
            )
            SELECT title, duration, min_play_duration, total_play_count
            FROM user_play_counts
            WHERE user_count >= 2
            ORDER BY min_play_duration ASC
            LIMIT 5",
            &[],
        )
        .await?;

    let songs: Vec<SkippedSong> = rows
        .iter()
        .map(|row| SkippedSong {
            title: row.get("title"),
            duration: format!("{}秒", row.get::<_, i32>("duration")),
            min_play: format!("{}秒", row.get::<_, i32>("min_play_duration")),
            play_count: row.get("total_play_count"),
        })
        .collect();

    if songs.is_empty() {
        println!("  (該当するデータがありません)");
    } else {
        println!("{}", Table::new(&songs));
    }

    // EXPLAINでCTEの最適化を確認
    println!("\n{}", "【CTEの実行計画】".yellow());
    let explain = client
        .query(
            "EXPLAIN (FORMAT TEXT)
            WITH plays_cte AS (
                SELECT s.title, s.duration, p.play_duration, p.user_id
                FROM streaming.plays p
                JOIN streaming.songs s ON p.song_id = s.id
            )
            SELECT title, COUNT(*) FROM plays_cte GROUP BY title",
            &[],
        )
        .await?;

    println!("  PostgresはCTEをインライン展開して最適化:");
    for row in explain.iter().take(5) {
        let plan: String = row.get(0);
        println!("    {}", plan.dimmed());
    }

    // Data-modifying CTE
    println!("\n{}", "【Data-modifying CTE】".yellow());
    println!("{}", "UPDATEの結果を即座に活用するパターン".dimmed());

    // テスト用に再生時間を更新して、結果を確認
    let result = client
        .query_one(
            "WITH updated_play AS (
                UPDATE streaming.plays
                SET play_duration = play_duration + 10
                WHERE id = 1
                RETURNING id, song_id, play_duration
            )
            SELECT
                s.title,
                s.duration AS song_duration,
                up.play_duration AS new_play_duration,
                CASE
                    WHEN up.play_duration >= s.duration THEN '完走'
                    WHEN up.play_duration >= s.duration / 2 THEN '半分以上'
                    ELSE 'スキップ'
                END AS status
            FROM updated_play up
            JOIN streaming.songs s ON s.id = up.song_id",
            &[],
        )
        .await?;

    let title: String = result.get("title");
    let song_dur: i32 = result.get("song_duration");
    let play_dur: i32 = result.get("new_play_duration");
    let status: String = result.get("status");

    println!(
        "  更新された再生: {} ({}秒/{}秒) → {}",
        title.cyan(),
        play_dur,
        song_dur,
        status.green()
    );

    // 複数CTEの連鎖
    println!("\n{}", "【複数CTEの連鎖】".yellow());
    println!("{}", "段階的にデータを絞り込む".dimmed());

    let chain = client
        .query(
            "WITH
            -- Step 1: 全再生を取得
            all_plays AS (
                SELECT p.*, s.title, s.duration AS song_duration
                FROM streaming.plays p
                JOIN streaming.songs s ON p.song_id = s.id
            ),
            -- Step 2: 完走した再生のみ抽出
            completed_plays AS (
                SELECT * FROM all_plays
                WHERE play_duration >= song_duration * 0.9
            ),
            -- Step 3: ユーザー別の完走数を集計
            user_completions AS (
                SELECT user_id, COUNT(*) AS completion_count
                FROM completed_plays
                GROUP BY user_id
            )
            SELECT
                a.name AS user_name,
                uc.completion_count
            FROM user_completions uc
            JOIN customers.accounts a ON a.id = uc.user_id
            ORDER BY completion_count DESC
            LIMIT 5",
            &[],
        )
        .await?;

    println!("  完走数トップ5ユーザー:");
    for row in chain {
        let name: String = row.get("user_name");
        let count: i64 = row.get("completion_count");
        println!("    {} - {} 曲完走", name.cyan(), count);
    }

    println!(
        "\n{}",
        "💡 ポイント: CTEは読みやすさだけでなく、".dimmed()
    );
    println!(
        "{}",
        "   Data-modifyingで更新と参照を1クエリに。".dimmed()
    );
    println!(
        "{}",
        "   PostgresはCTEを最適化するので、パフォーマンスも問題ない。".dimmed()
    );

    Ok(())
}
