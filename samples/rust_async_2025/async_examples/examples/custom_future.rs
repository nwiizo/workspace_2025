use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

struct CounterFuture {
    count: u32,
    max: u32,
}

impl CounterFuture {
    fn new(max: u32) -> Self {
        Self { count: 0, max }
    }
}

impl Future for CounterFuture {
    type Output = u32;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.count += 1;
        println!("ポーリング #{}: カウント = {}", self.count, self.count);

        // 実際の待機をシミュレート
        std::thread::sleep(Duration::from_millis(100));

        if self.count < self.max {
            // まだ完了していない
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            // 完了
            Poll::Ready(self.count)
        }
    }
}

#[tokio::main]
async fn main() {
    let future1 = CounterFuture::new(3);
    let future2 = CounterFuture::new(3);

    let (result1, result2) = tokio::join!(future1, future2);

    println!("\n結果1: {}", result1);
    println!("結果2: {}", result2);
}
