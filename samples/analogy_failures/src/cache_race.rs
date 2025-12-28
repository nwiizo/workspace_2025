// キャッシュの類推失敗: 競合状態
// 「手元に辞書を置く」類推では、キャッシュ無効化の複雑さが見えない

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

type UserId = u64;

#[derive(Clone, Debug)]
struct User {
    id: UserId,
    email: String,
}

// シミュレートされたデータベース
struct Database {
    users: RwLock<HashMap<UserId, User>>,
}

impl Database {
    fn new() -> Self {
        let mut users = HashMap::new();
        users.insert(
            1,
            User {
                id: 1,
                email: "old@example.com".to_string(),
            },
        );
        Database {
            users: RwLock::new(users),
        }
    }

    async fn fetch_user(&self, id: UserId) -> Option<User> {
        // DBアクセスをシミュレート（少し時間がかかる）
        tokio::time::sleep(Duration::from_millis(50)).await;
        self.users.read().unwrap().get(&id).cloned()
    }

    async fn update_email(&self, id: UserId, new_email: &str) {
        // DBアクセスをシミュレート
        tokio::time::sleep(Duration::from_millis(10)).await;
        if let Some(user) = self.users.write().unwrap().get_mut(&id) {
            user.email = new_email.to_string();
        }
    }
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

    async fn get_user(&self, id: UserId, db: &Database) -> Option<User> {
        // まずキャッシュを確認
        if let Some(user) = self.cache.read().unwrap().get(&id) {
            println!("  [キャッシュ] ヒット: {:?}", user.email);
            return Some(user.clone());
        }

        println!("  [キャッシュ] ミス、DBから取得中...");

        // なければDBから取得
        let user = db.fetch_user(id).await?;
        println!("  [DB] 取得完了: {:?}", user.email);

        // キャッシュに保存
        self.cache.write().unwrap().insert(id, user.clone());
        println!("  [キャッシュ] 保存: {:?}", user.email);

        Some(user)
    }

    fn invalidate(&self, id: UserId) {
        self.cache.write().unwrap().remove(&id);
        println!("  [キャッシュ] 無効化: user_id={}", id);
    }
}

async fn update_user_email(cache: &UserCache, db: &Database, id: UserId, new_email: &str) {
    println!("  [DB] メール更新: {} -> {}", id, new_email);
    db.update_email(id, new_email).await;
    cache.invalidate(id);
}

#[tokio::main]
async fn main() {
    println!("=== キャッシュの類推失敗: 競合状態 ===\n");

    let db = Arc::new(Database::new());
    let cache = Arc::new(UserCache::new());

    println!("シナリオ: 読み取りと更新が同時に発生\n");
    println!("時系列:");
    println!("  T1: リクエストAがget_user()を呼ぶ");
    println!("  T2: リクエストAがDBから古いデータを取得");
    println!("  T3: リクエストBがupdate_user_email()を呼ぶ");
    println!("  T4: リクエストBがキャッシュを無効化");
    println!("  T5: リクエストAが古いデータをキャッシュに保存 ← 問題！\n");

    // リクエストA: ユーザー情報を取得（途中で一時停止）
    let db_a = db.clone();
    let cache_a = cache.clone();
    let request_a = tokio::spawn(async move {
        println!("[T1] リクエストA: get_user()開始");

        // キャッシュを確認 → ない
        {
            let cached = cache_a.cache.read().unwrap().get(&1).cloned();
            if cached.is_some() {
                println!("  [キャッシュ] ヒット");
                return;
            }
            println!("  [キャッシュ] ミス");
        }

        // DBから取得（時間がかかる）
        println!("[T2] リクエストA: DBから取得中...");
        let user = db_a.fetch_user(1).await.unwrap();
        println!("  [DB] 取得完了: email={:?}", user.email);

        // ここで少し待つ（リクエストBが割り込む時間）
        println!("  (リクエストAは少し処理に時間がかかっている...)");
        tokio::time::sleep(Duration::from_millis(100)).await;

        // キャッシュに保存（この時点で古いデータ）
        println!("[T5] リクエストA: キャッシュに保存");
        cache_a.cache.write().unwrap().insert(1, user.clone());
        println!("  [キャッシュ] 保存: email={:?} ← 古いデータ！", user.email);
    });

    // 少し待ってからリクエストB
    tokio::time::sleep(Duration::from_millis(80)).await;

    // リクエストB: メールアドレスを更新
    let db_b = db.clone();
    let cache_b = cache.clone();
    let request_b = tokio::spawn(async move {
        println!("[T3] リクエストB: update_user_email()開始");
        println!("  [DB] メール更新: old@example.com -> new@example.com");
        db_b.update_email(1, "new@example.com").await;

        println!("[T4] リクエストB: キャッシュ無効化");
        cache_b.invalidate(1);
        println!("  リクエストB完了");
    });

    // 両方のリクエストが完了するのを待つ
    let _ = tokio::join!(request_a, request_b);

    println!("\n--- 結果確認 ---\n");

    // 現在のキャッシュの状態を確認
    let cached_user = cache.cache.read().unwrap().get(&1).cloned();
    let db_user = db.fetch_user(1).await;

    println!("DBの値:        email={:?}", db_user.map(|u| u.email));
    println!(
        "キャッシュの値: email={:?}",
        cached_user.map(|u| u.email)
    );

    println!("\n⚠️  キャッシュに古いデータが残っている！");
    println!("\n現実世界の辞書では:");
    println!("  辞書の内容は変わらない。置いておけば常に正しい。");
    println!("\nコンピュータのキャッシュでは:");
    println!("  元のデータが変わる。複数プロセスが同時にアクセスする。");
    println!("  処理の順序が予測できない。競合状態が発生する。");
    println!("\nPhil Karlton曰く:");
    println!("  「コンピュータサイエンスで難しいことは2つしかない。");
    println!("   キャッシュの無効化と、名前付けだ」");
}
