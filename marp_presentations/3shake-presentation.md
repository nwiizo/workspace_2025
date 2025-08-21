---
marp: true
theme: ./themes/3shake-theme.css
paginate: true
style: |
  :root {
    --logo-url: url("./assets/images/3shake-cover.png");
    --mini-font-size: 20px;
    --header-footer-height: 50px;
    --black: #333;
  }
  /* 通常ページの左下にロゴを表示（より左下に押し込む） */
  section:not(.title)::before {
    content: "";
    position: absolute;
    left: 15px;  /* より左に */
    bottom: 15px;  /* より下に */
    width: 60px !important;  /* ロゴサイズを調整 */
    height: 60px !important;  /* ロゴサイズを調整 */
    background-image: var(--logo-url);
    background-size: contain;
    background-repeat: no-repeat;
    background-position: center;
    opacity: 0.9;
    z-index: 100;
  }
  /* 背景色付きスライドではロゴ色を反転 */
  section[data-background-color="dark"]::before {
    filter: brightness(0) invert(1);
  }
  /* タイトルページのページ番号を非表示 */
  section.title::after {
    display: none;
  }
  /* タイトルスライドのロゴスタイル */
  .title-logo {
    position: fixed;
    top: 5px;
    left: 5px;
    width: 240px !important;
    height: auto;
    z-index: 9999;
  }
  /* すべての画像サイズを上書き - 3shake-logo.pngのみに適用 */
  img[src*="3shake-logo.png"] {
    max-width: 240px !important;
    width: 240px !important;
  }
  /* タイトルとサブタイトルのサイズ調整 */
  .title h1 {
    font-size: 3.2em !important;
    margin-bottom: 0.1em !important;
  }
  .title h3 {
    font-size: 1.5em !important;
    margin-top: 0.1em !important;
  }
  /* 作者情報のスタイル */
  .author-info {
    position: absolute !important;
    bottom: 40px !important;
    left: 50px !important;
    padding-left: 0 !important;
    text-indent: 0 !important;
    font-size: 1.2em !important;
    color: white !important;
    font-weight: bold !important;
  }
  /* スライドタイトル（h2）のスタイル */
  section h2 {
    font-size: 1.8em !important;
    margin-top: -25px !important;
    padding-top: 0 !important;
    margin-bottom: 10px !important;
    color: black !important;
    border-bottom: 1px solid #dadce0 !important;
  }
  /* コンテンツエリアの上部マージンを調整 */
  section > *:not(h2):not(header):not(footer) {
    margin-top: 1.2em !important;
  }
  /* 引用（参考文献）のスタイル */
  blockquote {
    border-top: 0.1em dashed var(--black);
    font-size: var(--mini-font-size);
    width: 100%;
    position: absolute;
    bottom: var(--header-footer-height);
    left: 0;
    padding: 10px 20px;
    margin: 0;
    box-sizing: border-box;
  }
---

<!-- 
_backgroundColor: #0a1929
_color: white
_class: title dark
-->

![bg](assets/images/3shake-background-full.png)

<img src="./assets/images/3shake-logo.png" alt="3-SHAKE logo" style="position: absolute !important; top: 5px !important; left: 5px !important; width: 240px !important; height: auto !important; z-index: 9999 !important;">

<div class="title" style="text-align: left; margin-top: 180px; margin-left: 50px; padding-left: 0;">

# 3-shake テンプレート<br>タイトル<span class="highlight-yellow">入力</span>

### サブタイトルを入力

</div>

<div class="author-info" style="text-align: left; padding-left: 0; text-indent: 0;">
2025/5/10 イベント名@nwiizo  
</div>

---

<!-- _backgroundColor: white -->

![bg](assets/images/3shake-background-full.png)

## アジェンダ

<div class="info-box">
このプレゼンテーションでは、DX推進のための具体的な提案と実行計画について説明します。
</div>

* <span class="highlight-blue">**プロジェクト背景と課題**</span> - 現状分析と改善ポイント
* <span class="highlight-green">**提案内容**</span> - ソリューションと技術的アプローチ
* <span class="highlight-yellow">**期待される効果**</span> - KPIと成功指標
* <span class="highlight-blue">**実装計画**</span> - タイムラインとリソース計画
* **まとめ** - 次のステップと推奨アクション

---

<!-- _backgroundColor: white -->
![bg left:45%](./assets/images/nwiizo_icon.jpg)
## nwiizo

<div class="info-box">
株式会社スリーシェイクで</br>ソフトウェアエンジニアをやっています。</br>
システムの開発、運用、最適化</br>
読書、グラビア、格闘技が好きです。
</div>

---

<!-- _backgroundColor: white -->

## <span class="highlight-blue">プロジェクト背景と課題</span>

<div style="display: grid; grid-template-columns: 1fr 1fr; gap: 40px;">
<div>

### 現状分析

* 従来の業務プロセスによる非効率性
* 部門間のデータ連携不足
* リアルタイム分析の欠如
* カスタマーエクスペリエンスの改善余地

</div>
<div>

### 解決すべき課題

* ✅ プロセス自動化による効率化
* ✅ 統合データプラットフォームの構築
* ✅ リアルタイム分析基盤の導入
* ✅ カスタマージャーニーの最適化

</div>
</div>

---

<!-- _backgroundColor: white -->

## <span class="highlight-green">提案内容: ソリューション概要</span>

<div style="display: grid; grid-template-columns: 6fr 4fr; gap: 40px; align-items: center;">
<div>

1. **クラウドネイティブアーキテクチャ**
   * スケーラブルで柔軟なインフラストラクチャ
   * マイクロサービスベースの設計$^{[1]}$


2. **データ統合プラットフォーム**
   * リアルタイムデータ処理/高度な分析・予測機能
   * 業務プロセスの自動化

  </div>
  </div>

> [1] https://example.com


---

<!-- _backgroundColor: white -->

## <span class="highlight-green">提案内容: バックエンド技術スタック</span>

<div style="margin: 0 auto; max-width: 80%;">

### インフラストラクチャ & バックエンド技術

* **クラウド基盤**: AWS / GCP
* **コンテナ化**: Kubernetes / Docker
* **サーバレス**: AWS Lambda / Cloud Functions
* **データベース**: Amazon Aurora / Cloud Spanner
* **キャッシュ**: Redis / Memcached
* **メッセージング**: Kafka / RabbitMQ
* **API管理**: Kong / Apigee
</div>
<div class="info-box">
クラウドネイティブ技術を活用し、スケーラブルかつ堅牢なバックエンド基盤を構築します。
</div>

---

<!-- _backgroundColor: white -->

## <span class="highlight-green">提案内容: フロントエンド/分析技術</span>

<div style="margin: 0 auto; max-width: 80%;">

### ユーザーインターフェース & 分析技術

* **UI/UX フレームワーク**: React / Vue.js
* **モバイルアプリ**: React Native / Flutter 
* **データ可視化**: D3.js / Tableau
* **データ分析基盤**: BigQuery / Redshift
* **レポーティング**: Looker / Power BI
* **リアルタイム分析**: Apache Spark / Flink

</div>

<div class="info-box">
直感的なユーザーインターフェースと高度なデータ分析機能を組み合わせ、最適なユーザー体験を提供します。
</div>

---

<!-- _backgroundColor: white -->

## <span class="highlight-green">提案内容: AI・モニタリング技術</span>

<div style="margin: 0 auto; max-width: 80%;">

### AI & モニタリング技術

* **AI/ML**: TensorFlow / PyTorch$^{[2]}$
* **予測分析**: Prophet / Auto ML
* **システム監視**: Prometheus / Grafana$^{[3]}$
* **ログ分析**: ELK Stack / Splunk
* **アラート**: PagerDuty / OpsGenie
* **自動修復**: Kubernetes Operators

</div>

<div class="info-box">
AIを活用した予測分析と堅牢なモニタリング体制により、システムの安定性と先進性を両立します。
</div>

> [1] https://www.tensorflow.org/  
> [2] https://grafana.com/

---

<!-- _backgroundColor: white -->

## <span class="highlight-yellow">期待される効果: KPI目標</span>

| KPI指標 | 現状値 | 目標値 | 改善幅 | 達成期間 |
|:-----|:----:|:----:|:----:|:----:|
| 業務効率化 | 67% | 95% | **+28%** | 3ヶ月 |
| 処理時間短縮 | 24分 | 3分 | **-21分** | 6ヶ月 |
| エラー率削減 | 5.2% | 0.5% | **-4.7%** | 6ヶ月 |
| 顧客満足度 | 72% | 92% | **+20%** | 12ヶ月 |
| ROI | - | 320% | - | 24ヶ月 |

<div class="info-box">
投資回収期間（ROI）は約8ヶ月を想定し、2年間で約320%のROI達成を見込んでいます。
</div>

---

<!-- _backgroundColor: white -->

## <span class="highlight-blue">実装計画: タイムライン</span>

```mermaid
gantt
    title プロジェクト実装タイムライン
    dateFormat  YYYY-MM-DD
    section フェーズ1
    要件定義・設計      :2025-06-01, 30d
    インフラ構築        :2025-06-15, 30d
    section フェーズ2
    コア機能開発        :2025-07-15, 45d
    テスト・検証        :2025-08-15, 30d
    section フェーズ3
    パイロット導入      :2025-09-15, 45d
    改善・最適化        :2025-10-15, 30d
    section フェーズ4
    本番展開            :2025-11-15, 30d
    モニタリング・支援  :2025-12-01, 90d
```

---

<!-- _backgroundColor: white -->

## <span class="highlight-blue">実装計画: リソース配分</span>

<div style="display: grid; grid-template-columns: 1fr 1fr; gap: 40px;">
<div>

### 人的リソース

* プロジェクトマネージャー (1名)
* ソリューションアーキテクト (2名)
* バックエンドエンジニア (3名)
* フロントエンドエンジニア (2名)
* データサイエンティスト (1名)
* QAエンジニア (2名)

</div>
<div>

---
## <span class="highlight-blue">実装計画: リソース配分</span>
### 主要マイルストーン

* **M1**: 要件定義完了 (2025年6月末)
* **M2**: システム基盤構築 (2025年7月末)
* **M3**: コア機能実装 (2025年9月初)
* **M4**: パイロット開始 (2025年9月末)
* **M5**: 本番リリース (2025年11月末)
* **M6**: 安定運用開始 (2026年2月末)

</div>
</div>

---

<!-- _backgroundColor: white -->

## まとめ: 期待される事業インパクト

<div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 20px; text-align: center;">
<div>

### 💼 ビジネス効率化

* プロセス自動化による業務効率向上
* 人的リソースの最適な再配置

</div>
<div>

### 📊 データ活用

* リアルタイムインサイト獲得
* データドリブン意思決定

</div>
<div>

### 🚀 顧客体験向上

* パーソナライズされた体験提供
* 顧客満足度・ロイヤリティ向上

</div>
</div>

<div class="info-box" style="margin-top: 20px;">
このプロジェクトは単なるシステム導入ではなく、組織全体のデジタル変革を実現する戦略的取り組みです。
</div>

---

<!-- 
_backgroundColor: #0a1929
_color: white
_class: title dark
-->

![bg](assets/images/3shake-background-full.png)

<!-- タイトルページ左上に大きなロゴを表示 -->
<div style="position: absolute !important; top: 5px !important; left: 5px !important; z-index: 9999 !important; margin: 0 !important; padding: 0 !important;">
  <img src="./assets/images/3shake-logo.png" style="width: 240px !important; height: auto !important; display: block !important;">
</div>

<div style="text-align: center; margin-top: 200px;">

# ありがとう<span class="highlight-yellow">ございました</span>

### ご質問・ご相談はお気軽にお問い合わせください

@3shake | https://3-shake.com
</div>