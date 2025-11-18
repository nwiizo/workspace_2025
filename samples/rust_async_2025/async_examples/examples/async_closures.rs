use std::time::Duration;

// ついに可能になった！
async fn test_basic_async_closure() {
    let async_closure = async || {
        tokio::time::sleep(Duration::from_secs(1)).await;
        "完了".to_string()
    };

    let result = async_closure().await;
    println!("結果: {}", result);
}

// AsyncFnトレイトを使った高階関数
async fn process_with_async_closure<F, Fut>(f: F) -> String
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = String>,
{
    f().await
}

#[tokio::main]
async fn main() {
    test_basic_async_closure().await;

    let result = process_with_async_closure(async || {
        "非同期クロージャ".to_string()
    })
    .await;
    println!("結果: {}", result);
}
