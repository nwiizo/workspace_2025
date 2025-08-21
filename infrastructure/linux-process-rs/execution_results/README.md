# Linux Process Management - 実行結果

このディレクトリには、各サンプルプログラムの実行結果が保存されています。

## ファイル一覧

- `basic_process.txt` - 基本的なプロセス操作の実行結果
- `fork_exec.txt` - Fork/Execパターンの実行結果
- `signal_handling.txt` - シグナル処理の実行結果
- `zombie_prevention.txt` - ゾンビプロセス対策の実行結果（部分）
- `async_process.txt` - 非同期プロセス管理の実行結果
- `process_group.txt` - プロセスグループ管理の実行結果
- `secure_spawn.txt` - セキュアなプロセス起動の実行結果

## 実行環境

- OS: macOS Darwin 24.6.0
- Rust: 1.70+
- 実行日時: 2025-08-19

## 注意事項

- 一部のサンプル（fork、シグナル処理など）はLinux/Unix環境でのみ完全に動作します
- macOSでは一部の機能（リソース制限など）が制限される場合があります
- 長時間実行されるサンプルは部分的な出力のみ保存されています

## 各ファイルの概要

### basic_process.txt
標準的なプロセス操作、パイプ、環境変数の設定例

### fork_exec.txt
forkシステムコールによる子プロセスの生成とexecによるプログラムの置き換え

### signal_handling.txt
シグナルハンドラの設定と子プロセスへのシグナル送信

### zombie_prevention.txt
ゾンビプロセスの発生と回収、SIGCHLDハンドラ、ダブルフォークテクニック

### async_process.txt
Tokioを使った非同期プロセス管理、並行実行、タイムアウト処理

### process_group.txt
プロセスグループの作成、セッションリーダー、デーモン化

### secure_spawn.txt
入力検証、権限管理、環境変数のクリーンアップ、リソース制限