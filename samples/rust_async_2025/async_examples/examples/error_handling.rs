use std::time::Duration;
use tokio::time::timeout;

async fn risky_operation() -> Result<String, &'static str> {
    tokio::time::sleep(Duration::from_secs(2)).await;
    Ok("成功".to_string())
}

async fn slow_operation() -> Result<String, &'static str> {
    tokio::time::sleep(Duration::from_secs(10)).await;
    Ok("遅い操作完了".to_string())
}

#[tokio::main]
async fn main() {
    // タイムアウト付き実行
    let result = timeout(Duration::from_secs(3), slow_operation()).await;

    match result {
        Ok(Ok(value)) => println!("成功: {}", value),
        Ok(Err(e)) => println!("操作エラー: {}", e),
        Err(_) => println!("タイムアウト"),
    }

    // 複数の操作を並行実行し、エラーを適切に処理
    let results = tokio::join!(
        risky_operation(),
        risky_operation(),
        risky_operation(),
    );

    match results {
        (Ok(r1), Ok(r2), Ok(r3)) => {
            println!("すべて成功: {}, {}, {}", r1, r2, r3);
        }
        _ => {
            println!("一部が失敗しました");
        }
    }
}
