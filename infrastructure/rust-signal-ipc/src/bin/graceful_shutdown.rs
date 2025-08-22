/// CancellationTokenパターンによるグレイスフルシャットダウン
///
/// Rust 2024のベストプラクティスを適用した実装

use anyhow::{Context, Result};
use tokio::signal;
use tokio::time::{sleep, timeout, Duration};
use tokio_util::sync::CancellationToken;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // ログの初期化
    tracing_subscriber::fmt::init();
    
    info!("グレイスフル・シャットダウンのデモを開始");
    info!("Ctrl+C で安全にシャットダウンします");
    
    // CancellationTokenを作成
    let token = CancellationToken::new();
    let counter = Arc::new(AtomicU64::new(0));
    
    // ワーカータスクを起動
    let worker1 = spawn_worker(token.child_token(), counter.clone(), "データ処理", 1).await;
    let worker2 = spawn_worker(token.child_token(), counter.clone(), "API処理", 2).await;
    let worker3 = spawn_worker(token.child_token(), counter.clone(), "バックグラウンドジョブ", 3).await;
    
    // メトリクス表示タスク
    let metrics_task = spawn_metrics_task(token.child_token(), counter.clone()).await;
    
    // Ctrl+Cシグナルを待つ
    signal::ctrl_c().await
        .context("Ctrl+Cハンドラの設定に失敗")?;
    
    warn!("\nシャットダウンシグナルを受信");
    info!("グレイスフル・シャットダウンを開始");
    
    // キャンセルトークンを発火
    token.cancel();
    
    // すべてのワーカーの完了を待つ（タイムアウト付き）
    let shutdown_timeout = Duration::from_secs(10);
    info!("ワーカーの終了を待機中（最大 {} 秒）", shutdown_timeout.as_secs());
    
    match timeout(
        shutdown_timeout,
        futures::future::join_all(vec![worker1, worker2, worker3, metrics_task])
    ).await {
        Ok(_) => {
            info!("すべてのワーカーが正常に終了");
        }
        Err(_) => {
            error!("タイムアウト: 一部のワーカーが時間内に終了しませんでした");
        }
    }
    
    let total_processed = counter.load(Ordering::Relaxed);
    info!("グレイスフル・シャットダウン完了");
    info!("総処理件数: {}", total_processed);
    
    Ok(())
}

/// ワーカータスクを生成
async fn spawn_worker(
    token: CancellationToken,
    counter: Arc<AtomicU64>,
    name: &str,
    id: u32,
) -> tokio::task::JoinHandle<()> {
    let name = name.to_string();
    
    tokio::spawn(async move {
        info!("ワーカー{}: {} タスクを開始", id, name);
        
        loop {
            tokio::select! {
                _ = token.cancelled() => {
                    info!("ワーカー{}: キャンセルシグナルを受信", id);
                    cleanup(&name, id).await;
                    break;
                }
                _ = process_task(&name, id, &counter) => {
                    // タスク処理完了
                }
            }
        }
        
        info!("ワーカー{}: 終了", id);
    })
}

/// タスク処理（シミュレーション）
async fn process_task(name: &str, id: u32, counter: &Arc<AtomicU64>) {
    // タスクの種類によって処理時間を変える
    let duration = match name {
        "データ処理" => Duration::from_millis(500),
        "API処理" => Duration::from_millis(800),
        "バックグラウンドジョブ" => Duration::from_secs(2),
        _ => Duration::from_secs(1),
    };
    
    sleep(duration).await;
    
    let count = if name == "バックグラウンドジョブ" { 5 } else { 1 };
    counter.fetch_add(count, Ordering::Relaxed);
    
    info!("ワーカー{}: {} 完了（{}件処理）", id, name, count);
}

/// クリーンアップ処理
async fn cleanup(name: &str, id: u32) {
    info!("ワーカー{}: {} のクリーンアップ開始", id, name);
    
    // クリーンアップの種類によって処理時間を変える
    let duration = match name {
        "データ処理" => {
            info!("ワーカー{}: バッファをフラッシュ中", id);
            Duration::from_millis(500)
        }
        "API処理" => {
            info!("ワーカー{}: 進行中のリクエストを完了中", id);
            Duration::from_secs(1)
        }
        "バックグラウンドジョブ" => {
            info!("ワーカー{}: 未処理ジョブをキューに保存中", id);
            Duration::from_millis(300)
        }
        _ => Duration::from_millis(100),
    };
    
    sleep(duration).await;
    info!("ワーカー{}: クリーンアップ完了", id);
}

/// メトリクス表示タスク
async fn spawn_metrics_task(
    token: CancellationToken,
    counter: Arc<AtomicU64>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        info!("メトリクス: 統計情報の表示を開始");
        let mut last_count = 0u64;
        
        loop {
            tokio::select! {
                _ = token.cancelled() => {
                    info!("メトリクス: シャットダウンシグナルを受信");
                    break;
                }
                _ = sleep(Duration::from_secs(3)) => {
                    let current_count = counter.load(Ordering::Relaxed);
                    let rate = current_count - last_count;
                    info!("統計: 総処理数={}, 処理速度={}/3秒", current_count, rate);
                    last_count = current_count;
                }
            }
        }
        
        info!("メトリクス: 最終統計を保存中");
        sleep(Duration::from_millis(100)).await;
        info!("メトリクス: 完了");
    })
}