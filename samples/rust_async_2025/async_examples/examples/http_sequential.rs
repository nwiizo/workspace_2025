use std::time::Instant;
use reqwest::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let url = "https://3-shake.com/";
    let start_time = Instant::now();

    // 順番に4回リクエスト
    for i in 1..=4 {
        let response = reqwest::get(url).await?;
        println!("リクエスト {} 完了: ステータス {}", i, response.status());
    }

    let elapsed = start_time.elapsed();
    println!("合計時間: {} ms", elapsed.as_millis());

    Ok(())
}
