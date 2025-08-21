# Container-Use (cu) 実践的な使い方とTips

## 🚀 セットアップ完了内容

1. **Docker環境**: Colima経由でDocker起動済み
2. **container-use**: v0.0.2 インストール済み
3. **Claude Code連携**: MCP経由で設定済み (`npx @anthropic-ai/claude-code mcp add container-use`)

## 📋 基本的な使い方

### コマンド一覧
```bash
cu list          # 環境一覧表示
cu watch         # リアルタイムモニタリング
cu terminal      # コンテナ内にターミナル接続
cu log           # 環境のログ表示
cu merge         # 環境を現在のブランチにマージ
cu delete        # 環境削除
```

## 💡 実践的なTips

### 1. リアルタイムモニタリング
```bash
# tmuxでバックグラウンド実行
tmux new-session -d -s cu-watch 'cu watch'
tmux attach -t cu-watch  # 監視画面を見る
```

### 2. 並列開発環境
複数のAIエージェントや開発タスクを同時実行:
- 各環境は独立したコンテナとGitブランチを持つ
- 競合なしに並列作業が可能
- 失敗してもメインブランチに影響なし

### 3. Claude Codeとの連携
Claude Codeから直接container-use環境を操作可能:
- 新しいセッションごとに自動的にコンテナ環境を作成
- 全ての操作がログに記録される
- Gitブランチとして変更を追跡

### 4. 安全な実験環境
```bash
# 実験的な変更を試す
git checkout -b experiment/risky-change
# container-useが自動的にこのブランチ用のコンテナを作成

# 成功したらマージ
cu merge experiment/risky-change

# 失敗したら削除
cu delete experiment/risky-change
```

### 5. デバッグとトラブルシューティング
```bash
# 環境の詳細ログ確認
cu log <environment-name>

# コンテナに直接アクセス
cu terminal <environment-name>

# Git履歴で変更を確認
git log --patch container-use/<environment-name>
```

## 🔧 高度な使い方

### 複数エージェントの協調作業
1. **フロントエンド専門エージェント**: UI開発に特化
2. **バックエンド専門エージェント**: API開発に特化
3. **テスト専門エージェント**: テスト作成に特化

各エージェントが独立したコンテナで作業し、最後に統合。

### パフォーマンステスト
並列で異なるアプローチを試して比較:
```bash
# アプローチA: 環境1で実装
# アプローチB: 環境2で実装
# 両方の結果を比較してベストな方を採用
```

## ⚠️ 注意点

1. **Docker必須**: コンテナ実行にはDockerが必要
2. **ディスク容量**: 各環境がコンテナイメージを持つため容量に注意
3. **ブランチ管理**: 不要になった環境は`cu delete`で削除推奨

## 🎯 ベストプラクティス

1. **命名規則**: 環境名は目的を明確に（例: `feature-auth`, `bugfix-api`）
2. **定期的なクリーンアップ**: 使用済み環境は削除
3. **ログの活用**: `cu watch`で全ての操作を監視
4. **早めのマージ**: 成功した変更は早めにマージ

## 📊 作成したテストスクリプト

1. **test-cu.py**: Python環境テスト
2. **parallel-test.sh**: 並列環境セットアップ
3. **test-env1.py, test-env2.js, test-env3.sh**: 並列実行デモ

これらのスクリプトで container-use の基本機能を検証可能。