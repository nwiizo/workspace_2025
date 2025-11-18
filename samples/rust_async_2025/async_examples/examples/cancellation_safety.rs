use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

// キャンセル安全でない例
async fn unsafe_increment(counter: Arc<Mutex<i32>>) {
    let mut guard = counter.lock().await;
    *guard += 1;
    // ここでキャンセルされると、ロックが保持されたまま
    tokio::time::sleep(Duration::from_secs(1)).await;
}

// キャンセル安全な例
async fn safe_increment(counter: Arc<Mutex<i32>>) {
    let mut guard = counter.lock().await;
    *guard += 1;
    drop(guard); // 明示的にロックを解放

    tokio::time::sleep(Duration::from_secs(1)).await;
}

#[tokio::main]
async fn main() {
    println!("キャンセル安全性のデモ");

    // 安全な例のテスト
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();

    tokio::spawn(async move {
        safe_increment(counter_clone).await;
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let value = counter.lock().await;
    println!("カウンター値: {}", *value);
}
