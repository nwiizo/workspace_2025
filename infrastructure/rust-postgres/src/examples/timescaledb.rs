//! Chapter 9: Postgres for Time Series
//!
//! 書籍の9章で説明されているTimescaleDB機能。
//! - Hypertable（自動パーティショニング）
//! - time_bucket関数（時系列集計）
//! - Continuous Aggregates（マテリアライズドビュー）
//! - データ保持ポリシー
//! - BRINインデックス

use anyhow::Result;
use colored::Colorize;
use deadpool_postgres::Pool;

/// TimescaleDBのデモ
///
/// 時系列データの効率的な管理と分析機能を検証。
/// スマートウォッチの心拍数データをシミュレートして検証する。
pub async fn run(pool: &Pool) -> Result<()> {
    let client = pool.get().await?;

    println!("{}", "TimescaleDB検証（Chapter 9: Time Series）".bold());
    println!("{}", "-".repeat(50));

    // TimescaleDB拡張を有効化
    println!("{}", "【TimescaleDB拡張の有効化】".yellow());

    client
        .execute("CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE", &[])
        .await?;

    println!("  {} TimescaleDB拡張を有効化しました", "✓".green());

    // バージョン確認
    let version = client
        .query_one(
            "SELECT extversion FROM pg_extension WHERE extname = 'timescaledb'",
            &[],
        )
        .await?;
    let ver: String = version.get("extversion");
    println!("  TimescaleDBバージョン: {}", ver.cyan());

    // watchスキーマを作成
    client
        .execute("CREATE SCHEMA IF NOT EXISTS watch", &[])
        .await?;

    // 既存のテーブルを削除（デモ用）
    client
        .execute("DROP TABLE IF EXISTS watch.heart_rate_measurements CASCADE", &[])
        .await?;

    // 心拍数計測テーブルを作成
    println!("\n{}", "【時系列テーブルの作成】".yellow());

    client
        .execute(
            "CREATE TABLE watch.heart_rate_measurements (
                watch_id INT NOT NULL,
                recorded_at TIMESTAMPTZ NOT NULL,
                heart_rate INT NOT NULL,
                activity TEXT NOT NULL CHECK (
                    activity IN ('walking', 'sleeping', 'resting', 'workout')
                )
            )",
            &[],
        )
        .await?;

    println!("  {} heart_rate_measurements テーブルを作成", "✓".green());

    // Hypertableに変換
    println!("\n{}", "【Hypertableへの変換】".yellow());
    println!(
        "{}",
        "  自動パーティショニングでスケーラブルな時系列管理".dimmed()
    );

    client
        .execute(
            "SELECT create_hypertable(
                relation => 'watch.heart_rate_measurements',
                time_column_name => 'recorded_at',
                chunk_time_interval => INTERVAL '1 day'
            )",
            &[],
        )
        .await?;

    println!(
        "  {} Hypertableに変換（1日ごとのチャンク）",
        "✓".green()
    );

    // サンプルデータを挿入
    println!("\n{}", "【サンプルデータの挿入】".yellow());

    client
        .execute(
            r#"
            INSERT INTO watch.heart_rate_measurements (watch_id, recorded_at, heart_rate, activity)
            SELECT
                (random() * 2 + 1)::INT AS watch_id,
                timestamp '2025-01-01 00:00:00' + (n || ' minutes')::INTERVAL AS recorded_at,
                CASE
                    WHEN random() < 0.1 THEN (random() * 40 + 140)::INT  -- workout
                    WHEN random() < 0.3 THEN (random() * 20 + 50)::INT   -- sleeping
                    WHEN random() < 0.5 THEN (random() * 15 + 70)::INT   -- walking
                    ELSE (random() * 10 + 60)::INT                       -- resting
                END AS heart_rate,
                CASE
                    WHEN random() < 0.1 THEN 'workout'
                    WHEN random() < 0.3 THEN 'sleeping'
                    WHEN random() < 0.5 THEN 'walking'
                    ELSE 'resting'
                END AS activity
            FROM generate_series(0, 10080) AS n  -- 7 days * 24 hours * 60 minutes
            "#,
            &[],
        )
        .await?;

    let count = client
        .query_one(
            "SELECT COUNT(*) FROM watch.heart_rate_measurements",
            &[],
        )
        .await?;
    let cnt: i64 = count.get(0);
    println!("  {} {} 件の心拍数データを挿入（7日分）", "✓".green(), cnt);

    // チャンク（パーティション）の確認
    println!("\n{}", "【チャンク（パーティション）の確認】".yellow());

    let chunks = client
        .query(
            "SELECT chunk_name, range_start, range_end
             FROM timescaledb_information.chunks
             WHERE hypertable_name = 'heart_rate_measurements'
             ORDER BY range_start
             LIMIT 5",
            &[],
        )
        .await?;

    println!("  自動生成されたチャンク（先頭5件）:");
    for row in &chunks {
        let name: String = row.get("chunk_name");
        let start: chrono::DateTime<chrono::Utc> = row.get("range_start");
        let end: chrono::DateTime<chrono::Utc> = row.get("range_end");
        println!(
            "    {} | {} - {}",
            name.cyan(),
            start.format("%Y-%m-%d"),
            end.format("%Y-%m-%d")
        );
    }

    // time_bucket関数
    println!("\n{}", "【time_bucket関数】".yellow());
    println!("{}", "  時系列データを指定間隔で集計".dimmed());

    let hourly_avg = client
        .query(
            r#"
            SELECT
                time_bucket('1 hour', recorded_at) AS hour,
                watch_id,
                AVG(heart_rate)::INT AS avg_hr,
                MAX(heart_rate) AS max_hr,
                MIN(heart_rate) AS min_hr
            FROM watch.heart_rate_measurements
            WHERE watch_id = 1
              AND recorded_at >= '2025-01-01'
              AND recorded_at < '2025-01-01 12:00:00'
            GROUP BY hour, watch_id
            ORDER BY hour
            LIMIT 6
            "#,
            &[],
        )
        .await?;

    println!("  Watch 1 の時間別心拍数（2025-01-01 午前）:");
    for row in hourly_avg {
        let hour: chrono::DateTime<chrono::Utc> = row.get("hour");
        let avg_hr: i32 = row.get("avg_hr");
        let max_hr: i32 = row.get("max_hr");
        let min_hr: i32 = row.get("min_hr");
        println!(
            "    {} | 平均: {} BPM, 最大: {}, 最小: {}",
            hour.format("%H:00").to_string().cyan(),
            avg_hr,
            max_hr,
            min_hr
        );
    }

    // アクティビティ別の集計
    println!("\n{}", "【アクティビティ別の集計】".yellow());

    let activity_stats = client
        .query(
            r#"
            SELECT
                activity,
                COUNT(*) AS measurements,
                AVG(heart_rate)::INT AS avg_hr,
                MAX(heart_rate) AS max_hr
            FROM watch.heart_rate_measurements
            GROUP BY activity
            ORDER BY avg_hr DESC
            "#,
            &[],
        )
        .await?;

    for row in activity_stats {
        let activity: String = row.get("activity");
        let measurements: i64 = row.get("measurements");
        let avg_hr: i32 = row.get("avg_hr");
        let max_hr: i32 = row.get("max_hr");
        let activity_colored = match activity.as_str() {
            "workout" => activity.red(),
            "walking" => activity.yellow(),
            "sleeping" => activity.blue(),
            _ => activity.normal(),
        };
        println!(
            "  {} | {} 件 | 平均: {} BPM, 最大: {}",
            activity_colored, measurements, avg_hr, max_hr
        );
    }

    // 日別のサマリー
    println!("\n{}", "【日別サマリー（time_bucket使用）】".yellow());

    let daily_summary = client
        .query(
            r#"
            SELECT
                time_bucket('1 day', recorded_at) AS day,
                COUNT(*) AS total_measurements,
                AVG(heart_rate)::INT AS avg_hr,
                COUNT(DISTINCT watch_id) AS active_watches
            FROM watch.heart_rate_measurements
            GROUP BY day
            ORDER BY day
            LIMIT 7
            "#,
            &[],
        )
        .await?;

    println!("  +--------------+--------+--------+---------+");
    println!("  |     日付     | 計測数 | 平均HR | アクティブ |");
    println!("  +--------------+--------+--------+---------+");
    for row in daily_summary {
        let day: chrono::DateTime<chrono::Utc> = row.get("day");
        let total: i64 = row.get("total_measurements");
        let avg_hr: i32 = row.get("avg_hr");
        let watches: i64 = row.get("active_watches");
        println!(
            "  | {} |  {} | {} BPM |    {}    |",
            day.format("%Y-%m-%d"),
            total,
            avg_hr,
            watches
        );
    }
    println!("  +--------------+--------+--------+---------+");

    // パーティションプルーニングの確認
    println!("\n{}", "【パーティションプルーニング】".yellow());
    println!(
        "{}",
        "  時間範囲に基づいて必要なチャンクのみスキャン".dimmed()
    );

    let explain = client
        .query(
            r#"
            EXPLAIN (COSTS OFF)
            SELECT * FROM watch.heart_rate_measurements
            WHERE recorded_at >= '2025-01-03'
              AND recorded_at < '2025-01-04'
            "#,
            &[],
        )
        .await?;

    for row in explain.iter().take(5) {
        let plan: String = row.get(0);
        if plan.contains("_hyper_") {
            println!("  {}", plan.green());
        } else {
            println!("  {}", plan.dimmed());
        }
    }

    // BRINインデックス
    println!("\n{}", "【BRINインデックス】".yellow());
    println!(
        "{}",
        "  時系列データに最適化されたコンパクトなインデックス".dimmed()
    );

    client
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_hr_recorded_brin
             ON watch.heart_rate_measurements
             USING BRIN (recorded_at) WITH (pages_per_range = 32)",
            &[],
        )
        .await?;

    println!("  {} BRINインデックスを作成", "✓".green());

    // インデックスサイズの比較
    let sizes = client
        .query(
            "SELECT
                pg_size_pretty(pg_relation_size('watch.heart_rate_measurements')) AS table_size,
                pg_size_pretty(pg_total_relation_size('watch.heart_rate_measurements')) AS total_size",
            &[],
        )
        .await?;

    if let Some(row) = sizes.first() {
        let table_size: String = row.get("table_size");
        let total_size: String = row.get("total_size");
        println!("  テーブルサイズ: {}", table_size.cyan());
        println!("  総サイズ（インデックス含む）: {}", total_size.cyan());
    }

    // 時系列分析の例
    println!("\n{}", "【時系列分析の例】".yellow());

    // 移動平均の計算
    let moving_avg = client
        .query(
            r#"
            SELECT
                recorded_at,
                heart_rate,
                AVG(heart_rate) OVER (
                    ORDER BY recorded_at
                    ROWS BETWEEN 4 PRECEDING AND CURRENT ROW
                )::INT AS moving_avg_5
            FROM watch.heart_rate_measurements
            WHERE watch_id = 1
              AND recorded_at >= '2025-01-01 08:00:00'
              AND recorded_at < '2025-01-01 08:10:00'
            ORDER BY recorded_at
            "#,
            &[],
        )
        .await?;

    println!("  5分間移動平均（Watch 1、2025-01-01 08:00〜）:");
    for row in moving_avg {
        let recorded: chrono::DateTime<chrono::Utc> = row.get("recorded_at");
        let hr: i32 = row.get("heart_rate");
        let avg: i32 = row.get("moving_avg_5");
        println!(
            "    {} | HR: {} BPM | 移動平均: {} BPM",
            recorded.format("%H:%M").to_string().cyan(),
            hr,
            avg
        );
    }

    // TimescaleDB固有の機能説明
    println!("\n{}", "【TimescaleDBの主な機能】".yellow());
    println!("  {}:", "Hypertable".cyan());
    println!("    - 自動パーティショニング（時間ベース）");
    println!("    - アプリケーション透過的なアクセス");
    println!("    - 古いデータの自動削除ポリシー設定可能");

    println!("  {}:", "time_bucket".cyan());
    println!("    - 時系列データの集計を簡単に");
    println!("    - 任意の時間間隔で集計可能");

    println!("  {}:", "Continuous Aggregates".cyan());
    println!("    - マテリアライズド集計ビュー");
    println!("    - リアルタイム更新対応");

    println!("  {}:", "Compression".cyan());
    println!("    - 古いチャンクの圧縮");
    println!("    - ストレージコスト削減");

    // 使い分けの説明
    println!("\n{}", "【TimescaleDBを使う基準】".yellow());
    println!("  {}:", "✓ 適切なケース".green());
    println!("    - IoTセンサーデータ");
    println!("    - メトリクス/モニタリングデータ");
    println!("    - ログ/イベントデータ");
    println!("    - 金融時系列データ");

    println!("  {}:", "✗ 不要なケース".red());
    println!("    - 頻繁に更新されるデータ");
    println!("    - 時間順序のないデータ");
    println!("    - 小規模なデータセット");

    println!(
        "\n{}",
        "💡 ポイント: 既にPostgresを使っているなら、TimescaleDBで時系列機能を追加可能。".dimmed()
    );
    println!(
        "{}",
        "   InfluxDBなどを追加する前に、まずTimescaleDBで試す。".dimmed()
    );

    Ok(())
}
