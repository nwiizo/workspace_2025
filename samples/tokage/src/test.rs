use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use std::{
    fs::File,
    io::{BufReader, Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct TestConfig {
    pub tests: Vec<TestCase>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub command: String,
    pub args: Option<Vec<String>>,
    pub input: Option<String>,
    pub expected_output: String,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug)]
pub struct TestResult {
    pub name: String,
    pub success: bool,
    pub actual_output: String,
    pub diff: Option<Vec<DiffLine>>,
}

#[derive(Debug)]
pub struct DiffLine {
    pub tag: ChangeTag,
    pub content: String,
}

pub fn load_config(config_path: &PathBuf) -> Result<TestConfig> {
    // ファイルを開く
    let file = File::open(config_path)
        .with_context(|| format!("Failed to open config file: {:?}", config_path))?;
    let mut reader = BufReader::new(file);
    
    // ファイル拡張子を取得
    let extension = config_path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    
    // ファイル内容を文字列として読み込む
    let mut content = String::new();
    reader.read_to_string(&mut content)
        .with_context(|| format!("Failed to read config file: {:?}", config_path))?;
    
    // 拡張子に応じて適切なパーサーを使用
    match extension.to_lowercase().as_str() {
        "yaml" | "yml" => {
            match serde_yaml::from_str::<TestConfig>(&content) {
                Ok(config) => Ok(config),
                Err(e) => Err(anyhow::anyhow!("Failed to parse YAML config: {}", e))
            }
        },
        "toml" => {
            match toml::from_str::<TestConfig>(&content) {
                Ok(config) => Ok(config),
                Err(e) => Err(anyhow::anyhow!("Failed to parse TOML config: {}", e))
            }
        },
        _ => Err(anyhow::anyhow!("Unsupported config file format: {}", extension))
    }
}

pub fn run_tests(tests: &[TestCase]) -> Result<Vec<TestResult>> {
    let mut results = Vec::new();
    
    for test in tests {
        println!("Running test: {}", test.name);
        
        let mut command = Command::new(&test.command);
        
        // Add arguments if provided
        if let Some(args) = &test.args {
            command.args(args);
        }
        
        // Setup stdin if input is provided
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
        
        results.push(TestResult {
            name: test.name.clone(),
            success,
            actual_output,
            diff,
        });
    }
    
    Ok(results)
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