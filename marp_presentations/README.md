# Marp プレゼンテーション

## 概要

このディレクトリには、[Marp](https://marp.app/)（Markdown Presentation Ecosystem）を使用してプレゼンテーションを作成するためのテンプレートとリソースが含まれています。

## ディレクトリ構造

```
marp_presentations/
├── README.md                      # このファイル
├── sample-presentation.md         # 基本的なサンプルプレゼンテーション
├── title-slide.md                 # タイトル背景画像を使用したプレゼンテーション
├── 3shake-presentation.md         # 3-SHAKE用プレゼンテーション（背景画像スタイル）
├── 3shake-standard-slides.md      # 3-SHAKE用標準スタイルプレゼンテーション
├── assets/                        # 画像などのリソースを格納するディレクトリ
│   └── images/                    # 画像ファイル用
│       ├── title-background.jpg   # タイトル背景用画像
│       ├── 3shake-background.png  # 3-SHAKE用背景画像
│       ├── 3shake-background-full.png # 3-SHAKE用フル背景画像
│       ├── 3shake-logo.png        # 3-SHAKEロゴ
│       └── 3shake-cover.png       # 3-SHAKEカバー画像
├── templates/                     # テンプレートファイル
│   ├── background-template.md     # 背景画像を使用したテンプレート
│   ├── 3shake-logo-template.md    # 3-SHAKEロゴを使用したテンプレート
│   └── 3shake-standard-template.md # 3-SHAKE標準スタイルテンプレート
└── themes/                        # カスタムCSSテーマ
    ├── custom.css                 # 基本的なカスタムテーマ
    ├── title-theme.css            # タイトル画像用のテーマ
    ├── 3shake-theme.css           # 3-SHAKE用カスタムテーマ
    └── 3shake-standard-theme.css  # 3-SHAKE標準スライドスタイル用テーマ
```

## プレゼンテーションスタイル

### 1. 基本スタイル
シンプルなMarkdownスタイルのプレゼンテーション。

### 2. タイトル背景画像スタイル
タイトルスライドに大きな背景画像を使用したスタイル。

### 3. 3-SHAKE背景画像スタイル
3-SHAKEブランドの背景画像とカラースキームを使用した豪華なスタイル。

### 4. 3-SHAKE標準スタイル
青いタイトルバーと黄色の下線、各スライドの左下にページ番号と同じ高さに小さく3-SHAKEロゴが自動的に配置される公式プレゼンテーションスタイル。

## 始め方

### 1. インストール

Marpを使用するには、次のいずれかの方法でインストールします：

#### VSCode拡張機能（推奨）

1. [VSCode](https://code.visualstudio.com/)をインストール
2. [Marp for VS Code](https://marketplace.visualstudio.com/items?itemName=marp-team.marp-vscode)拡張機能をインストール

#### Marp CLI

```bash
npm install -g @marp-team/marp-cli
```

### 2. プレゼンテーションの作成

1. `.md`ファイルを作成し、先頭に次のYAML Front Matterを追加します：

```markdown
---
marp: true
theme: default  # または custom、title-theme、3shake-theme、3shake-standard-theme
paginate: true
---

# プレゼンテーションタイトル

コンテンツはここに
```

2. `---`（ハイフン3つ）でスライドを区切ります

### 3. 3-SHAKE標準スタイルの使用

3-SHAKE標準スタイルを使用するには：

```markdown
---
marp: true
theme: ./themes/3shake-standard-theme.css
paginate: true
style: |
  :root {
    --logo-url: url("./assets/images/3shake-logo.png");
  }
---

# スライドのタイトル

* 箇条書き項目
  * サブ項目
```

各スライドに青いタイトルバーと黄色の下線が適用され、すべてのスライドの左下にページ番号と同じ高さに小さな3-SHAKEロゴが自動的に表示されます。画像パスを正しく設定するために、`style`セクションで`--logo-url`変数を設定することが重要です。

### 4. 背景画像とロゴの使用

#### 背景画像の使用

```markdown
<!-- 
_backgroundColor: black  # または #0a1929（3-SHAKE用）
_color: white
-->

![bg](assets/images/3shake-background.png)

# タイトルテキスト
```

#### 独自のロゴを追加（標準のロゴ以外に追加したい場合）

```markdown
<div style="position: absolute; right: 30px; top: 20px;">
<img src="assets/images/3shake-logo.png" width="100px">
</div>
```

#### カバー画像の使用

```markdown
![bg right:40%](assets/images/3shake-cover.png)
```

テンプレートから始めたい場合は、以下のいずれかをコピーして使用してください：
- `templates/background-template.md` - シンプルな背景画像テンプレート
- `templates/3shake-logo-template.md` - ロゴと背景画像を使用した3-SHAKE用テンプレート
- `templates/3shake-standard-template.md` - 3-SHAKE標準スタイルテンプレート（左下に自動的にロゴ配置）

### 5. 3-SHAKEカラースキームの使用

3-SHAKEのブランドカラーをハイライトするには：

```markdown
# プレゼンテーション<span class="highlight-yellow">タイトル</span>

* <span class="highlight-blue">青色テキスト</span>
* <span class="highlight-green">緑色テキスト</span>
* <span class="highlight-yellow">黄色テキスト</span>

<div class="info-box">
情報ボックス
</div>
```

### 6. プレゼンテーションのエクスポート

#### VSCodeを使用する場合

1. コマンドパレットを開く (`Ctrl+Shift+P` または `Cmd+Shift+P`)
2. `Marp: Export slide deck...`を選択
3. 出力形式（HTML, PDF, PPTXなど）を選択

#### Marp CLIを使用する場合

```bash
# HTMLとして出力
marp --html 3shake-standard-slides.md

# PDFとして出力
marp --pdf 3shake-standard-slides.md

# PowerPointとして出力
marp --pptx 3shake-standard-slides.md
```

## カスタムテーマの適用

作成したカスタムテーマを使用するには、YAMLフロントマターで指定します：

```markdown
---
marp: true
theme: ./themes/3shake-standard-theme.css
---
```

## 参考リンク

- [Marp公式サイト](https://marp.app/)
- [Marp CLI ドキュメント](https://github.com/marp-team/marp-cli)
- [Marpit マークダウン記法](https://marpit.marp.app/markdown) 