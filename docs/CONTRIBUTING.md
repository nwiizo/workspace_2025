# 開発ルール

個人的なワークスペースの開発ルールをまとめています。

## 作業環境

- ghqを使用してリポジトリを管理
- GitHub CLIを使用してGitHub操作を効率化

## 基本的な開発フロー

### 1. プロジェクトのセットアップ

```bash
# リポジトリのクローン
ghq get https://github.com/username/repo-name

# プロジェクトディレクトリへ移動
cd $(ghq root)/github.com/username/repo-name
```

### 2. 作業開始

```bash
# mainブランチを最新化
git checkout main
git pull origin main

# 作業ブランチを作成
gh issue create --title "イシューのタイトル" --body "イシューの詳細" # 必要に応じてイシューを作成
# イシューからブランチを作成
gh issue develop [イシュー番号]
# または
git checkout -b feature/branch-name
```

### 3. コミットルール

```bash
# 変更を記録
git add .
git commit -m "feat: 変更内容"
```

コミットメッセージの種類:
- `feat`: 新機能
- `fix`: バグ修正
- `docs`: ドキュメント更新
- `refactor`: リファクタリング
- `style`: コードスタイルの修正
- `test`: テストコード

### 4. Pull Request

```bash
# 変更をプッシュ
git push origin HEAD

# PRを作成
gh pr create
```

### 5. マージ後の後処理

```bash
# mainブランチに戻る
git checkout main
git pull origin main

# 作業ブランチを削除
git branch -d feature/branch-name
```

## 便利なコマンド

```bash
# リポジトリをブラウザで開く
gh browse

# PRの一覧を表示
gh pr list

# PRの詳細を確認
gh pr view [PR番号]

# イシューの一覧を表示
gh issue list
```
