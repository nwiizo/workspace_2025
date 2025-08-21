/// 記事のデモプログラム - 全機能を統合したサンプル
/// 
/// このプログラムは記事で紹介した全ての機能を実際に動作させます：
/// 1. ProcessGuardパターン
/// 2. セキュリティ（入力検証）
/// 3. プロセスプール
/// 4. シグナル処理
use linux_process_rs::{ProcessGuard, ProcessPool, ProcessError};
use linux_process_rs::process::validate_input;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Rustプロセス管理デモプログラム ===\n");
    
    // 1. ProcessGuardパターンのデモ
    demo_process_guard()?;
    
    // 2. セキュリティ機能のデモ
    demo_security()?;
    
    // 3. プロセスプールのデモ
    demo_process_pool()?;
    
    println!("\n=== 全デモ完了 ===");
    Ok(())
}

/// ProcessGuardパターンのデモ
fn demo_process_guard() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- 1. ProcessGuardパターン ---");
    println!("プロセスの自動クリーンアップをテストします\n");
    
    {
        println!("スコープ開始: ProcessGuardを作成");
        let guard = ProcessGuard::new_with_args("sleep", &["2"])?;
        
        println!("PID: {:?}", guard.pid());
        println!("実行中: {}", guard.is_running());
        
        println!("1秒待機...");
        thread::sleep(Duration::from_secs(1));
        
        println!("スコープを抜けます（自動クリーンアップが発生）");
    } // ここでDropが呼ばれ、自動的にプロセスが終了
    
    println!("ProcessGuardのスコープ外: プロセスは自動的に終了しました\n");
    
    Ok(())
}

/// セキュリティ機能のデモ
fn demo_security() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- 2. セキュリティ機能（入力検証） ---");
    println!("危険な入力をブロックします\n");
    
    // テストケース
    let test_inputs = vec![
        ("normal_file.txt", true, "正常なファイル名"),
        ("file.txt; rm -rf /", false, "コマンドインジェクション"),
        ("../../../etc/passwd", false, "パストラバーサル"),
        ("$(whoami)", false, "コマンド置換"),
        ("file && malicious", false, "コマンド連結"),
        ("~/secret", false, "ホームディレクトリ展開"),
    ];
    
    for (input, should_pass, description) in test_inputs {
        match validate_input(input) {
            Ok(_) => {
                if should_pass {
                    println!("✅ 許可: {} - {}", input, description);
                } else {
                    println!("❌ エラー: {} を許可してしまいました！", input);
                }
            }
            Err(e) => {
                if !should_pass {
                    println!("🛡️ ブロック: {} - {} (理由: {})", input, description, e);
                } else {
                    println!("❌ エラー: {} を誤ってブロックしました", input);
                }
            }
        }
    }
    
    println!();
    Ok(())
}

/// プロセスプールのデモ
fn demo_process_pool() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- 3. プロセスプール ---");
    println!("複数のワーカープロセスを管理します\n");
    
    // プロセスプールを作成（最大3ワーカー）
    let pool = ProcessPool::new("DemoPool", 3);
    
    // ワーカーを起動
    println!("ワーカーを起動します...");
    for i in 0..3 {
        match pool.spawn_worker_with_args("sleep", &[&format!("{}", i + 1)]) {
            Ok(pid) => println!("  ワーカー{} 起動成功: PID={}", i, pid),
            Err(e) => println!("  ワーカー{} 起動失敗: {}", i, e),
        }
    }
    
    // プールの状態を表示
    pool.status();
    
    // 最大数を超えて起動しようとする
    println!("最大数を超えてワーカーを起動しようとします...");
    match pool.spawn_worker("sleep") {
        Ok(_) => println!("  予期しない成功"),
        Err(ProcessError::InvalidInput(msg)) => {
            println!("  期待通りエラー: {}", msg);
        }
        Err(e) => println!("  別のエラー: {}", e),
    }
    
    println!("\nアクティブワーカー数: {}", pool.active_workers());
    
    // 少し待ってから一部のワーカーが終了
    println!("\n2秒待機（一部のワーカーが終了）...");
    thread::sleep(Duration::from_secs(2));
    
    println!("アクティブワーカー数: {}", pool.active_workers());
    
    // 残りのワーカーを終了
    println!("\n全ワーカーを終了します...");
    pool.terminate_all()?;
    
    println!("アクティブワーカー数: {}", pool.active_workers());
    
    Ok(())
}