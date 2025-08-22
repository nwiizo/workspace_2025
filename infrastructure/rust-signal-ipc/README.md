# Rust Signal IPC - シグナル処理とプロセス間通信の検証システム

Rustでシグナル処理とプロセス間通信（IPC）を実装し、包括的な検証システムを含むプロジェクトです。モダンなRust 2025パターン（CancellationToken、UUID相関、async closures）を採用しています。

## 🚀 2025年の最新動向

- **Rust 1.85.0**: async closuresの安定化により、より表現力豊かな非同期処理が可能に
- **Tokio**: async-stdの廃止（2025年3月予定）により事実上の標準に
- **高性能IPC**: ゼロコピーIPCライブラリの成熟
- **Linux Kernel**: Linux 6.13でRustサポートが転換点に到達

## 概要

このプロジェクトでは、以下の機能を実装しています：

### 基本機能
- **シグナル処理**: SIGINT、SIGTERM、SIGHUPの処理
- **Unix Domain Socket**: プロセス間通信の基本実装
- **メッセージプロトコル**: UUID付きシリアライズ可能なIPCメッセージ構造
- **Pub/Subパターン**: 複数クライアント間のメッセージ配信
- **グレイスフルシャットダウン**: CancellationTokenパターンによる安全な終了処理
- **相関ID**: リクエスト・レスポンスの追跡機能

### 検証システム（src/examples/）
- **基本シグナル処理の検証**: シグナルハンドラーの登録と配送
- **シグナルマスクの検証**: ブロッキングとペンディング状態
- **並行性テスト**: マルチスレッド環境での動作検証
- **パフォーマンス測定**: レイテンシとスループット
- **エッジケース処理**: 異常系とストレステスト

## プロジェクト構造

```
rust-signal-ipc/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # ライブラリのルート
│   ├── errors.rs                 # エラー型定義
│   ├── ipc.rs                    # IPCメッセージ構造
│   ├── examples/                 # 検証システム
│   │   ├── mod.rs
│   │   ├── basic_signal.rs       # 基本シグナル処理
│   │   ├── signal_mask.rs        # シグナルマスク
│   │   ├── thread_safety.rs      # スレッドセーフティ
│   │   ├── performance.rs        # パフォーマンス測定
│   │   ├── edge_cases.rs         # エッジケース
│   │   └── test_runner.rs        # 統合テストランナー
│   └── bin/                      # 実行可能バイナリ
│       ├── signal_handler.rs
│       ├── unix_socket_server.rs
│       ├── unix_socket_client.rs
│       ├── ipc_server.rs
│       ├── ipc_client.rs
│       ├── ipc_pubsub.rs
│       ├── ipc_multiclient.rs
│       ├── signal_validator.rs   # 検証システム実行
│       ├── graceful_shutdown.rs  # グレイスフルシャットダウン
│       └── async_retry.rs        # Async closures実装例
├── tests/
│   └── integration_test.rs
├── run_examples.sh               # デモスクリプト
└── demo.sh                       # IPCデモ
```

## ビルド

```bash
cargo build --release
```

## 実行例

### 1. シグナル処理検証システム

包括的なシグナル処理のテストを実行：

```bash
# 通常実行
cargo run --bin signal-validator

# 詳細モード
cargo run --bin signal-validator -- -v

# JSON出力
cargo run --bin signal-validator -- --json

# ベンチマークのみ
cargo run --bin signal-validator -- --benchmark
```

### 2. シグナルハンドラー

```bash
cargo run --bin signal-handler
```

別のターミナルで：
```bash
# SIGHUPを送信（設定再読み込み）
kill -HUP <PID>

# SIGTERMを送信（終了）
kill -TERM <PID>
```

### 3. IPCサーバー/クライアント（構造化メッセージ）

サーバー起動：
```bash
cargo run --bin ipc-server
```

クライアント実行：
```bash
cargo run --bin ipc-client
```

利用可能なコマンド：
- `ping` - 接続確認
- `time` - サーバーの現在時刻
- `info` - サーバー情報
- `calc:A op B` - 簡単な計算（例: `calc:10 + 20`）
- `notify TEXT` - 通知送信
- `heartbeat` - ハートビート
- `stress N` - ストレステスト（N個のメッセージ送信）

### 4. 複数クライアントテスト

```bash
# 10クライアントから各20メッセージを送信
cargo run --bin ipc-multiclient 10 20
```

### 5. Pub/Subパターン

```bash
# サーバー起動
cargo run --bin ipc-pubsub

# クライアントでトピック購読とパブリッシュ
cargo run --bin ipc-client
# SUB:news
# PUB:news:Breaking news!
# LIST
```

## テスト

```bash
cargo test
```

## 🔧 デバッグツール

### Linuxデバッグツール

#### strace - システムコールトレース

```bash
# 基本的な使い方
strace ./target/debug/my_program

# シグナル関連のシステムコールのみ
strace -e trace=signal,sigaction,kill,pause cargo run

# タイムスタンプ付きで表示
strace -t ./my_program
```

#### valgrind - メモリデバッガー

```bash
# メモリリークチェック
valgrind --leak-check=full ./target/debug/my_program

# データ競合検出
valgrind --tool=helgrind ./target/debug/my_program
```

#### perf - パフォーマンス分析

```bash
# CPUプロファイリング
perf record -g ./target/release/my_program
perf report

# リアルタイム統計
perf stat ./target/release/my_program
```

### Rust専用デバッグツール

#### cargo-expand - マクロ展開

```bash
cargo install cargo-expand
cargo expand
```

#### cargo-flamegraph - フレームグラフ生成

```bash
cargo install flamegraph
cargo flamegraph
```

#### tokio-console - 非同期ランタイムデバッグ

```bash
# インストール
cargo install --locked tokio-console

# アプリケーション起動（別ターミナル）
RUSTFLAGS="--cfg tokio_unstable" cargo run

# コンソールで監視
tokio-console
```

#### miri - 未定義動作検出

```bash
rustup +nightly component add miri
cargo +nightly miri test
```

### 環境変数の設定

```bash
# バックトレースを有効化
export RUST_BACKTRACE=1

# ログレベルの設定
export RUST_LOG=debug

# フレームポインタを強制的に使用（プロファイリング精度向上）
export RUSTFLAGS="-C force-frame-pointers=yes"
```

## 新機能（2025パターン実装）

### Async Closures（Rust 1.85.0の新機能）

```bash
cargo run --bin async-retry
```

Rust 1.85.0で安定化されたasync closuresを使った実装例：
- 指数バックオフ付きリトライ
- 並列実行ヘルパー
- チェーン処理
- タイムアウト付き実行

## 新機能（2024パターン実装）

### CancellationTokenによるグレイスフルシャットダウン

```bash
cargo run --bin graceful-shutdown
```

複数のワーカータスクが協調的に終了する様子を確認できます：
- データ処理ワーカー
- API処理ワーカー
- バックグラウンドジョブワーカー
- メトリクス表示タスク

Ctrl+Cで安全にシャットダウンし、各ワーカーが適切にクリーンアップ処理を行います。

### UUID相関IDによるメッセージ追跡

すべてのIPCメッセージに一意のIDと相関IDが付与されます：

```rust
pub struct IPCMessage {
    pub id: Uuid,                     // メッセージの一意識別子
    pub correlation_id: Option<Uuid>, // リクエスト・レスポンスの対応付け
    // ...
}
```

### 高度なエラーハンドリング

エラーの種類に応じた適切な処理：

```rust
if error.is_retryable() {
    // 再試行可能なエラー（ネットワーク一時障害など）
    retry_with_backoff();
} else if error.is_fatal() {
    // 致命的エラー（プロトコル違反など）
    shutdown_connection();
}
```

## 主な機能

### エラーハンドリング

`thiserror`を使用した構造化されたエラー型：
- I/Oエラー
- シリアライゼーションエラー
- プロトコルエラー
- 接続エラー

### IPCメッセージ

bincode形式でシリアライズ可能なメッセージ構造：
- メッセージタイプ（Request、Response、Notification等）
- ペイロード（最大1MB）
- タイムスタンプ検証

### シグナル処理

signal-hookクレートを使用した安全なシグナル処理：
- SIGINT（Ctrl+C）
- SIGTERM（終了シグナル）
- SIGHUP（設定再読み込み）

## 依存関係

### コア機能
- `signal-hook` - シグナル処理
- `nix` - Unix系システムコール
- `libc` - 低レベルシステムAPI

### 非同期処理
- `tokio` - 非同期ランタイム
- `tokio-util` - CancellationToken等のユーティリティ
- `futures` - 非同期プリミティブ

### データ処理
- `uuid` - UUID生成と管理
- `serde` / `bincode` - シリアライゼーション
- `chrono` - タイムスタンプ処理

### エラーハンドリング・ログ
- `thiserror` - エラー型定義
- `anyhow` - エラーハンドリング（バイナリ用）
- `tracing` - 構造化ログ出力

## 今後の拡張

### 実装済み
- ✅ 非同期処理（tokio）の統合
- ✅ UUID相関による高度なメッセージプロトコル
- ✅ CancellationTokenによるグレイスフル・シャットダウン
- ✅ エラーの再試行可能/致命的判定

### 今後の実装予定
- Server::Starterパターン（ゼロダウンタイム更新）
- Prometheusフォーマットでのメトリクス収集
- プロセスプール管理
- バックプレッシャー制御
- 分散トレーシング対応

## ライセンス

MIT