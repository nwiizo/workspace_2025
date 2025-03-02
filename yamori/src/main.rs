// src/main.rs
mod app;
mod test;
mod ui;
mod cli;

use anyhow::{Context, Result};
use app::{App, PopupType};
use clap::Parser;
use chrono::TimeZone;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::{env, io, path::PathBuf, time::Duration};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the test configuration file (YAML or TOML)
    #[arg(short = 'y', long = "yamori-config", default_value = "tests/configs/tests.toml")]
    config: std::path::PathBuf,

    /// Run in CLI mode (no TUI)
    #[arg(short = 'c', long = "cli", default_value = "false")]
    cli_mode: bool,
}

fn main() -> Result<()> {
    // 環境変数 YAMORI_CONFIG から設定ファイルのパスを取得
    let config_from_env = env::var("YAMORI_CONFIG").ok();
    
    // コマンドライン引数を解析
    let mut args = Args::parse();
    
    // 環境変数から設定ファイルのパスが指定されていれば、それを優先
    if let Some(config_path) = config_from_env {
        args.config = PathBuf::from(config_path);
    }
    
    // Check if CLI mode is enabled
    if args.cli_mode {
        return cli::run_cli(args.config);
    }
    
    // コマンド出力を抑制
    // println!("Using config file: {}", args.config.display());
    
    // Load and parse the configuration
    let config = test::load_config(&args.config)
        .with_context(|| format!("failed to load config from `{}`", args.config.display()))?;
    
    // Run all tests
    let test_results = test::run_tests(&config)?;
    
    // Display results in TUI
    start_ui(test_results, config, args.config)?;
    
    Ok(())
}

fn start_ui(test_results: Vec<test::TestResult>, config: test::TestConfig, _config_path: PathBuf) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create app state
    let mut app = App::new(test_results, config);
    
    // ターミナルを完全に再初期化する関数
    let reset_terminal_completely = || -> Result<()> {
        // 一度ターミナルを元に戻す
        disable_raw_mode()?;
        execute!(
            io::stdout(),
            LeaveAlternateScreen,
        )?;
        
        // 少し待機
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // ターミナルを再設定
        enable_raw_mode()?;
        execute!(
            io::stdout(),
            EnterAlternateScreen,
        )?;
        
        Ok(())
    };
    
    // ターミナルを再初期化する関数
    let reset_terminal = |term: &mut Terminal<CrosstermBackend<io::Stdout>>| -> Result<()> {
        // 画面をクリア
        term.clear()?;
        
        // バッファをフラッシュ
        term.flush()?;
        
        // カーソルを左上に移動
        execute!(
            term.backend_mut(),
            crossterm::cursor::MoveTo(0, 0)
        )?;
        
        // 画面全体を再描画するためのフラグを設定
        term.draw(|_| {})?;
        
        Ok(())
    };
    
    // Start the main loop
    loop {
        // 画面を描画
        terminal.draw(|frame| {
            ui::render_ui::<CrosstermBackend<io::Stdout>>(frame, &app);
        })?;
        
        // 結果ポップアップの更新（時間経過で消える）
        if app.update_result_popup() {
            // ポップアップの状態が変わったら再描画
            terminal.draw(|frame| {
                ui::render_ui::<CrosstermBackend<io::Stdout>>(frame, &app);
            })?;
        }
        
        // Handle input with timeout
        if crossterm::event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        if !app.show_popup {
                            break;
                        }
                    },
                    KeyCode::Char('?') => {
                        if !app.show_popup {
                            app.toggle_help();
                        }
                    },
                    KeyCode::Down | KeyCode::Char('j') => {
                        if !app.show_help && !app.show_popup {
                            if app.tab_index == 4 {
                                // 履歴タブでは履歴を移動
                                app.next_history();
                            } else {
                                app.next();
                            }
                        }
                    },
                    KeyCode::Up | KeyCode::Char('k') => {
                        if !app.show_help && !app.show_popup {
                            if app.tab_index == 4 {
                                // 履歴タブでは履歴を移動
                                app.previous_history();
                            } else {
                                app.previous();
                            }
                        }
                    },
                    KeyCode::Right | KeyCode::Char('l') => {
                        if !app.show_help && !app.show_popup {
                            app.next_tab();
                        }
                    },
                    KeyCode::Left | KeyCode::Char('h') => {
                        if !app.show_help && !app.show_popup {
                            app.previous_tab();
                        }
                    },
                    KeyCode::Char('r') => {
                        if !app.show_help && !app.show_popup {
                            // ポップアップを表示
                            app.toggle_popup(PopupType::RunTests);
                        }
                    },
                    KeyCode::Char('b') => {
                        if !app.show_help && !app.show_popup {
                            // ポップアップを表示
                            app.toggle_popup(PopupType::BuildToggle);
                        }
                    },
                    KeyCode::Char('R') => {
                        if !app.show_help && !app.show_popup {
                            // ポップアップを表示
                            app.toggle_popup(PopupType::RunRelease);
                        }
                    },
                    // 履歴表示モードの切り替え
                    KeyCode::Char('H') => {
                        if !app.show_help && !app.show_popup {
                            app.toggle_history_view();
                        }
                    },
                    KeyCode::Enter => {
                        // ポップアップでの確認処理
                        if app.show_popup {
                            match app.popup_type {
                                PopupType::RunTests => {
                                    // テストを再実行
                                    app.close_popup();
                                    match test::run_tests(&app.config) {
                                        Ok(results) => {
                                            // 現在の結果を履歴に追加
                                            app.add_to_history();
                                            
                                            // 結果ポップアップを表示
                                            let (passed, total, pass_rate) = app.get_stats();
                                            app.show_result_popup(format!(
                                                "Tests completed!\n\nPassed: {}/{} ({:.1}%)",
                                                passed, total, pass_rate
                                            ));
                                            
                                            app.test_results = results;
                                            // UI の状態をリセット
                                            app.reset_ui_state();
                                            
                                            // ターミナルを完全に再初期化
                                            reset_terminal_completely()?;
                                            
                                            // バックエンドを再作成
                                            let backend = CrosstermBackend::new(io::stdout());
                                            terminal = Terminal::new(backend)?;
                                        },
                                        Err(e) => {
                                            // エラーが発生した場合は、ステータスバーに表示するなどの処理を追加できます
                                            // コマンド出力を抑制
                                            // eprintln!("Error running tests: {}", e);
                                            
                                            // エラーポップアップを表示
                                            app.show_result_popup(format!("Error running tests:\n{}", e));
                                        }
                                    }
                                },
                                PopupType::RunRelease => {
                                    // リリースモードを有効にしてテストを再実行
                                    app.close_popup();
                                    if let Some(build) = &mut app.config.build {
                                        build.release = true;
                                    }
                                    
                                    match test::run_tests(&app.config) {
                                        Ok(results) => {
                                            // 現在の結果を履歴に追加
                                            app.add_to_history();
                                            
                                            // 結果ポップアップを表示
                                            let (passed, total, pass_rate) = app.get_stats();
                                            app.show_result_popup(format!(
                                                "Release tests completed!\n\nPassed: {}/{} ({:.1}%)",
                                                passed, total, pass_rate
                                            ));
                                            
                                            app.test_results = results;
                                            // UI の状態をリセット
                                            app.reset_ui_state();
                                            
                                            // ターミナルを完全に再初期化
                                            reset_terminal_completely()?;
                                            
                                            // バックエンドを再作成
                                            let backend = CrosstermBackend::new(io::stdout());
                                            terminal = Terminal::new(backend)?;
                                        },
                                        Err(e) => {
                                            // コマンド出力を抑制
                                            // eprintln!("Error running tests: {}", e);
                                            
                                            // エラーポップアップを表示
                                            app.show_result_popup(format!("Error running release tests:\n{}", e));
                                        }
                                    }
                                },
                                PopupType::BuildToggle => {
                                    // リリースモードを切り替え
                                    app.close_popup();
                                    app.toggle_release_mode();
                                    
                                    // 結果ポップアップを表示
                                    app.show_result_popup(format!(
                                        "Build mode changed to: {}",
                                        if app.release_mode { "RELEASE" } else { "DEBUG" }
                                    ));
                                    
                                    // ターミナルを再初期化
                                    reset_terminal(&mut terminal)?;
                                },
                                PopupType::None | PopupType::ResultNotification => {},
                            }
                        } else if app.tab_index == 4 {
                            // 履歴タブでEnterキーを押した場合、選択した履歴を表示
                            if let Some(history) = app.history.get(app.selected_history) {
                                app.test_results = history.test_results.clone();
                                app.release_mode = history.release_mode;
                                
                                // 結果タブに切り替え
                                app.tab_index = 0;
                                
                                // 結果ポップアップを表示
                                app.show_result_popup(format!(
                                    "Loaded history entry #{}\nTimestamp: {}",
                                    app.selected_history + 1,
                                    chrono::Utc.timestamp_opt(history.timestamp as i64, 0).unwrap()
                                        .format("%Y-%m-%d %H:%M:%S")
                                ));
                                
                                reset_terminal(&mut terminal)?;
                            }
                        }
                    },
                    KeyCode::Esc => {
                        if app.show_popup {
                            // ポップアップを閉じる
                            app.close_popup();
                        } else if app.show_help {
                            app.toggle_help();
                        } else if app.result_popup_visible {
                            // 結果ポップアップを閉じる
                            app.result_popup_visible = false;
                            app.result_popup_time = None;
                        }
                    },
                    _ => {}
                }
            }
        }
    }
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
    )?;
    terminal.show_cursor()?;
    
    Ok(())
}
