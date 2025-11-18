use tokio::sync::Mutex as TokioMutex;

async fn some_async_function() {
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
}

// ✅ 良い例：tokio::sync::Mutexを使用
async fn good_example() {
    let data = TokioMutex::new(0);
    let guard = data.lock().await;

    // これは問題ない
    some_async_function().await;

    drop(guard);
}

#[tokio::main]
async fn main() {
    println!("Send境界のデモ");
    good_example().await;
    println!("完了");
}
