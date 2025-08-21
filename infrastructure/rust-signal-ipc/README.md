# Rust Signal IPC - シグナル処理とプロセス間通信の基本実装

Rustでシグナル処理とプロセス間通信（IPC）を実装した基本的なプロジェクトです。

## 概要

このプロジェクトでは、以下の基本的な機能を実装しています：

- **シグナル処理**: SIGINT、SIGTERM、SIGHUPの処理
- **Unix Domain Socket**: プロセス間通信の基本実装
- **メッセージプロトコル**: シリアライズ可能なIPCメッセージ構造

## プロジェクト構造

```
rust-signal-ipc/
├── Cargo.toml
├── src/
│   ├── lib.rs              # ライブラリのルート
│   ├── errors.rs           # エラー型定義
│   ├── ipc.rs             # IPCメッセージ構造
│   └── bin/               # 実行可能バイナリ
│       ├── signal_handler.rs
│       ├── unix_socket_server.rs
│       └── unix_socket_client.rs
└── tests/
    └── integration_test.rs # 統合テスト
```

## ビルド

```bash
cargo build --release
```

## 実行例

### 1. シグナルハンドラー

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

### 2. Unix Domain Socket通信

サーバーを起動：
```bash
cargo run --bin unix-socket-server
```

別のターミナルでクライアントを起動：
```bash
cargo run --bin unix-socket-client
```

クライアントで使えるコマンド：
- `ping` - サーバーの応答確認
- `time` - サーバーの現在時刻を取得
- `quit` - 終了
- その他 - エコーバック（大文字変換）

## テスト

```bash
cargo test
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

- `signal-hook` - シグナル処理
- `thiserror` - エラー型定義
- `anyhow` - エラーハンドリング（バイナリ用）
- `serde` / `bincode` - シリアライゼーション
- `chrono` - タイムスタンプ処理
- `tracing` - ログ出力

## 今後の拡張

このシンプルな実装から、以下のような拡張が可能です：

- 非同期処理（tokio）の追加
- より高度なメッセージプロトコル
- グレイスフル・シャットダウン
- メトリクス収集
- プロセスプール管理

## ライセンス

MIT