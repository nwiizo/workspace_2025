# Rust Linux Process Management Examples

Rustを使ったLinuxプロセス管理の包括的なサンプル集です。

## サンプル一覧

### 1. 基本的なプロセス操作 (`basic_process`)
- `std::process::Command`を使った基本的なプロセス起動
- パイプを使った入出力制御
- 環境変数とワーキングディレクトリの設定
- Unix固有の機能

### 2. Fork/Execパターン (`fork_exec`)
- `nix`クレートを使用したfork()の実装
- fork + execによるプロセスの置き換え
- 複数の子プロセスの管理

### 3. シグナル処理 (`signal_handling`)
- `signal-hook`を使った安全なシグナル処理
- 子プロセスへのシグナル送信
- グレースフルシャットダウンの実装

### 4. ゾンビプロセス対策 (`zombie_prevention`)
- ゾンビプロセスの発生と回収
- SIGCHLDハンドラによる自動回収
- ダブルフォークによる孤児プロセス化

### 5. 非同期プロセス管理 (`async_process`)
- Tokioを使った非同期プロセス実行
- ストリーミング出力の処理
- 複数プロセスの並行実行
- タイムアウト処理

### 6. プロセスグループ管理 (`process_group`)
- プロセスグループの作成と管理
- セッションリーダーの作成
- デーモン化の実装

### 7. セキュアなプロセス起動 (`secure_spawn`)
- 入力検証とサニタイゼーション
- 権限の削除
- 環境変数のクリーンアップ
- リソース制限
- 自動クリーンアップ

## ビルドと実行

### 依存関係のインストール
```bash
cargo build
```

### 個別のサンプルを実行
```bash
# 基本的なプロセス操作
cargo run --bin basic_process

# Fork/Execパターン（Linux/Unix環境のみ）
cargo run --bin fork_exec

# シグナル処理
cargo run --bin signal_handling

# ゾンビプロセス対策（Linux/Unix環境のみ）
cargo run --bin zombie_prevention

# 非同期プロセス管理
cargo run --bin async_process

# プロセスグループ管理（Linux/Unix環境のみ）
cargo run --bin process_group

# セキュアなプロセス起動
cargo run --bin secure_spawn
```

## 注意事項

- 一部のサンプル（fork、シグナル処理、プロセスグループ管理など）はLinux/Unix環境でのみ動作します
- macOSでは一部の機能が制限される場合があります
- Windows環境では基本的なプロセス操作と非同期処理のサンプルのみ動作します

## 開発環境

- Rust 1.70以上
- Linux/Unix環境推奨（完全な機能を利用する場合）