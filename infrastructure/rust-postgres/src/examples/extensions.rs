//! Chapter 7: 拡張機能の検証
//!
//! 書籍の7章で説明されているPostgreSQL拡張機能。
//! - pgcrypto（暗号化）
//! - pg_trgm（トライグラム類似度検索）
//! - uuid-ossp（UUID生成）
//!
//! 注意: pgvectorやPostGISは別途インストールが必要

use anyhow::Result;
use colored::Colorize;
use deadpool_postgres::Pool;

/// 拡張機能のデモ
///
/// PostgreSQLにビルトインされている拡張機能を検証。
pub async fn run(pool: &Pool) -> Result<()> {
    let client = pool.get().await?;

    println!("{}", "拡張機能の検証（Chapter 7）".bold());
    println!("{}", "-".repeat(50));

    // 利用可能な拡張を表示
    println!("{}", "【利用可能な拡張機能】".yellow());

    let extensions = client
        .query(
            "SELECT name, comment FROM pg_available_extensions
            WHERE name IN ('pgcrypto', 'pg_trgm', 'uuid-ossp', 'hstore', 'ltree', 'citext')
            ORDER BY name",
            &[],
        )
        .await?;

    for row in &extensions {
        let name: String = row.get("name");
        let comment: Option<String> = row.get("comment");
        println!(
            "  {} - {}",
            name.cyan(),
            comment.unwrap_or_default().dimmed()
        );
    }

    // pgcrypto を有効化
    println!("\n{}", "【pgcrypto: 暗号化機能】".yellow());

    client
        .execute("CREATE EXTENSION IF NOT EXISTS pgcrypto", &[])
        .await?;

    // パスワードのハッシュ化
    let hash_result = client
        .query_one(
            "SELECT crypt('mypassword', gen_salt('bf', 8)) AS hashed",
            &[],
        )
        .await?;
    let hashed: String = hash_result.get("hashed");
    println!("  bcryptでハッシュ化: {}", hashed.dimmed());

    // パスワード検証
    let verify_ok = client
        .query_one(
            "SELECT crypt('mypassword', $1) = $1 AS verified",
            &[&hashed],
        )
        .await?;
    let verified: bool = verify_ok.get("verified");
    println!(
        "  正しいパスワードで検証: {}",
        if verified {
            "✓ 一致".green()
        } else {
            "✗ 不一致".red()
        }
    );

    let verify_ng = client
        .query_one(
            "SELECT crypt('wrongpassword', $1) = $1 AS verified",
            &[&hashed],
        )
        .await?;
    let verified_ng: bool = verify_ng.get("verified");
    println!(
        "  間違ったパスワードで検証: {}",
        if verified_ng {
            "✓ 一致".red()
        } else {
            "✗ 不一致".green()
        }
    );

    // AES暗号化
    println!("\n  {}", "AES暗号化:".cyan());
    let encrypted = client
        .query_one(
            "SELECT encode(encrypt('sensitive data', 'secret_key_1234!', 'aes'), 'base64') AS encrypted",
            &[],
        )
        .await?;
    let enc_data: String = encrypted.get("encrypted");
    println!("    暗号化データ: {}", enc_data.dimmed());

    let decrypted = client
        .query_one(
            "SELECT convert_from(decrypt(decode($1, 'base64'), 'secret_key_1234!', 'aes'), 'utf8') AS decrypted",
            &[&enc_data],
        )
        .await?;
    let dec_data: String = decrypted.get("decrypted");
    println!("    復号化データ: {}", dec_data.green());

    // pg_trgm を有効化
    println!("\n{}", "【pg_trgm: 類似度検索】".yellow());

    client
        .execute("CREATE EXTENSION IF NOT EXISTS pg_trgm", &[])
        .await?;

    // 類似度計算
    let similarity = client
        .query_one(
            "SELECT similarity('PostgreSQL', 'Postgres') AS sim",
            &[],
        )
        .await?;
    let sim: f32 = similarity.get("sim");
    println!("  'PostgreSQL' と 'Postgres' の類似度: {:.2}", sim);

    // テスト用テーブルを作成してファジー検索
    client
        .execute(
            "CREATE TEMP TABLE IF NOT EXISTS products_search (
                id SERIAL PRIMARY KEY,
                name VARCHAR(100) NOT NULL
            )",
            &[],
        )
        .await?;

    client
        .execute(
            "INSERT INTO products_search (name) VALUES
            ('Cappuccino'),
            ('Caramel Macchiato'),
            ('Caffè Latte'),
            ('Café Mocha'),
            ('Cold Brew'),
            ('Espresso'),
            ('Americano')",
            &[],
        )
        .await?;

    // GINインデックスを作成（トライグラム用）
    client
        .execute(
            "CREATE INDEX idx_products_trgm ON products_search USING GIN(name gin_trgm_ops)",
            &[],
        )
        .await?;

    // ファジー検索
    println!("\n  {}", "ファジー検索（'Capuchino' でタイプミス検索）:".cyan());

    let fuzzy_results = client
        .query(
            "SELECT name, similarity(name, 'Capuchino') AS sim
            FROM products_search
            WHERE name % 'Capuchino'
            ORDER BY sim DESC",
            &[],
        )
        .await?;

    for row in fuzzy_results {
        let name: String = row.get("name");
        let sim: f32 = row.get("sim");
        println!("    {} (類似度: {:.2})", name.cyan(), sim);
    }

    // LIKE検索の高速化
    println!(
        "\n  {}",
        "LIKE検索（GINインデックス使用）:".cyan()
    );

    let explain = client
        .query(
            "EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF)
            SELECT * FROM products_search WHERE name ILIKE '%capp%'",
            &[],
        )
        .await?;

    for row in explain.iter().take(3) {
        let plan: String = row.get(0);
        if plan.contains("Bitmap") || plan.contains("Index") {
            println!("    {}", plan.green());
        } else {
            println!("    {}", plan.dimmed());
        }
    }

    // uuid-ossp を有効化
    println!("\n{}", "【uuid-ossp: UUID生成】".yellow());

    client
        .execute("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\"", &[])
        .await?;

    let uuid_v4 = client
        .query_one("SELECT uuid_generate_v4()::TEXT AS uuid", &[])
        .await?;
    let uuid: String = uuid_v4.get("uuid");
    println!("  UUID v4（ランダム）: {}", uuid.cyan());

    let uuid_v1 = client
        .query_one("SELECT uuid_generate_v1()::TEXT AS uuid", &[])
        .await?;
    let uuid1: String = uuid_v1.get("uuid");
    println!("  UUID v1（時刻ベース）: {}", uuid1.cyan());

    // citext を有効化
    println!("\n{}", "【citext: 大文字小文字を無視したテキスト】".yellow());

    client
        .execute("CREATE EXTENSION IF NOT EXISTS citext", &[])
        .await?;

    client
        .execute(
            "CREATE TEMP TABLE IF NOT EXISTS users_citext (
                id SERIAL PRIMARY KEY,
                email CITEXT NOT NULL UNIQUE
            )",
            &[],
        )
        .await?;

    client
        .execute(
            "INSERT INTO users_citext (email) VALUES ('User@Example.COM')",
            &[],
        )
        .await?;

    let citext_result = client
        .query_one(
            "SELECT email FROM users_citext WHERE email = 'user@example.com'",
            &[],
        )
        .await?;
    let email: String = citext_result.get("email");
    println!(
        "  'user@example.com' で検索 → {}",
        email.cyan()
    );
    println!("  → 大文字小文字を無視してマッチ！");

    // hstore（キーバリューストア）
    println!("\n{}", "【hstore: キーバリューストア】".yellow());

    client
        .execute("CREATE EXTENSION IF NOT EXISTS hstore", &[])
        .await?;

    let hstore_demo = client
        .query_one(
            "SELECT 'color => red, size => large'::hstore -> 'color' AS color,
                    'color => red, size => large'::hstore -> 'size' AS size",
            &[],
        )
        .await?;
    let color: Option<String> = hstore_demo.get("color");
    let size: Option<String> = hstore_demo.get("size");
    println!(
        "  hstore から取得: color={}, size={}",
        color.unwrap_or_default().cyan(),
        size.unwrap_or_default().cyan()
    );

    // 拡張機能の比較
    println!("\n{}", "【拡張機能の使い分け】".yellow());
    println!("  {}:", "pgcrypto".cyan());
    println!("    - パスワードハッシュ化（bcrypt推奨）");
    println!("    - データ暗号化（AES）");
    println!("    - ランダムバイト生成");

    println!("  {}:", "pg_trgm".cyan());
    println!("    - ファジー検索（タイプミス許容）");
    println!("    - LIKE/ILIKEの高速化");
    println!("    - 文字列の類似度計算");

    println!("  {}:", "uuid-ossp".cyan());
    println!("    - UUID v4（ランダム、一般的）");
    println!("    - UUID v1（時刻ベース、ソート可能）");
    println!("    - 分散システムの主キー");

    println!("  {}:", "citext".cyan());
    println!("    - メールアドレスの正規化");
    println!("    - ユーザー名の大文字小文字無視");

    println!("\n{}", "【高度な拡張機能（別途インストール必要）】".yellow());
    println!("  {}:", "pgvector".cyan());
    println!("    - ベクトル類似度検索");
    println!("    - AI/ML埋め込み検索");
    println!("    - IVFFlat/HNSWインデックス");

    println!("  {}:", "PostGIS".cyan());
    println!("    - 地理空間データ");
    println!("    - 距離計算、範囲検索");
    println!("    - GeoJSON対応");

    println!(
        "\n{}",
        "💡 ポイント: 拡張機能でPostgresを「スイスアーミーナイフ」に。".dimmed()
    );
    println!(
        "{}",
        "   専用ツールを追加する前に、拡張機能で解決できないか検討。".dimmed()
    );

    Ok(())
}
