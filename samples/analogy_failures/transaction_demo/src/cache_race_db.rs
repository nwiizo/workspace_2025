// キャッシュ競合: 実際のDBを使った検証
// 読み取りと更新が同時に発生して、古いデータがキャッシュに残る

use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

type UserId = i32;

#[derive(Clone, Debug)]
struct User {
    id: UserId,
    email: String,
}

struct UserCache {
    cache: RwLock<HashMap<UserId, User>>,
}

impl UserCache {
    fn new() -> Self {
        UserCache {
            cache: RwLock::new(HashMap::new()),
        }
    }

    fn get(&self, id: UserId) -> Option<User> {
        self.cache.read().unwrap().get(&id).cloned()
    }

    fn set(&self, id: UserId, user: User) {
        self.cache.write().unwrap().insert(id, user);
    }

    fn invalidate(&self, id: UserId) {
        self.cache.write().unwrap().remove(&id);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== キャッシュ競合: 実際のDBを使った検証 ===\n");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://demo:demo@localhost:5433/analogy_demo")
        .await?;

    // テスト用ユーザーテーブルを作成
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            email VARCHAR(255) NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    // テストデータをリセット
    sqlx::query("DELETE FROM users").execute(&pool).await?;
    sqlx::query("INSERT INTO users (id, email) VALUES (1, 'old@example.com')")
        .execute(&pool)
        .await?;

    let cache = Arc::new(UserCache::new());

    println!("--- 初期状態 ---");
    let db_user = fetch_user_from_db(&pool, 1).await?;
    println!("DB: email = {:?}", db_user.as_ref().map(|u| &u.email));
    println!("キャッシュ: {:?}", cache.get(1).map(|u| u.email));

    println!("\n--- 競合シナリオを再現 ---\n");
    println!("時系列:");
    println!("  T1: リクエストAがキャッシュを確認 → ない");
    println!("  T2: リクエストAがDBから取得 (email=old@example.com)");
    println!("  T3: リクエストBがDBを更新 (email=new@example.com)");
    println!("  T4: リクエストBがキャッシュを無効化");
    println!("  T5: リクエストAがキャッシュに古いデータを保存 ← 問題！\n");

    let pool_a = pool.clone();
    let cache_a = cache.clone();
    let pool_b = pool.clone();
    let cache_b = cache.clone();

    // リクエストA: ユーザー情報を取得（途中で一時停止してリクエストBに割り込ませる）
    let request_a = tokio::spawn(async move {
        println!("[T1] リクエストA: キャッシュを確認");
        let cached = cache_a.get(1);
        if cached.is_some() {
            println!("  → キャッシュヒット");
            return;
        }
        println!("  → キャッシュミス");

        println!("[T2] リクエストA: DBから取得中...");
        let user = fetch_user_from_db(&pool_a, 1).await.unwrap().unwrap();
        println!("  → 取得完了: email = {:?}", user.email);

        // ここで一時停止（リクエストBが割り込む時間を作る）
        println!("  (処理に時間がかかっている...)");
        tokio::time::sleep(Duration::from_millis(200)).await;

        println!("[T5] リクエストA: キャッシュに保存");
        cache_a.set(1, user.clone());
        println!("  → 保存完了: email = {:?} ← 古いデータ！", user.email);
    });

    // 少し待ってからリクエストB（T2の後、T5の前に実行）
    tokio::time::sleep(Duration::from_millis(100)).await;

    let request_b = tokio::spawn(async move {
        println!("[T3] リクエストB: DBを更新");
        sqlx::query("UPDATE users SET email = 'new@example.com' WHERE id = 1")
            .execute(&pool_b)
            .await
            .unwrap();
        println!("  → 更新完了: email = new@example.com");

        println!("[T4] リクエストB: キャッシュを無効化");
        cache_b.invalidate(1);
        println!("  → 無効化完了");
    });

    // 両方のリクエストが完了するのを待つ
    let _ = tokio::join!(request_a, request_b);

    println!("\n--- 結果確認 ---");
    let db_user = fetch_user_from_db(&pool, 1).await?;
    let cached_user = cache.get(1);

    println!("DB: email = {:?}", db_user.as_ref().map(|u| &u.email));
    println!(
        "キャッシュ: email = {:?}",
        cached_user.as_ref().map(|u| &u.email)
    );

    if let (Some(db), Some(cached)) = (&db_user, &cached_user) {
        if db.email != cached.email {
            println!("\n⚠️  不整合発生！");
            println!("   DBは 'new@example.com' なのに");
            println!("   キャッシュは 'old@example.com' のまま");
            println!("\n   この状態でキャッシュを参照すると、");
            println!("   ユーザーには古いメールアドレスが表示される");
        }
    }

    println!("\n--- Phil Karlton曰く ---");
    println!("「コンピュータサイエンスで難しいことは2つしかない。");
    println!(" キャッシュの無効化と、名前付けだ」");

    // クリーンアップ
    sqlx::query("DROP TABLE IF EXISTS users")
        .execute(&pool)
        .await?;

    Ok(())
}

async fn fetch_user_from_db(pool: &PgPool, id: UserId) -> Result<Option<User>, sqlx::Error> {
    let row = sqlx::query("SELECT id, email FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(row.map(|r| User {
        id: r.get("id"),
        email: r.get("email"),
    }))
}
