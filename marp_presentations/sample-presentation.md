---
marp: true
theme: default
paginate: true
---

# サンプルプレゼンテーション

作成者: あなたの名前

---

## プレゼンテーションの概要

* Marpの紹介
* 基本的な使い方
* スタイリングとテーマ
* 便利な機能

---

## Marpとは？

Marp（Markdown Presentation Ecosystem）は、Markdownで簡単にプレゼンテーションを作成できるツールです。

* シンプルなMarkdown記法
* HTMLやPDFに変換可能
* CSSでカスタマイズ可能

![h:200](https://marp.app/og-image.png)

---

## 基本的な使い方

```markdown
# スライドのタイトル

コンテンツはMarkdownで記述します

---

# 新しいスライド

区切りは「---」で行います
```

---

## スタイリングとテーマ

テーマは `theme` ディレクティブで指定できます。

```css
/* themes/custom.css */
@import 'default';

section {
  background: #f5f5f5;
  font-family: 'Helvetica', sans-serif;
}
```

---

## 画像の配置

画像は `assets/images` ディレクトリに保存すると良いでしょう。

```markdown
![width:500px](assets/images/example.jpg)
```

---

## 便利な機能

* 数式のサポート: $E=mc^2$
* コードのシンタックスハイライト
* スライドの背景画像

---

# ありがとうございました！

詳細は [Marp公式サイト](https://marp.app/) をご覧ください。 