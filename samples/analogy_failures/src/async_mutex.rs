// 非同期処理の類推失敗: Mutexによる直列化
// 「料理を並行して作る」つもりが、実際には直列実行になる例

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

struct Stove {
    name: String,
}

impl Stove {
    fn new() -> Self {
        Stove {
            name: "メインコンロ".to_string(),
        }
    }
}

async fn cook_on_stove(stove: &mut Stove, dish: &str, duration_secs: u64) {
    println!("[{}] {} の調理を開始", stove.name, dish);
    tokio::time::sleep(Duration::from_secs(duration_secs)).await;
    println!("[{}] {} の調理が完了", stove.name, dish);
}

struct Kitchen {
    stove: Arc<Mutex<Stove>>,
}

#[tokio::main]
async fn main() {
    println!("=== 非同期処理の類推失敗: Mutexによる直列化 ===\n");

    let kitchen = Kitchen {
        stove: Arc::new(Mutex::new(Stove::new())),
    };

    println!("「2つの料理を並行して作る」つもりで書いたコード:\n");
    println!("期待: 各料理2秒 → 並行なので合計約2秒");
    println!("実際: 各料理2秒 → 直列なので合計約4秒\n");

    let start = Instant::now();

    let stove = kitchen.stove.clone();

    // 2つの料理を「並行して」作ろうとする
    let dish1 = tokio::spawn({
        let stove = stove.clone();
        async move {
            let mut s = stove.lock().await; // コンロを確保
            cook_on_stove(&mut s, "パスタ", 2).await;
        }
    });

    let dish2 = tokio::spawn({
        let stove = stove.clone();
        async move {
            let mut s = stove.lock().await; // コンロを確保しようとする...
            cook_on_stove(&mut s, "ソース", 2).await; // dish1が終わるまで待つ
        }
    });

    let _ = tokio::join!(dish1, dish2);

    let elapsed = start.elapsed();
    println!("\n合計調理時間: {:.2}秒", elapsed.as_secs_f64());
    println!("\n→ 並行処理のつもりが、Mutexで直列化されていた！");
    println!("  現実の料理: コンロの左右で同時に調理できる");
    println!("  コンピュータ: Mutexは一度に1つのタスクしかアクセスできない");
}
