// もうuseステートメントが不要！
// use std::future::Future; // ← 不要になった

async fn my_future() -> i32 {
    42
}

// FutureもIntoFutureもpreludeに含まれているので
// そのまま使える
fn process_future<F: std::future::Future<Output = i32>>(future: F) {
    println!("Futureを受け取りました");
    // 実際には await するか、ランタイムで実行する必要があります
    drop(future);
}

#[tokio::main]
async fn main() {
    let future = my_future();
    process_future(future);

    let result = my_future().await;
    println!("結果: {}", result);
}
