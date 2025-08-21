/// シンプルなシグナルハンドラーの実装
///
/// SIGINT, SIGTERM, SIGHUPを処理する基本的なシグナルハンドラー

use anyhow::{Context, Result};
use signal_hook::{consts::{SIGINT, SIGTERM, SIGHUP}, iterator::Signals};
use std::{thread, time::Duration};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{info, warn};

fn main() -> Result<()> {
    // ログの初期化
    tracing_subscriber::fmt::init();
    
    info!("シグナルハンドラーを起動しました");
    info!("PID: {}", std::process::id());
    info!("Ctrl+C (SIGINT) で終了");
    info!("kill -HUP {} で設定再読み込み", std::process::id());
    info!("kill -TERM {} で終了", std::process::id());
    
    // シャットダウンフラグ
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    // 複数のシグナルを監視
    let mut signals = Signals::new(&[SIGINT, SIGTERM, SIGHUP])
        .context("シグナルハンドラーの設定に失敗")?;
    
    // シグナル処理用のスレッドを生成
    thread::spawn(move || {
        for sig in signals.forever() {
            match sig {
                SIGINT => {
                    warn!("[SIGINT] Ctrl+Cを受信");
                    simple_shutdown();
                    r.store(false, Ordering::SeqCst);
                    break;
                }
                SIGTERM => {
                    warn!("[SIGTERM] 終了シグナルを受信");
                    simple_shutdown();
                    r.store(false, Ordering::SeqCst);
                    break;
                }
                SIGHUP => {
                    info!("[SIGHUP] 設定再読み込みシグナルを受信");
                    reload_configuration();
                }
                _ => {}
            }
        }
    });
    
    // メインスレッドの処理
    let mut counter = 0;
    while running.load(Ordering::SeqCst) {
        info!("処理実行中... カウント: {}", counter);
        counter += 1;
        thread::sleep(Duration::from_secs(2));
    }
    
    info!("プログラムを終了します");
    Ok(())
}

/// シンプルなシャットダウン処理
fn simple_shutdown() {
    info!("シャットダウンを開始");
    info!("リソースをクリーンアップ中...");
    thread::sleep(Duration::from_millis(100));
    info!("シャットダウン完了");
}

/// 設定の再読み込み（シミュレーション）
fn reload_configuration() {
    info!("設定を再読み込み中...");
    thread::sleep(Duration::from_millis(50));
    info!("設定の再読み込み完了");
}