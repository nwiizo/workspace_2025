use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestConfig {
    pub tests: Vec<TestCase>,
    pub build: Option<BuildConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildConfig {
    pub release: bool,
    pub pre_build_commands: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestCase {
    pub name: String,
    pub command: String,
    pub args: Option<Vec<String>>,
    pub input: Option<String>,
    pub expected_output: String,
    pub timeout_secs: Option<u64>,
    pub build: Option<BuildConfig>,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub success: bool,
    pub actual_output: String,
    pub diff: Option<Vec<DiffLine>>,
    pub command: String,
    pub args: Vec<String>,
    pub input: Option<String>,
    pub execution_time: Duration,
    pub is_release: bool,
    pub build_commands: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub tag: ChangeTag,
    pub content: String,
}

pub fn load_config(config_path: &PathBuf) -> Result<TestConfig> {
    // Check if file exists
    if !config_path.exists() {
        return Err(anyhow::anyhow!(
            "Config file not found: {}. Please make sure the file exists in the specified path.",
            config_path.display()
        ));
    }
    
    // ファイルを開く
    let content = std::fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config file: {:?}", config_path))?;
    
    // ファイル拡張子を取得して小文字に変換
    let extension = config_path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .unwrap_or_default();
    
    // 拡張子に応じて適切なパーサーを使用
    match extension.as_str() {
        "yaml" | "yml" => {
            // コマンド出力を抑制
            // println!("Parsing YAML configuration from {}", config_path.display());
            serde_yaml::from_str(&content)
                .map_err(|e| anyhow::anyhow!("YAML parse error: {}", e))
        },
        "toml" => {
            // コマンド出力を抑制
            // println!("Parsing TOML configuration from {}", config_path.display());
            toml::from_str(&content)
                .map_err(|e| anyhow::anyhow!("TOML parse error: {}", e))
        },
        _ => {
            Err(anyhow::anyhow!("Unsupported configuration format: {}. Please use .yaml, .yml, or .toml files.", extension))
        }
    }
}

// テンプレート変数を処理する関数
fn process_template(template: &str, is_release: bool) -> String {
    // {{#if release}}--release{{/if}} 形式のテンプレートを処理
    let release_template = "{{#if release}}";
    let release_end = "{{/if}}";
    
    if template.contains(release_template) {
        let mut result = String::new();
        let mut current_pos = 0;
        
        while let Some(start_pos) = template[current_pos..].find(release_template) {
            let start_pos = current_pos + start_pos;
            result.push_str(&template[current_pos..start_pos]);
            
            if let Some(end_pos) = template[start_pos..].find(release_end) {
                let end_pos = start_pos + end_pos;
                let conditional_content = &template[start_pos + release_template.len()..end_pos];
                
                if is_release {
                    result.push_str(conditional_content);
                }
                
                current_pos = end_pos + release_end.len();
            } else {
                // 終了タグが見つからない場合は残りをそのまま追加
                result.push_str(&template[start_pos..]);
                current_pos = template.len();
            }
        }
        
        // 残りの部分を追加
        if current_pos < template.len() {
            result.push_str(&template[current_pos..]);
        }
        
        result
    } else if template.contains("{{#if build.release}}") {
        // {{#if build.release}}release{{else}}debug{{/if}} 形式のテンプレートを処理
        let build_release_template = "{{#if build.release}}";
        let build_else = "{{else}}";
        let build_end = "{{/if}}";
        
        let mut result = String::new();
        let mut current_pos = 0;
        
        while let Some(start_pos) = template[current_pos..].find(build_release_template) {
            let start_pos = current_pos + start_pos;
            result.push_str(&template[current_pos..start_pos]);
            
            if let Some(else_pos) = template[start_pos..].find(build_else) {
                let else_pos = start_pos + else_pos;
                
                if let Some(end_pos) = template[else_pos..].find(build_end) {
                    let end_pos = else_pos + end_pos;
                    
                    let if_content = &template[start_pos + build_release_template.len()..else_pos];
                    let else_content = &template[else_pos + build_else.len()..end_pos];
                    
                    if is_release {
                        result.push_str(if_content);
                    } else {
                        result.push_str(else_content);
                    }
                    
                    current_pos = end_pos + build_end.len();
                } else {
                    // 終了タグが見つからない場合は残りをそのまま追加
                    result.push_str(&template[start_pos..]);
                    current_pos = template.len();
                }
            } else {
                // elseタグが見つからない場合は残りをそのまま追加
                result.push_str(&template[start_pos..]);
                current_pos = template.len();
            }
        }
        
        // 残りの部分を追加
        if current_pos < template.len() {
            result.push_str(&template[current_pos..]);
        }
        
        result
    } else {
        // テンプレート変数がない場合はそのまま返す
        template.to_string()
    }
}

// ビルド前のコマンドを実行する関数
pub fn run_pre_build_commands(config: &TestConfig) -> Result<()> {
    if let Some(build) = &config.build {
        if let Some(commands) = &build.pre_build_commands {
            for cmd_template in commands {
                // テンプレート変数を処理
                let cmd = process_template(cmd_template, build.release);
                
                // コマンド出力を抑制
                // println!("Running pre-build command: {}", cmd);
                
                // コマンドを実行
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .output()
                    .with_context(|| format!("Failed to execute pre-build command: {}", cmd))?;
                
                if !output.status.success() {
                    return Err(anyhow::anyhow!("Pre-build command failed: {}", cmd));
                }
                
                // コマンド出力を抑制
                // let stdout = String::from_utf8_lossy(&output.stdout);
                // println!("{}", stdout.trim());
            }
        }
    }
    
    Ok(())
}

pub fn run_tests(config: &TestConfig) -> Result<Vec<TestResult>> {
    // ビルド前のコマンドを実行
    run_pre_build_commands(config)?;
    
    let mut results = Vec::new();
    let global_release = config.build.as_ref().map_or(false, |b| b.release);
    
    for test in &config.tests {
        // コマンド出力を抑制
        // println!("Running test: {}", test.name);
        
        // テスト固有のビルド設定があれば実行
        if let Some(build) = &test.build {
            run_test_build_commands(test, build)?;
        }
        
        // テスト固有のリリースモード設定があればそれを使用、なければグローバル設定を使用
        let is_release = test.build.as_ref().map_or(global_release, |b| b.release);
        
        let mut command = Command::new(&test.command);
        
        // Process arguments if provided
        let processed_args = if let Some(args) = &test.args {
            // テンプレート変数を処理
            let processed: Vec<String> = args.iter()
                .map(|arg| process_template(arg, is_release))
                .collect();
            
            command.args(&processed);
            processed
        } else {
            Vec::new()
        };
        
        // Setup stdin if input is provided
        let start_time = std::time::Instant::now();
        
        let mut child = if let Some(_input) = &test.input {
            command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .with_context(|| format!("Failed to spawn command: {}", test.command))?
        } else {
            command
                .stdout(Stdio::piped())
                .spawn()
                .with_context(|| format!("Failed to spawn command: {}", test.command))?
        };
        
        // Write to stdin if input is provided
        if let Some(input) = &test.input {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(input.as_bytes())
                    .context("Failed to write to stdin")?;
                // 標準入力をクローズして、コマンドが入力の終了を認識できるようにする
                // drop(stdin)は自動的に行われる
            }
        }
        
        // Get output with timeout
        let timeout = Duration::from_secs(test.timeout_secs.unwrap_or(30));
        let output_status = child.wait_timeout(timeout)
            .context("Command execution failed")?;
        
        let execution_time = start_time.elapsed();
        
        let output = if output_status.is_some() {
            child.wait_with_output()?
        } else {
            child.kill()?;
            return Err(anyhow::anyhow!("Command timed out: {}", test.name));
        };
        
        let actual_output = String::from_utf8_lossy(&output.stdout).to_string();
        let success = actual_output.trim() == test.expected_output.trim();
        
        // Generate diff if test failed
        let diff = if !success {
            let text_diff = TextDiff::from_lines(&test.expected_output, &actual_output);
            
            let mut diff_lines = Vec::new();
            for change in text_diff.iter_all_changes() {
                diff_lines.push(DiffLine {
                    tag: change.tag(),
                    content: change.value().to_string(),
                });
            }
            
            Some(diff_lines)
        } else {
            None
        };
        
        // Extract build commands if available
        let build_commands = test.build.as_ref().and_then(|b| {
            b.pre_build_commands.clone()
        });
        
        results.push(TestResult {
            name: test.name.clone(),
            success,
            actual_output,
            diff,
            command: test.command.clone(),
            args: processed_args,
            input: test.input.clone(),
            execution_time,
            is_release,
            build_commands,
        });
    }
    
    Ok(results)
}

// テスト固有のビルドコマンドを実行する関数
fn run_test_build_commands(test: &TestCase, build: &BuildConfig) -> Result<()> {
    if let Some(commands) = &build.pre_build_commands {
        for cmd_template in commands {
            // テンプレート変数を処理
            let cmd = process_template(cmd_template, build.release);
            
            // コマンド出力を抑制
            // println!("Running pre-build command for test '{}': {}", test.name, cmd);
            
            // コマンドを実行
            let output = Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .output()
                .with_context(|| format!("Failed to execute pre-build command: {}", cmd))?;
            
            if !output.status.success() {
                return Err(anyhow::anyhow!("Pre-build command failed for test '{}': {}", test.name, cmd));
            }
            
            // コマンド出力を抑制
            // let stdout = String::from_utf8_lossy(&output.stdout);
            // println!("{}", stdout.trim());
        }
    }
    
    Ok(())
}

// Extension trait for Command to add wait_timeout functionality
pub trait CommandExt {
    fn wait_timeout(&mut self, timeout: Duration) -> Result<Option<std::process::ExitStatus>>;
}

impl CommandExt for std::process::Child {
    fn wait_timeout(&mut self, timeout: Duration) -> Result<Option<std::process::ExitStatus>> {
        // 最初に即時終了しているかチェック
        match self.try_wait()? {
            Some(status) => return Ok(Some(status)),
            None => {}
        }
        
        // タイムアウト処理
        // 実際のアプリケーションでは、より洗練された方法を使用すべきです
        let start = std::time::Instant::now();
        
        while start.elapsed() < timeout {
            match self.try_wait()? {
                Some(status) => return Ok(Some(status)),
                None => {
                    // 短い間隔でポーリング
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
        
        // タイムアウト
        Ok(None)
    }
} 