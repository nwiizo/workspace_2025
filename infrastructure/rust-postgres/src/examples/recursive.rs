//! Chapter 3: Recursive Queryの検証
//!
//! 書籍の3.5節で説明されているRecursive Queryの検証。
//! 連続再生のリンクリスト構造をたどる例。

use anyhow::Result;
use colored::Colorize;
use deadpool_postgres::Pool;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct PlaySequence {
    #[tabled(rename = "順番")]
    seq_num: i32,
    #[tabled(rename = "曲名")]
    title: String,
    #[tabled(rename = "再生時間")]
    duration: String,
    #[tabled(rename = "再生済み")]
    played: String,
}

/// Recursive Queryのデモ
///
/// streaming.playsテーブルのplayed_afterカラムを使って、
/// 連続再生のシーケンスをたどる。
///
/// これはリンクリスト構造をSQLで処理する典型的な例。
pub async fn run(pool: &Pool) -> Result<()> {
    let client = pool.get().await?;

    println!("{}", "Recursive Query（再帰クエリ）の検証".bold());
    println!("{}", "-".repeat(50));

    // 連続再生のシーケンスを持つ再生を探す
    println!("{}", "【連続再生シーケンスの検索】".yellow());

    // まず、連続再生の起点（played_afterがNULL）を探す
    let start_plays = client
        .query(
            "SELECT p.id, p.user_id, c.name AS user_name, COUNT(next.id) AS sequence_length
            FROM streaming.plays p
            JOIN customers.accounts c ON c.id = p.user_id
            LEFT JOIN streaming.plays next ON next.played_after = p.id
            WHERE p.played_after IS NULL
            GROUP BY p.id, p.user_id, c.name
            HAVING COUNT(next.id) > 0
            ORDER BY sequence_length DESC
            LIMIT 5",
            &[],
        )
        .await?;

    println!("  連続再生セッションの起点:");
    for row in &start_plays {
        let id: i32 = row.get("id");
        let user: String = row.get("user_name");
        let length: i64 = row.get("sequence_length");
        println!("    再生ID: {} ({}) - 後続: {}曲", id, user.cyan(), length);
    }

    // 特定のセッションをRecursive Queryでたどる
    if !start_plays.is_empty() {
        let start_id: i32 = start_plays[0].get("id");

        println!(
            "\n{}",
            format!("【再生ID {} からの連続再生シーケンス】", start_id)
                .yellow()
                .bold()
        );

        let sequence = client
            .query(
                "WITH RECURSIVE play_sequence AS (
                    -- 非再帰項: 起点となる再生
                    SELECT
                        p.id,
                        p.user_id,
                        p.song_id,
                        p.play_start_time,
                        p.play_duration,
                        p.played_after,
                        1 AS seq_num
                    FROM streaming.plays p
                    WHERE p.id = $1

                    UNION ALL

                    -- 再帰項: 次の再生を探す
                    SELECT
                        p.id,
                        p.user_id,
                        p.song_id,
                        p.play_start_time,
                        p.play_duration,
                        p.played_after,
                        ps.seq_num + 1
                    FROM streaming.plays p
                    JOIN play_sequence ps ON p.played_after = ps.id
                )
                SELECT
                    ps.seq_num,
                    s.title,
                    s.duration AS song_duration,
                    ps.play_duration
                FROM play_sequence ps
                JOIN streaming.songs s ON s.id = ps.song_id
                ORDER BY ps.seq_num",
                &[&start_id],
            )
            .await?;

        let plays: Vec<PlaySequence> = sequence
            .iter()
            .map(|row| {
                let song_dur: i32 = row.get("song_duration");
                let play_dur: i32 = row.get("play_duration");
                let pct = (play_dur as f64 / song_dur as f64 * 100.0) as i32;

                PlaySequence {
                    seq_num: row.get("seq_num"),
                    title: row.get("title"),
                    duration: format!("{}秒", song_dur),
                    played: format!("{}秒 ({}%)", play_dur, pct),
                }
            })
            .collect();

        println!("{}", Table::new(&plays));
    }

    // 階層構造の別の例: 注文アイテムの「バンドル」を再帰的に展開
    println!("\n{}", "【Recursive CTEの実行フロー】".yellow());
    println!("{}", "擬似コードで表現:".dimmed());
    println!(
        "{}",
        r#"
    1. 非再帰項を実行 → 初期結果をworking_tableに格納
    2. while working_table is not empty:
         a. 再帰項を実行（working_tableを参照）
         b. 結果をintermediate_tableに格納
         c. intermediate_tableを最終結果に追加
         d. working_table = intermediate_table
    3. 最終結果を返す
    "#
        .dimmed()
    );

    // 循環検出のデモ
    println!("\n{}", "【循環検出の対策】".yellow());
    println!("{}", "配列を使って訪問済みノードを追跡:".dimmed());

    let cycle_safe = client
        .query(
            "WITH RECURSIVE play_sequence AS (
                SELECT
                    p.id,
                    p.song_id,
                    1 AS depth,
                    ARRAY[p.id] AS path,  -- 訪問済みIDを配列で追跡
                    false AS is_cycle
                FROM streaming.plays p
                WHERE p.id = $1

                UNION ALL

                SELECT
                    p.id,
                    p.song_id,
                    ps.depth + 1,
                    ps.path || p.id,
                    p.id = ANY(ps.path)  -- 循環検出
                FROM streaming.plays p
                JOIN play_sequence ps ON p.played_after = ps.id
                WHERE NOT p.id = ANY(ps.path)  -- 循環を避ける
                  AND ps.depth < 100  -- 最大深度制限
            )
            SELECT COUNT(*) AS total_plays, MAX(depth) AS max_depth
            FROM play_sequence",
            &[&start_plays.first().map(|r| r.get::<_, i32>("id")).unwrap_or(1)],
        )
        .await?;

    if let Some(row) = cycle_safe.first() {
        let count: i64 = row.get("total_plays");
        let depth: i32 = row.get("max_depth");
        println!(
            "  循環検出付きで探索: {} 件, 最大深度: {}",
            count.to_string().green(),
            depth
        );
    }

    println!(
        "\n{}",
        "💡 ポイント: Recursive Queryでリンクリスト/階層構造を".dimmed()
    );
    println!(
        "{}",
        "   1回のクエリで処理。N+1問題を回避できる。".dimmed()
    );
    println!(
        "{}",
        "   循環がある場合は配列で訪問済みを追跡して防止。".dimmed()
    );

    Ok(())
}
