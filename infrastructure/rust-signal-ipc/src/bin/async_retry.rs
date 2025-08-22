/// Async closuresを使ったリトライ機構の実装
///
/// Rust 1.85.0で安定化されたasync closuresの実践例

use std::future::Future;
use std::pin::Pin;
use tokio::time::{sleep, Duration};
use anyhow::{Result, Context};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// リトライ可能なエラーを判定するトレイト
trait RetryableError {
    fn is_retryable(&self) -> bool;
}

impl RetryableError for anyhow::Error {
    fn is_retryable(&self) -> bool {
        // エラーメッセージに基づいて判定（実際はエラー型で判定すべき）
        let msg = self.to_string();
        msg.contains("一時的") || msg.contains("timeout") || msg.contains("connection")
    }
}

/// 指数バックオフ付きリトライ実行
async fn retry_with_exponential_backoff<F, Fut, T>(
    mut f: F,
    max_retries: u32,
    initial_delay_ms: u64,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut last_error = None;
    let mut delay_ms = initial_delay_ms;
    
    for attempt in 1..=max_retries {
        println!("[Retry] 試行 {}/{}", attempt, max_retries);
        
        match f().await {
            Ok(result) => {
                println!("[Retry] 成功！（試行 {}回目）", attempt);
                return Ok(result);
            }
            Err(e) => {
                // エラーがリトライ可能か判定
                if !e.is_retryable() {
                    println!("[Retry] 致命的エラー: {}", e);
                    return Err(e);
                }
                
                println!("[Retry] 失敗（試行 {}）: {}", attempt, e);
                last_error = Some(e);
                
                if attempt < max_retries {
                    println!("[Retry] {}ms後にリトライ", delay_ms);
                    sleep(Duration::from_millis(delay_ms)).await;
                    
                    // 指数バックオフ（最大10秒）
                    delay_ms = (delay_ms * 2).min(10_000);
                }
            }
        }
    }
    
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("リトライ失敗")))
}

/// 並列実行ヘルパー
async fn parallel_execute<T>(
    tasks: Vec<(String, Pin<Box<dyn Future<Output = Result<T>> + Send>>)>,
) -> Vec<(String, Result<T>)>
where
    T: Send + 'static,
{
    let mut handles = Vec::new();
    
    for (name, task) in tasks {
        let handle = tokio::spawn(async move {
            let result = task.await;
            (name, result)
        });
        handles.push(handle);
    }
    
    let mut results = Vec::new();
    
    for handle in handles {
        match handle.await {
            Ok(result) => results.push(result),
            Err(e) => {
                results.push((
                    "unknown".to_string(),
                    Err(anyhow::anyhow!("タスクパニック: {}", e))
                ));
            }
        }
    }
    
    results
}

/// サービスのシミュレーション
struct MockService {
    failure_count: Arc<AtomicU32>,
    max_failures: u32,
}

impl MockService {
    fn new(max_failures: u32) -> Self {
        Self {
            failure_count: Arc::new(AtomicU32::new(0)),
            max_failures,
        }
    }
    
    async fn unreliable_operation(&self, id: u32) -> Result<String> {
        // 処理時間のシミュレーション
        sleep(Duration::from_millis(100)).await;
        
        let failures = self.failure_count.fetch_add(1, Ordering::SeqCst);
        
        if failures < self.max_failures {
            // 指定回数まで失敗
            Err(anyhow::anyhow!("一時的なエラー（失敗 {}/{}）", 
                                failures + 1, self.max_failures))
        } else {
            // その後は成功
            Ok(format!("操作{}成功（{}回の失敗後）", id, failures))
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Async Closures デモ ===\n");
    
    // 1. 基本的なリトライ
    println!("1. 基本的なリトライ:");
    let service = MockService::new(2);
    
    let result = retry_with_exponential_backoff(|| async {
        service.unreliable_operation(1).await
    }, 5, 100).await?;
    
    println!("最終結果: {}\n", result);
    
    // 2. 複数の非同期操作を並列実行
    println!("2. 並列実行:");
    
    let tasks: Vec<(String, Pin<Box<dyn Future<Output = Result<String>> + Send>>)> = vec![
        ("タスク1".to_string(), Box::pin(async {
            sleep(Duration::from_millis(500)).await;
            Ok::<String, anyhow::Error>("タスク1完了".to_string())
        })),
        ("タスク2".to_string(), Box::pin(async {
            sleep(Duration::from_millis(300)).await;
            Ok("タスク2完了".to_string())
        })),
        ("タスク3".to_string(), Box::pin(async {
            sleep(Duration::from_millis(100)).await;
            if rand::random::<bool>() {
                Ok("タスク3完了".to_string())
            } else {
                Err(anyhow::anyhow!("タスク3失敗"))
            }
        })),
    ];
    
    let start = std::time::Instant::now();
    let results = parallel_execute(tasks).await;
    let elapsed = start.elapsed();
    
    println!("実行時間: {:.2}秒", elapsed.as_secs_f64());
    for (name, result) in results {
        match result {
            Ok(msg) => println!("  {}: ✓ {}", name, msg),
            Err(e) => println!("  {}: ✗ {}", name, e),
        }
    }
    
    // 3. チェーン処理
    println!("\n3. チェーン処理:");
    
    let service2 = MockService::new(1);
    let service3 = MockService::new(0);
    
    let chained_result = retry_with_exponential_backoff(|| async {
        // 最初の操作
        let step1 = service2.unreliable_operation(100)
            .await?;
        
        println!("  ステップ1完了: {}", step1);
        
        // 2番目の操作
        let step2 = service3.unreliable_operation(200)
            .await?;
        
        println!("  ステップ2完了: {}", step2);
        
        Ok(format!("チェーン完了: {} -> {}", step1, step2))
    }, 3, 200).await?;
    
    println!("チェーン結果: {}", chained_result);
    
    // 4. タイムアウト付き実行
    println!("\n4. タイムアウト付き実行:");
    
    let timeout_result = tokio::time::timeout(
        Duration::from_secs(2),
        retry_with_exponential_backoff(|| async {
            // 長時間かかる処理
            sleep(Duration::from_millis(100)).await;
            Ok::<String, anyhow::Error>("処理完了".to_string())
        }, 3, 100)
    ).await;
    
    match timeout_result {
        Ok(Ok(result)) => println!("成功: {}", result),
        Ok(Err(e)) => println!("エラー: {}", e),
        Err(_) => println!("タイムアウト！"),
    }
    
    Ok(())
}