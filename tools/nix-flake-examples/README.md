# Nix Flake Examples

このディレクトリには、ブログ記事「Nix Flake で開発環境を統一する」の検証用サンプルが含まれています。

## ディレクトリ構成

```
nix-flake-examples/
├── rust-project/        # Rust 開発環境の例
│   ├── flake.nix
│   ├── flake.lock
│   └── .envrc
├── go-project/          # Go 開発環境の例
│   ├── flake.nix
│   ├── flake.lock
│   └── .envrc
├── python-project/      # Python 開発環境の例
│   ├── flake.nix
│   ├── flake.lock
│   └── .envrc
├── nodejs-project/      # Node.js 開発環境の例
│   ├── flake.nix
│   ├── flake.lock
│   └── .envrc
├── react-project/       # React + Vite 開発環境の例
│   ├── flake.nix
│   ├── flake.lock
│   └── .envrc
├── multi-language/      # マルチ言語開発環境の例
│   ├── flake.nix
│   ├── flake.lock
│   └── .envrc
├── overlay-example/     # カスタム overlay の例
│   ├── flake.nix
│   ├── flake.lock
│   └── .envrc
├── docker-integration/  # Docker イメージビルドの例
│   ├── flake.nix
│   ├── flake.lock
│   └── .envrc
└── ci-cd-example/       # CI/CD 統合の例
    └── .github/workflows/ci.yml
```

## 使い方

### 前提条件

1. Nix がインストールされていること
2. Flakes が有効化されていること

```bash
# ~/.config/nix/nix.conf
experimental-features = nix-command flakes
```

### 各プロジェクトの実行

```bash
# Rust 環境に入る
cd rust-project
nix develop

# Go 環境に入る
cd go-project
nix develop

# Python 環境に入る
cd python-project
nix develop

# Node.js 環境に入る
cd nodejs-project
nix develop

# React + Vite 環境に入る
cd react-project
nix develop

# マルチ言語環境に入る
cd multi-language
nix develop          # 全言語
nix develop .#rust   # Rust のみ
nix develop .#go     # Go のみ
```

### direnv との連携（推奨）

全てのプロジェクトに `.envrc` ファイルが含まれています。direnv + nix-direnv を使うと、`cd` するだけで自動的に開発環境がロードされます。

**セットアップ（初回のみ）**

```bash
# 1. nix-direnv のインストール
nix profile install nixpkgs#nix-direnv

# 2. ~/.config/direnv/direnvrc を作成
mkdir -p ~/.config/direnv
cat > ~/.config/direnv/direnvrc << 'EOF'
if [ -f "$HOME/.nix-profile/share/nix-direnv/direnvrc" ]; then
  source "$HOME/.nix-profile/share/nix-direnv/direnvrc"
fi
EOF

# 3. シェルへのフック追加（.bashrc, .zshrc, config.fish など）
# bash: eval "$(direnv hook bash)"
# zsh:  eval "$(direnv hook zsh)"
# fish: direnv hook fish | source
```

**使い方**

```bash
# プロジェクトに移動して許可
cd rust-project
direnv allow

# 以降は cd するだけで自動的に環境がロード
# 環境変数、PATH が自動的に設定される
```

**キャッシュについて**

nix-direnv は `.direnv/` ディレクトリにキャッシュを作成します。2回目以降のロードは数ミリ秒で完了します。

```
.direnv/
├── flake-inputs/           # Flake 入力のキャッシュ
├── flake-profile-*         # Nix Store へのシンボリックリンク
└── flake-profile-*.rc      # 環境変数のキャッシュ
```

**注意**: `.direnv/` は `.gitignore` に追加することを推奨します。

### Docker イメージのビルド

```bash
cd docker-integration

# シンプルなイメージ
nix build .#docker-simple
docker load < result

# レイヤード イメージ（効率的なキャッシュ）
nix build .#docker-layered
docker load < result

# 最小限のイメージ
nix build .#docker-minimal
docker load < result
```

## 各サンプルの特徴

### rust-project

- rust-overlay による toolchain 管理
- cargo-watch, cargo-edit などの開発ツール
- treefmt による自動フォーマット

### go-project

- Go（最新安定版）
- golangci-lint, gopls, delve
- gotestsum によるテスト結果の可視化

### python-project

- Python 3.12
- uv による高速パッケージ管理
- ruff, pyright による品質管理

### nodejs-project

- Node.js 22
- corepack による pnpm/yarn 管理
- TypeScript サポート

### react-project

- React + Vite 開発環境
- Node.js 22 + corepack (pnpm/yarn)
- TypeScript + TypeScript LSP
- Prettier による自動フォーマット
- 2つの devShell:
  - `default`: 既存プロジェクト用の開発環境
  - `init`: 新規 Vite プロジェクトの作成用

```bash
# 新規プロジェクト作成
cd react-project
nix develop .#init  # npm create vite を自動実行

# 既存プロジェクトで開発
nix develop         # 開発ツールのみ
```

### multi-language

- 複数の devShell を一つの flake.nix で管理
- 言語ごとの環境を選択可能

### overlay-example

- カスタムパッケージの作成例
- 既存パッケージのラップ例

### docker-integration

- Nix で Docker イメージをビルド
- レイヤード イメージによる効率的なキャッシュ
- Distroless スタイルの最小イメージ

### ci-cd-example

- GitHub Actions での Nix 使用例
- DeterminateSystems の nix-installer
- Magic Nix Cache によるビルド高速化
