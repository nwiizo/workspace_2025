use std::time::Duration;

// これがついに標準機能に！
trait AsyncService {
    async fn process(&self, data: String) -> Result<String, Box<dyn std::error::Error>>;
    async fn validate(&self, input: &str) -> bool;
}

struct MyService;

impl AsyncService for MyService {
    async fn process(&self, data: String) -> Result<String, Box<dyn std::error::Error>> {
        tokio::time::sleep(Duration::from_secs(1)).await;
        Ok(format!("処理完了: {}", data))
    }

    async fn validate(&self, input: &str) -> bool {
        !input.is_empty()
    }
}

// ジェネリックな非同期関数でも使える
async fn use_service<T: AsyncService>(service: &T) {
    match service.process("データ".to_string()).await {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("エラー: {}", e),
    }
}

#[tokio::main]
async fn main() {
    let service = MyService;
    use_service(&service).await;

    let is_valid = service.validate("test").await;
    println!("検証結果: {}", is_valid);
}
