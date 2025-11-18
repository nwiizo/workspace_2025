use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct Counter {
    value: Arc<RwLock<i32>>,
}

impl Counter {
    fn new() -> Self {
        Self {
            value: Arc::new(RwLock::new(0)),
        }
    }

    async fn increment(&self) {
        let mut value = self.value.write().await;
        *value += 1;
    }

    async fn get(&self) -> i32 {
        let value = self.value.read().await;
        *value
    }
}

#[tokio::main]
async fn main() {
    let counter = Counter::new();
    let mut handles = vec![];

    // 10個のタスクで並行してインクリメント
    for _ in 0..10 {
        let counter_clone = counter.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..100 {
                counter_clone.increment().await;
            }
        });
        handles.push(handle);
    }

    // すべて完了を待つ
    for handle in handles {
        handle.await.unwrap();
    }

    println!("最終カウント: {}", counter.get().await);
}
