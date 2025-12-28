//! JSONB機能の検証
//!
//! PostgreSQLのJSONB型は、「Just Use Postgres」思想の典型例。
//! ドキュメントDBを別途用意しなくても、JSONデータを効率的に扱える。

use anyhow::Result;
use colored::Colorize;
use deadpool_postgres::Pool;
use serde_json::Value;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct CustomerPreference {
    name: String,
    theme: String,
    language: String,
    notifications: String,
}

/// JSONBのデモ
///
/// customers.accountsのpreferencesカラムと
/// products.catalogのtagsカラムを使って、
/// JSONBの各種操作を検証する。
pub async fn run(pool: &Pool) -> Result<()> {
    let client = pool.get().await?;

    println!("{}", "JSONB機能の検証".bold());
    println!("{}", "-".repeat(50));

    // JSONBフィールドの抽出
    println!("{}", "【JSONBフィールドの抽出】".yellow());

    let rows = client
        .query(
            "SELECT
                name,
                preferences->>'theme' AS theme,
                preferences->>'language' AS language,
                preferences->>'notifications' AS notifications
            FROM customers.accounts
            WHERE preferences IS NOT NULL
            ORDER BY id
            LIMIT 10",
            &[],
        )
        .await?;

    let prefs: Vec<CustomerPreference> = rows
        .iter()
        .map(|row| CustomerPreference {
            name: row.get("name"),
            theme: row.get::<_, Option<String>>("theme").unwrap_or_default(),
            language: row.get::<_, Option<String>>("language").unwrap_or_default(),
            notifications: row.get::<_, Option<String>>("notifications").unwrap_or_default(),
        })
        .collect();

    println!("{}", Table::new(&prefs));

    // JSONBでのフィルタリング
    println!("\n{}", "【JSONBによるフィルタリング】".yellow());

    let dark_theme = client
        .query(
            "SELECT name, preferences
            FROM customers.accounts
            WHERE preferences->>'theme' = 'dark'
            LIMIT 5",
            &[],
        )
        .await?;

    println!("  ダークテーマを選択している顧客:");
    for row in dark_theme {
        let name: String = row.get("name");
        println!("    - {}", name.cyan());
    }

    // JSONB配列の検索（GINインデックス活用）
    println!("\n{}", "【JSONB配列の検索】".yellow());
    println!("{}", "GINインデックスを使った効率的な検索".dimmed());

    let tagged = client
        .query(
            "SELECT name, category, tags
            FROM products.catalog
            WHERE tags @> '[\"hot\"]'::jsonb
            ORDER BY name",
            &[],
        )
        .await?;

    println!("  'hot'タグを持つ商品:");
    for row in tagged {
        let name: String = row.get("name");
        let category: String = row.get("category");
        let tags: Value = row.get("tags");
        println!("    {} ({}) - {:?}", name.cyan(), category, tags);
    }

    // 複数タグでの検索
    println!("\n  複数タグ検索 ('hot' AND 'milk'):");
    let multi_tagged = client
        .query(
            "SELECT name, tags
            FROM products.catalog
            WHERE tags @> '[\"hot\", \"milk\"]'::jsonb",
            &[],
        )
        .await?;

    for row in multi_tagged {
        let name: String = row.get("name");
        let tags: Value = row.get("tags");
        println!("    {} - {:?}", name.cyan(), tags);
    }

    // JSONBの更新
    println!("\n{}", "【JSONBの更新】".yellow());

    // テスト用に更新して戻す
    let before = client
        .query_one(
            "SELECT preferences FROM customers.accounts WHERE id = 1",
            &[],
        )
        .await?;
    let before_prefs: Value = before.get(0);
    println!("  更新前: {}", serde_json::to_string_pretty(&before_prefs)?);

    // jsonb_set で部分更新
    client
        .execute(
            "UPDATE customers.accounts
            SET preferences = jsonb_set(preferences, '{theme}', '\"system\"')
            WHERE id = 1",
            &[],
        )
        .await?;

    let after = client
        .query_one(
            "SELECT preferences FROM customers.accounts WHERE id = 1",
            &[],
        )
        .await?;
    let after_prefs: Value = after.get(0);
    println!("  更新後: {}", serde_json::to_string_pretty(&after_prefs)?);

    // 元に戻す
    client
        .execute(
            "UPDATE customers.accounts SET preferences = $1 WHERE id = 1",
            &[&before_prefs],
        )
        .await?;

    // JSONB集計
    println!("\n{}", "【JSONB集計】".yellow());

    let stats = client
        .query(
            "SELECT
                preferences->>'theme' AS theme,
                COUNT(*) AS count
            FROM customers.accounts
            WHERE preferences->>'theme' IS NOT NULL
            GROUP BY preferences->>'theme'
            ORDER BY count DESC",
            &[],
        )
        .await?;

    println!("  テーマ別顧客数:");
    for row in stats {
        let theme: String = row.get("theme");
        let count: i64 = row.get("count");
        println!("    {} - {} 人", theme.cyan(), count);
    }

    // jsonb_agg で配列に集約
    println!("\n{}", "【jsonb_aggによる集約】".yellow());

    let agg = client
        .query_one(
            "SELECT jsonb_agg(DISTINCT preferences->>'language') AS languages
            FROM customers.accounts
            WHERE preferences->>'language' IS NOT NULL",
            &[],
        )
        .await?;

    let languages: Value = agg.get(0);
    println!("  使用言語一覧: {}", languages);

    // インデックスの確認
    println!("\n{}", "【GINインデックスの効果】".yellow());

    let explain = client
        .query(
            "EXPLAIN (FORMAT TEXT)
            SELECT * FROM products.catalog WHERE tags @> '[\"hot\"]'::jsonb",
            &[],
        )
        .await?;

    println!("  実行計画:");
    for row in explain {
        let plan: String = row.get(0);
        println!("    {}", plan.dimmed());
    }

    println!(
        "\n{}",
        "💡 ポイント: JSONBでドキュメントDBの代替が可能。".dimmed()
    );
    println!(
        "{}",
        "   GINインデックスで高速検索、jsonb_setで部分更新。".dimmed()
    );
    println!(
        "{}",
        "   リレーショナルデータとJSONデータを同じDBで管理。".dimmed()
    );

    Ok(())
}
