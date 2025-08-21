//! Process management module with ownership and lifetime best practices

use crate::errors::{ProcessError, ProcessResult};
use std::io;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

/// Builder pattern for creating processes with validation
#[derive(Debug)]
pub struct ProcessBuilder {
    command: String,
    args: Vec<String>,
    env_vars: Vec<(String, String)>,
    working_dir: Option<String>,
    stdin: Option<Stdio>,
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
    timeout: Option<Duration>,
}

impl ProcessBuilder {
    /// Create a new process builder
    pub fn new<S: Into<String>>(command: S) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            env_vars: Vec::new(),
            working_dir: None,
            stdin: None,
            stdout: None,
            stderr: None,
            timeout: None,
        }
    }

    /// Add an argument to the command
    pub fn arg<S: Into<String>>(mut self, arg: S) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Set timeout for the process
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Configure stdin
    pub fn stdin(mut self, cfg: Stdio) -> Self {
        self.stdin = Some(cfg);
        self
    }

    /// Configure stdout
    pub fn stdout(mut self, cfg: Stdio) -> Self {
        self.stdout = Some(cfg);
        self
    }

    /// Configure stderr
    pub fn stderr(mut self, cfg: Stdio) -> Self {
        self.stderr = Some(cfg);
        self
    }

    /// Set an environment variable
    pub fn env<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.env_vars.push((key.into(), value.into()));
        self
    }

    /// Set working directory
    pub fn current_dir<P: Into<String>>(mut self, dir: P) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Validate and build the command
    fn build_command(&mut self) -> ProcessResult<Command> {
        // Validate command
        if self.command.is_empty() {
            return Err(ProcessError::InvalidInput("Command cannot be empty".into()));
        }

        // セキュリティチェック：コマンドパスの検証
        validate_command_path(&self.command)?;

        let mut cmd = Command::new(&self.command);

        // Add arguments with validation
        for arg in &self.args {
            validate_input(arg)?;
            cmd.arg(arg);
        }

        // Set environment variables with validation
        for (key, value) in &self.env_vars {
            validate_env_var(key, value)?;
            cmd.env(key, value);
        }

        // Set working directory
        if let Some(ref dir) = self.working_dir {
            validate_path(dir)?;
            cmd.current_dir(dir);
        }

        // Configure stdio
        if let Some(stdin) = self.stdin.take() {
            cmd.stdin(stdin);
        }
        if let Some(stdout) = self.stdout.take() {
            cmd.stdout(stdout);
        }
        if let Some(stderr) = self.stderr.take() {
            cmd.stderr(stderr);
        }

        Ok(cmd)
    }

    /// Spawn the process and return a handle
    pub fn spawn(mut self) -> ProcessResult<ProcessGuard> {
        let name = self.command.clone();
        let timeout = self.timeout;
        let mut cmd = self.build_command()?;
        let child = cmd.spawn()?;

        Ok(ProcessGuard {
            child: Some(child),
            name,
            timeout,
        })
    }

    /// Execute with output capture
    pub fn output(mut self) -> ProcessResult<std::process::Output> {
        let mut cmd = self.build_command()?;
        Ok(cmd.output()?)
    }
}

/// RAII guard for process cleanup
pub struct ProcessGuard {
    child: Option<Child>,
    name: String,
    timeout: Option<Duration>,
}

impl ProcessGuard {
    /// Wait for the process to finish
    pub fn wait(&mut self) -> ProcessResult<ProcessOutput> {
        if let Some(mut child) = self.child.take() {
            let status = if let Some(timeout) = self.timeout {
                match wait_with_timeout(&mut child, timeout) {
                    Ok(status) => status,
                    Err(_) => {
                        child.kill()?;
                        child.wait()?; // 必ず待機してゾンビプロセスを防ぐ
                        return Err(ProcessError::TimeoutError {
                            seconds: timeout.as_secs(),
                        });
                    }
                }
            } else {
                child.wait()?
            };

            Ok(ProcessOutput {
                status: status.code(),
                success: status.success(),
            })
        } else {
            Err(ProcessError::ProcessTerminated { pid: 0 })
        }
    }
}

impl Drop for ProcessGuard {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            // Try graceful termination first
            if child.try_wait().ok().and_then(|s| s).is_none() {
                eprintln!("ProcessGuard: Terminating process '{}'", self.name);

                // まずSIGTERMを送信（Unix系）
                #[cfg(unix)]
                {
                    let pid = nix::unistd::Pid::from_raw(child.id() as i32);
                    let _ = nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGTERM);
                    std::thread::sleep(Duration::from_millis(100));
                }

                // まだ生きていればSIGKILL
                if child.try_wait().ok().and_then(|s| s).is_none() {
                    let _ = child.kill();
                }

                // 必ず待機してゾンビプロセスを防ぐ
                let _ = child.wait();
            }
        }
    }
}

/// Process execution output
#[derive(Debug, Clone)]
pub struct ProcessOutput {
    pub status: Option<i32>,
    pub success: bool,
}

/// Helper function for timeout implementation
fn wait_with_timeout(child: &mut Child, timeout: Duration) -> io::Result<std::process::ExitStatus> {
    let start = std::time::Instant::now();

    loop {
        match child.try_wait()? {
            Some(status) => return Ok(status),
            None => {
                if start.elapsed() >= timeout {
                    return Err(io::Error::new(io::ErrorKind::TimedOut, "Process timed out"));
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

/// Input validator for security
pub fn validate_input(input: &str) -> ProcessResult<&str> {
    // 危険な文字のチェック（2025年のベストプラクティス）
    const DANGEROUS_CHARS: &[char] = &[
        ';', '&', '|', '$', '`', '>', '<', '(', ')', '{', '}', '\n', '\r',
        '\0', // ヌルバイト攻撃対策
        '\'', '"', // クォート攻撃対策
    ];

    for c in DANGEROUS_CHARS {
        if input.contains(*c) {
            let char_str = if *c == '\0' {
                "\\0".to_string()
            } else {
                c.to_string()
            };
            return Err(ProcessError::InvalidInput(format!(
                "Input contains dangerous character '{}'",
                char_str
            )));
        }
    }

    // パストラバーサル攻撃のチェック
    if input.contains("..") || input.contains("~") {
        return Err(ProcessError::InvalidInput("Path traversal detected".into()));
    }

    // コマンドインジェクションパターンのチェック
    let dangerous_patterns = ["$(", "${", "[[", "]]", "&&", "||"];

    for pattern in dangerous_patterns {
        if input.contains(pattern) {
            return Err(ProcessError::InvalidInput(format!(
                "Dangerous pattern '{}' detected",
                pattern
            )));
        }
    }

    Ok(input)
}

/// Validate command path
fn validate_command_path(cmd: &str) -> ProcessResult<()> {
    // 絶対パスの場合は存在確認
    if cmd.starts_with('/') && !std::path::Path::new(cmd).exists() {
        return Err(ProcessError::InvalidInput(format!(
            "Command not found: {}",
            cmd
        )));
    }

    // 相対パスは禁止（セキュリティ上の理由）
    if cmd.contains('/') && !cmd.starts_with('/') {
        return Err(ProcessError::InvalidInput(
            "Relative paths are not allowed for security reasons".into(),
        ));
    }

    validate_input(cmd)?;
    Ok(())
}

/// Validate environment variable
fn validate_env_var(key: &str, value: &str) -> ProcessResult<()> {
    // 環境変数名の検証
    if !key.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(ProcessError::InvalidInput(format!(
            "Invalid environment variable name: {}",
            key
        )));
    }

    // 値の検証（より緩い制約）
    if value.contains('\0') {
        return Err(ProcessError::InvalidInput(
            "Environment variable value contains null byte".into(),
        ));
    }

    Ok(())
}

/// Validate path
fn validate_path(path: &str) -> ProcessResult<()> {
    validate_input(path)?;

    // 追加のパス検証
    if !std::path::Path::new(path).exists() {
        return Err(ProcessError::InvalidInput(format!(
            "Path does not exist: {}",
            path
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_validation() {
        assert!(validate_input("normal_file.txt").is_ok());
        assert!(validate_input("file.txt; rm -rf /").is_err());
        assert!(validate_input("../../../etc/passwd").is_err());
        assert!(validate_input("$(whoami)").is_err());
        assert!(validate_input("file\0name").is_err());
        assert!(validate_input("~/secret").is_err());
        assert!(validate_input("cmd && malicious").is_err());
    }

    #[test]
    fn test_env_var_validation() {
        assert!(validate_env_var("MY_VAR", "value").is_ok());
        assert!(validate_env_var("MY-VAR", "value").is_err());
        assert!(validate_env_var("MYVAR", "value\0").is_err());
    }
}
