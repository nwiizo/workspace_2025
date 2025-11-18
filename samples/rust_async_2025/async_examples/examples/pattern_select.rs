use std::time::Duration;

async fn fetch_from_server_a() -> Result<String, Box<dyn std::error::Error>> {
    tokio::time::sleep(Duration::from_secs(2)).await;
    Ok("サーバーA".to_string())
}

async fn fetch_from_server_b() -> Result<String, Box<dyn std::error::Error>> {
    tokio::time::sleep(Duration::from_secs(1)).await;
    Ok("サーバーB".to_string())
}

#[tokio::main]
async fn main() {
    use tokio::select;

    // 最初に完了したほうを使う
    select! {
        result_a = fetch_from_server_a() => {
            println!("サーバーAから: {:?}", result_a);
        }
        result_b = fetch_from_server_b() => {
            println!("サーバーBから: {:?}", result_b);
        }
    }
}
