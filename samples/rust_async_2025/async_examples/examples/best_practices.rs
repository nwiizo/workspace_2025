use std::time::Duration;
use tokio::task;

fn cpu_intensive_work(n: u64) -> u64 {
    // フィボナッチ数の計算など
    (0..n).sum()
}

#[tokio::main]
async fn main() {
    println!("ベストプラクティスのデモ\n");

    // 良い例: tokio::time::sleepを使用
    println!("非同期スリープ開始...");
    tokio::spawn(async {
        tokio::time::sleep(Duration::from_secs(1)).await;
        println!("非同期スリープ完了");
    });

    // CPU集約的な処理はspawn_blockingで
    println!("\nCPU集約的な処理開始...");
    let result = task::spawn_blocking(|| cpu_intensive_work(1_000_000))
        .await
        .unwrap();

    println!("結果: {}", result);

    // タスクの完了を待つ
    tokio::time::sleep(Duration::from_millis(1500)).await;
}
