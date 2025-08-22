/// シグナル処理検証システムの実行可能ファイル
///
/// すべての検証テストを実行し、結果をレポート

use rust_signal_ipc::examples::test_runner::{SignalTestRunner, run_benchmark};
use std::env;
use anyhow::Result;

fn main() -> Result<()> {
    // コマンドライン引数の処理
    let args: Vec<String> = env::args().collect();
    let verbose = args.contains(&"--verbose".to_string()) || args.contains(&"-v".to_string());
    let benchmark_only = args.contains(&"--benchmark".to_string());
    let json_output = args.contains(&"--json".to_string());
    let help = args.contains(&"--help".to_string()) || args.contains(&"-h".to_string());
    
    if help {
        print_help();
        return Ok(());
    }
    
    // ログ初期化
    if !json_output {
        tracing_subscriber::fmt::init();
    }
    
    if benchmark_only {
        // ベンチマークのみ実行
        run_benchmark();
    } else {
        // テストランナーを作成して実行
        let mut runner = SignalTestRunner::new(verbose);
        
        if !json_output {
            println!("╔════════════════════════════════════════════════════════╗");
            println!("║     Rust シグナル処理検証システム v0.1.0              ║");
            println!("║     Signal Processing Validation System               ║");
            println!("╚════════════════════════════════════════════════════════╝");
        }
        
        // すべてのテストを実行
        runner.run_all_tests();
        
        // 結果を出力
        if json_output {
            println!("{}", runner.results_as_json());
        } else {
            println!("{}", runner.generate_report());
            
            // ベンチマークも実行
            if verbose {
                run_benchmark();
            }
        }
    }
    
    Ok(())
}

fn print_help() {
    println!("Rust Signal Processing Validator");
    println!();
    println!("使用方法:");
    println!("  signal-validator [オプション]");
    println!();
    println!("オプション:");
    println!("  -h, --help       このヘルプメッセージを表示");
    println!("  -v, --verbose    詳細な出力を表示");
    println!("  --benchmark      ベンチマークのみ実行");
    println!("  --json           結果をJSON形式で出力");
    println!();
    println!("例:");
    println!("  signal-validator                # 通常実行");
    println!("  signal-validator -v             # 詳細モード");
    println!("  signal-validator --benchmark    # ベンチマークのみ");
    println!("  signal-validator --json         # JSON出力");
}