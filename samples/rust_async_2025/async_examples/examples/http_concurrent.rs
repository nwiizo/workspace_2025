use std::time::Instant;
use reqwest::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let url = "https://3-shake.com/";
    let start_time = Instant::now();

    // 4つのリクエストを同時に実行
    let (r1, r2, r3, r4) = tokio::join!(
        reqwest::get(url),
        reqwest::get(url),
        reqwest::get(url),
        reqwest::get(url),
    );

    // 結果を確認
    println!("リクエスト1: {:?}", r1.map(|r| r.status()));
    println!("リクエスト2: {:?}", r2.map(|r| r.status()));
    println!("リクエスト3: {:?}", r3.map(|r| r.status()));
    println!("リクエスト4: {:?}", r4.map(|r| r.status()));

    let elapsed = start_time.elapsed();
    println!("合計時間: {} ms", elapsed.as_millis());

    Ok(())
}
