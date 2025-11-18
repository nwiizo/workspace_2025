use std::time::Duration;

// Async closureを使った例
async fn process_items<F, Fut>(items: Vec<String>, processor: F) -> Vec<String>
where
    F: Fn(String) -> Fut,
    Fut: std::future::Future<Output = String>,
{
    let mut results = Vec::new();
    for item in items {
        results.push(processor(item).await);
    }
    results
}

// Async trait methodを使った例
trait DataProcessor {
    async fn process(&self, data: &str) -> String;
}

struct UppercaseProcessor;

impl DataProcessor for UppercaseProcessor {
    async fn process(&self, data: &str) -> String {
        tokio::time::sleep(Duration::from_millis(100)).await;
        data.to_uppercase()
    }
}

#[tokio::main]
async fn main() {
    // Async closureの使用
    let items = vec!["hello".to_string(), "world".to_string()];
    let results = process_items(items, |item| async move {
        format!("処理済み: {}", item)
    })
    .await;

    println!("結果: {:?}", results);

    // Async traitの使用
    let processor = UppercaseProcessor;
    let result = processor.process("rust 2024").await;
    println!("変換結果: {}", result);
}
