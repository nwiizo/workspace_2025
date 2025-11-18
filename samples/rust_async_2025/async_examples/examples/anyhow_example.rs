use anyhow::Result;

async fn step1() -> Result<String> {
    Ok("ステップ1完了".to_string())
}

async fn step2() -> Result<String> {
    Ok("ステップ2完了".to_string())
}

async fn process() -> Result<()> {
    let result1 = step1().await?;
    let result2 = step2().await?;

    println!("{}", result1);
    println!("{}", result2);

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = process().await {
        eprintln!("エラー: {}", e);
    }
}
