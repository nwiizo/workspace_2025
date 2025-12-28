//! Chapter 5: JSON機能の詳細検証
//!
//! 書籍の5章で説明されているJSON機能の詳細。
//! - json vs jsonb
//! - JSONクエリ演算子（->, ->>, @>, ?）
//! - JSON Path Expressions
//! - jsonb_set による部分更新
//! - GINインデックス（default vs jsonb_path_ops）

use anyhow::Result;
use colored::Colorize;
use deadpool_postgres::Pool;
use serde_json::Value;

/// JSON機能の詳細デモ
///
/// 書籍ではピザ注文システムの例が使われている。
/// ここではコーヒーのカスタマイズオーダーで検証。
pub async fn run(pool: &Pool) -> Result<()> {
    let client = pool.get().await?;

    println!("{}", "JSON機能の詳細検証（Chapter 5）".bold());
    println!("{}", "-".repeat(50));

    // テスト用テーブルを作成
    client
        .execute(
            "CREATE TEMP TABLE IF NOT EXISTS coffee_orders (
                id SERIAL PRIMARY KEY,
                customer_id INTEGER NOT NULL,
                coffee JSONB NOT NULL,
                price NUMERIC(5,2) NOT NULL,
                created_at TIMESTAMP DEFAULT NOW()
            )",
            &[],
        )
        .await?;

    // サンプルデータを挿入
    client
        .execute(
            r#"
            INSERT INTO coffee_orders (customer_id, coffee, price) VALUES
            (1, '{"type": "latte", "size": "large", "milk": "oat", "extras": {"shots": 2, "syrup": "vanilla"}}', 6.50),
            (2, '{"type": "cappuccino", "size": "medium", "milk": "whole", "extras": {"shots": 1}}', 4.50),
            (3, '{"type": "americano", "size": "small", "milk": null, "extras": {"shots": 3}}', 3.50),
            (1, '{"type": "mocha", "size": "large", "milk": "almond", "extras": {"shots": 2, "syrup": "chocolate", "whip": true}}', 7.00),
            (4, '{"type": "latte", "size": "medium", "milk": "oat", "extras": {"shots": 1, "syrup": "caramel"}}', 5.50)
            "#,
            &[],
        )
        .await?;

    // -> vs ->> の違い
    println!("{}", "【-> vs ->> の違い】".yellow());
    println!("{}", "->はJSON型、->>はTEXT型を返す".dimmed());

    let arrows = client
        .query_one(
            r#"SELECT
                coffee->'type' AS json_type,
                coffee->>'type' AS text_type,
                pg_typeof(coffee->'type')::TEXT AS json_typeof,
                pg_typeof(coffee->>'type')::TEXT AS text_typeof
            FROM coffee_orders LIMIT 1"#,
            &[],
        )
        .await?;

    let json_type: Value = arrows.get("json_type");
    let text_type: String = arrows.get("text_type");
    let json_typeof: String = arrows.get("json_typeof");
    let text_typeof: String = arrows.get("text_typeof");

    println!("  coffee->'type'  = {} (型: {})", json_type, json_typeof.cyan());
    println!("  coffee->>'type' = {} (型: {})", text_type, text_typeof.cyan());

    // ネストしたJSONへのアクセス
    println!("\n{}", "【ネストしたJSONへのアクセス】".yellow());

    let nested = client
        .query(
            r#"SELECT
                coffee->>'type' AS coffee_type,
                coffee->'extras'->>'shots' AS shots,
                coffee->'extras'->>'syrup' AS syrup
            FROM coffee_orders
            WHERE coffee->'extras' ? 'syrup'"#,
            &[],
        )
        .await?;

    println!("  シロップ入りの注文:");
    for row in nested {
        let coffee_type: String = row.get("coffee_type");
        let shots: Option<String> = row.get("shots");
        let syrup: Option<String> = row.get("syrup");
        println!(
            "    {} - {}ショット, {}シロップ",
            coffee_type.cyan(),
            shots.unwrap_or_default(),
            syrup.unwrap_or("なし".to_string())
        );
    }

    // @> 包含演算子
    println!("\n{}", "【@> 包含演算子】".yellow());
    println!("{}", "JSONオブジェクトの包含関係を確認".dimmed());

    let contains = client
        .query(
            r#"SELECT coffee->>'type' AS coffee_type, coffee->>'milk' AS milk
            FROM coffee_orders
            WHERE coffee @> '{"milk": "oat"}'"#,
            &[],
        )
        .await?;

    println!("  オートミルクの注文:");
    for row in contains {
        let coffee_type: String = row.get("coffee_type");
        let milk: Option<String> = row.get("milk");
        println!("    {} ({})", coffee_type.cyan(), milk.unwrap_or_default());
    }

    // 複雑な条件
    let complex = client
        .query_one(
            r#"SELECT COUNT(*) as count
            FROM coffee_orders
            WHERE coffee @> '{"size": "large", "extras": {"shots": 2}}'"#,
            &[],
        )
        .await?;
    let count: i64 = complex.get("count");
    println!("  ラージサイズ + 2ショット: {} 件", count.to_string().green());

    // ? キー存在確認
    println!("\n{}", "【? キー存在確認演算子】".yellow());

    let has_whip = client
        .query_one(
            r#"SELECT COUNT(*) as count FROM coffee_orders WHERE coffee->'extras' ? 'whip'"#,
            &[],
        )
        .await?;
    let whip_count: i64 = has_whip.get("count");
    println!("  ホイップ付き注文: {} 件", whip_count.to_string().green());

    // JSON Path Expression
    println!("\n{}", "【JSON Path Expression】".yellow());
    println!("{}", "より柔軟なクエリが可能".dimmed());

    let path_result = client
        .query(
            r#"SELECT coffee->>'type' AS coffee_type
            FROM coffee_orders
            WHERE jsonb_path_exists(coffee, '$.extras ? (@.shots > 1)')"#,
            &[],
        )
        .await?;

    println!("  2ショット以上の注文（JSON Path使用）:");
    for row in path_result {
        let coffee_type: String = row.get("coffee_type");
        println!("    {}", coffee_type.cyan());
    }

    // jsonb_set による部分更新
    println!("\n{}", "【jsonb_set による部分更新】".yellow());

    let before = client
        .query_one("SELECT coffee FROM coffee_orders WHERE id = 1", &[])
        .await?;
    let before_json: Value = before.get(0);
    println!("  更新前: {}", serde_json::to_string(&before_json)?);

    client
        .execute(
            r#"UPDATE coffee_orders
            SET coffee = jsonb_set(coffee, '{extras,syrup}', '"hazelnut"')
            WHERE id = 1"#,
            &[],
        )
        .await?;

    let after = client
        .query_one("SELECT coffee FROM coffee_orders WHERE id = 1", &[])
        .await?;
    let after_json: Value = after.get(0);
    println!("  更新後: {}", serde_json::to_string(&after_json)?);

    // 元に戻す
    client
        .execute(
            r#"UPDATE coffee_orders SET coffee = $1 WHERE id = 1"#,
            &[&before_json],
        )
        .await?;

    // GINインデックスの比較
    println!("\n{}", "【GINインデックスの比較】".yellow());

    // デフォルトGIN
    client
        .execute(
            "CREATE INDEX idx_coffee_gin ON coffee_orders USING GIN(coffee)",
            &[],
        )
        .await?;

    // jsonb_path_ops GIN
    client
        .execute(
            "CREATE INDEX idx_coffee_gin_pathops ON coffee_orders USING GIN(coffee jsonb_path_ops)",
            &[],
        )
        .await?;

    let sizes = client
        .query(
            "SELECT indexname, pg_size_pretty(pg_relation_size(indexname::regclass)) AS size
            FROM pg_indexes
            WHERE tablename = 'coffee_orders' AND indexname LIKE 'idx_coffee%'",
            &[],
        )
        .await?;

    for row in sizes {
        let name: String = row.get("indexname");
        let size: String = row.get("size");
        println!("  {} - {}", name.cyan(), size);
    }

    // 使い分けの説明
    println!("\n{}", "【GINインデックスの使い分け】".yellow());
    println!("  {}:", "default GIN".cyan());
    println!("    - @>, ?, @?, @@ をサポート");
    println!("    - キー存在確認（?）が必要な場合に使用");
    println!("  {}:", "jsonb_path_ops".cyan());
    println!("    - @>, @?, @@ をサポート（?は非対応）");
    println!("    - サイズが小さく、検索が速い");
    println!("    - 包含検索のみの場合に推奨");

    println!(
        "\n{}",
        "💡 ポイント: JSONBでハイブリッドアプローチを実現。".dimmed()
    );
    println!(
        "{}",
        "   検索頻度の高いフィールドは通常型、柔軟性が必要な部分はJSONB。".dimmed()
    );
    println!(
        "{}",
        "   GINインデックスで高速検索、jsonb_setで部分更新。".dimmed()
    );

    Ok(())
}
