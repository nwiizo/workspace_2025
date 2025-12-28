// 非同期処理の類推失敗: デッドロック
// 現実の料理では「ちょっと使わせて」で解決するが、スレッドは声をかけない

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

struct Stove;
struct Oven;

struct Kitchen {
    stove: Arc<Mutex<Stove>>,
    oven: Arc<Mutex<Oven>>,
}

#[tokio::main]
async fn main() {
    println!("=== 非同期処理の類推失敗: デッドロック ===\n");

    let kitchen = Arc::new(Kitchen {
        stove: Arc::new(Mutex::new(Stove)),
        oven: Arc::new(Mutex::new(Oven)),
    });

    println!("タスク1: コンロ → オーブンの順で確保");
    println!("タスク2: オーブン → コンロの順で確保");
    println!("\n→ お互いが相手のリソースを待ち続けてデッドロック！\n");
    println!("(3秒後にタイムアウトします)\n");

    let kitchen1 = kitchen.clone();
    let kitchen2 = kitchen.clone();

    // タスク1: まずコンロを確保、次にオーブンを確保
    let task1 = tokio::spawn(async move {
        println!("[タスク1] コンロを確保しようとしています...");
        let _stove = kitchen1.stove.lock().await;
        println!("[タスク1] コンロを確保しました！");

        // 少し待つ（タスク2がオーブンを確保する時間を与える）
        tokio::time::sleep(Duration::from_millis(100)).await;

        println!("[タスク1] オーブンを確保しようとしています...");
        let _oven = kitchen1.oven.lock().await; // ここでブロック
        println!("[タスク1] オーブンを確保しました！");
    });

    // タスク2: まずオーブンを確保、次にコンロを確保
    let task2 = tokio::spawn(async move {
        println!("[タスク2] オーブンを確保しようとしています...");
        let _oven = kitchen2.oven.lock().await;
        println!("[タスク2] オーブンを確保しました！");

        // 少し待つ（タスク1がコンロを確保する時間を与える）
        tokio::time::sleep(Duration::from_millis(100)).await;

        println!("[タスク2] コンロを確保しようとしています...");
        let _stove = kitchen2.stove.lock().await; // ここでブロック
        println!("[タスク2] コンロを確保しました！");
    });

    // タイムアウト付きで待機
    let result = tokio::time::timeout(Duration::from_secs(3), async {
        let _ = tokio::join!(task1, task2);
    })
    .await;

    match result {
        Ok(_) => println!("\n両方のタスクが完了しました"),
        Err(_) => {
            println!("\n⚠️  タイムアウト！デッドロックが発生しました。");
            println!("\n現実の料理では:");
            println!("  「ちょっとコンロ使わせて」と声をかければ解決");
            println!("\nコンピュータでは:");
            println!("  スレッドは声をかけない。お互いが永遠に待ち続ける。");
            println!("\n解決策:");
            println!("  - ロックの取得順序を統一する（常にコンロ→オーブンの順）");
            println!("  - try_lock()でタイムアウト付きロックを使う");
            println!("  - 必要なリソースを一度にすべて取得する");
        }
    }
}
