# Nix Crate Guide - Rust System Programming

## 概要

`nix`クレートは、各種*nixシステム関数へのRust向けの安全なバインディングを提供します。システムコールを`Result<T, E>`でラップし、Rustらしいエラーハンドリングを実現します。

## なぜnixクレートを使うのか

1. **型安全性**: Cのシステムコールを型安全なRust APIでラップ
2. **エラーハンドリング**: `Result`型による明示的なエラー処理
3. **メモリ安全性**: 手動メモリ管理の必要なし
4. **クロスプラットフォーム**: Unix系OS間の差異を吸収

## 基本的な使い方

### Cargo.tomlへの追加

```toml
[dependencies]
nix = { version = "0.27", features = [
    "signal",     # シグナル処理
    "process",    # プロセス管理
    "user",       # ユーザー/グループ操作
    "mount",      # マウント操作
    "sched",      # スケジューリング
    "resource"    # リソース制限
]}
```

## プロセス管理

### 1. 基本的なfork

```rust
use nix::sys::wait::waitpid;
use nix::unistd::{fork, ForkResult};

fn basic_fork() -> Result<(), Box<dyn std::error::Error>> {
    match unsafe { fork()? } {
        ForkResult::Parent { child } => {
            println!("子プロセスPID: {}", child);
            let status = waitpid(child, None)?;
            println!("終了ステータス: {:?}", status);
        }
        ForkResult::Child => {
            println!("子プロセスです");
            std::process::exit(0);
        }
    }
    Ok(())
}
```

**実行例:**
```bash
cargo run --bin nix_fork_basic
```

### 2. 複数ワーカープロセス

```rust
fn create_workers(count: usize) -> Result<(), Box<dyn std::error::Error>> {
    for i in 0..count {
        match unsafe { fork()? } {
            ForkResult::Parent { child } => {
                println!("ワーカー{} PID: {}", i, child);
            }
            ForkResult::Child => {
                // ワーカー処理
                worker_task(i);
                std::process::exit(0);
            }
        }
    }
    
    // 全ワーカーの終了を待つ
    while let Ok(status) = waitpid(None, None) {
        println!("ワーカー終了: {:?}", status);
    }
    Ok(())
}
```

**実行例:**
```bash
cargo run --bin nix_multi_worker
```

### 3. Fork-Execパターン

最も安全なパターン。マルチスレッド環境でも安全:

```rust
use nix::unistd::execvp;
use std::ffi::CString;

match unsafe { fork()? } {
    ForkResult::Parent { child } => {
        waitpid(child, None)?;
    }
    ForkResult::Child => {
        let cmd = CString::new("ls").unwrap();
        let args = vec![
            CString::new("ls").unwrap(),
            CString::new("-la").unwrap(),
        ];
        execvp(&cmd, &args)?;
        unreachable!();
    }
}
```

**実行例:**
```bash
cargo run --bin nix_fork_exec
```

## シグナル処理

### シグナルハンドラの設定

```rust
use nix::sys::signal::{self, Signal, SigHandler};

extern "C" fn handle_sigchld(_: i32) {
    // 子プロセスを刈り取る
    use nix::sys::wait::{waitpid, WaitPidFlag};
    while let Ok(_) = waitpid(None, Some(WaitPidFlag::WNOHANG)) {}
}

fn setup_handlers() -> nix::Result<()> {
    unsafe {
        // SIGCHLDでゾンビプロセスを防ぐ
        signal::signal(Signal::SIGCHLD, SigHandler::Handler(handle_sigchld))?;
        
        // SIGPIPEを無視
        signal::signal(Signal::SIGPIPE, SigHandler::SigIgn)?;
    }
    Ok(())
}
```

**実行例:**
```bash
cargo run --bin nix_signal_handler
# Ctrl+Cで優雅な終了をテスト
```

## デーモン化

ダブルフォーク技法によるデーモン作成:

```rust
use nix::unistd::{fork, setsid, ForkResult};

fn daemonize() -> nix::Result<()> {
    // 最初のfork
    match unsafe { fork()? } {
        ForkResult::Parent { .. } => std::process::exit(0),
        ForkResult::Child => {}
    }
    
    // 新しいセッションを作成
    setsid()?;
    
    // 2回目のfork（制御端末の取得を防ぐ）
    match unsafe { fork()? } {
        ForkResult::Parent { .. } => std::process::exit(0),
        ForkResult::Child => {}
    }
    
    Ok(())
}
```

**実行例:**
```bash
cargo run --bin nix_daemon
# ログを確認
tail -f /tmp/rust_daemon.log
```

## 重要な安全性の考慮事項

### fork()の安全性ルール

#### シングルスレッドプログラム
- 一般的に`fork()`を自由に使用可能
- 子プロセスで複雑な処理も可能

#### マルチスレッドプログラム
fork後、子プロセスは以下に限定すべき:
- async-signal-safe関数のみ呼び出す
- 即座に`exec*()`を呼ぶ
- メモリ割り当てを避ける（async-signal-safeではない）

### Async-Signal-Safe関数

**安全:**
- `_exit()`, `execve()`, `dup2()`, `close()`
- `pipe()`, `read()`, `write()`（注意事項あり）
- 単純なシグナル操作

**危険:**
- `malloc()`, `free()`（メモリ割り当て）
- `printf()`などのstdio関数
- ミューテックス操作
- ほとんどのRust標準ライブラリ関数

## エラーハンドリング

```rust
use nix::errno::Errno;

match some_nix_operation() {
    Ok(result) => handle_success(result),
    Err(Errno::EAGAIN) => {
        // リトライ
        retry_operation()
    }
    Err(Errno::EPERM) => {
        // 権限エラー
        eprintln!("Permission denied");
    }
    Err(e) => {
        // その他のエラー
        return Err(e.into());
    }
}
```

## waitpidの返り値

`waitpid`は`WaitStatus`列挙型を返します:

```rust
use nix::sys::wait::WaitStatus;

match waitpid(child_pid, None)? {
    WaitStatus::Exited(pid, code) => {
        println!("PID {} が終了コード {} で終了", pid, code);
    }
    WaitStatus::Signaled(pid, sig, core_dumped) => {
        println!("PID {} がシグナル {:?} で終了", pid, sig);
    }
    WaitStatus::Stopped(pid, sig) => {
        println!("PID {} がシグナル {:?} で停止", pid, sig);
    }
    WaitStatus::Continued(pid) => {
        println!("PID {} が再開", pid);
    }
    _ => {}
}
```

## プラットフォーム固有の機能

```rust
#[cfg(target_os = "linux")]
use nix::sys::prctl;  // Linux固有

#[cfg(target_os = "freebsd")]
use nix::sys::procctl;  // FreeBSD固有

#[cfg(any(target_os = "linux", target_os = "android"))]
fn linux_specific_feature() {
    // Linux/Android固有の処理
}
```

## ベストプラクティス

### 1. リソース管理にRAIIを使用

```rust
use nix::unistd::close;

struct FileDescriptor(i32);

impl Drop for FileDescriptor {
    fn drop(&mut self) {
        let _ = close(self.0);
    }
}
```

### 2. 明示的なエラー処理

```rust
fn safe_fork() -> nix::Result<ForkResult> {
    unsafe { fork() }
}

// 使用時
match safe_fork() {
    Ok(result) => { /* 処理 */ }
    Err(e) => eprintln!("Fork failed: {}", e),
}
```

### 3. 子プロセスでの早期exit

```rust
ForkResult::Child => {
    // 最小限の処理のみ
    if let Err(e) = child_work() {
        eprintln!("Child error: {}", e);
        std::process::exit(1);
    }
    std::process::exit(0);
}
```

## テスト

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fork() {
        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                let status = waitpid(child, None).unwrap();
                assert!(matches!(
                    status,
                    WaitStatus::Exited(_, 0)
                ));
            }
            Ok(ForkResult::Child) => {
                std::process::exit(0);
            }
            Err(_) => panic!("Fork failed"),
        }
    }
}
```

## トラブルシューティング

### ゾンビプロセス
- `SIGCHLD`ハンドラを設定
- `waitpid()`で子プロセスを刈り取る

### リソースリーク
- ファイルディスクリプタを適切にクローズ
- fork後、不要なFDをクローズ

### デッドロック
- fork後、子プロセスでミューテックスを避ける
- exec前に最小限の処理のみ

## 実行可能なサンプル

このプロジェクトには以下のサンプルが含まれています:

1. `nix_fork_basic` - 基本的なforkの例
2. `nix_multi_worker` - 複数ワーカープロセス
3. `nix_fork_exec` - fork-execパターン
4. `nix_signal_handler` - シグナル処理
5. `nix_daemon` - デーモンプロセス作成

各サンプルを実行:
```bash
cargo run --bin <サンプル名>
```

## 参考資料

- [nix documentation](https://docs.rs/nix)
- [Linux man pages](https://man7.org/linux/man-pages/)
- [Advanced Programming in the UNIX Environment](https://www.apuebook.com/)
- [The Linux Programming Interface](https://man7.org/tlpi/)