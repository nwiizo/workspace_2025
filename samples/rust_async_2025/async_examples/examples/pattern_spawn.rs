use std::time::Duration;
use tokio::task::JoinHandle;

async fn worker(id: i32, duration: u64) -> String {
    tokio::time::sleep(Duration::from_secs(duration)).await;
    format!("ワーカー {} 完了", id)
}

#[tokio::main]
async fn main() {
    let mut handles: Vec<JoinHandle<String>> = Vec::new();

    // 複数のワーカーをスポーン
    for i in 0..5 {
        let handle = tokio::spawn(worker(i, i as u64 + 1));
        handles.push(handle);
    }

    // すべての完了を待つ
    for handle in handles {
        match handle.await {
            Ok(result) => println!("{}", result),
            Err(e) => eprintln!("エラー: {}", e),
        }
    }
}
