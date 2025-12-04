# RustでOWASP API Security Top 10を体験する（前編）：認証・認可の基礎とデータ保護

<span style="font-size: 125%">この記事は、[Rust Advent Calendar 2025](https://qiita.com/advent-calendar/2025/rust) **5日目**のエントリ記事です。</span>

## はじめに

先日、あるプロジェクトのコードレビューで「このエンドポイント、認証は通ってるけど認可は大丈夫か」と聞いたら、「認証してるから大丈夫でしょ」という返答が返ってきた。

その瞬間、私の脳内では警報が鳴り響いた。これはあれだ。「鍵がかかってるから金庫は安全」と言いながら、金庫の中身を誰でも見られる状態にしているやつだ。

認証（Authentication）と認可（Authorization）の違い。頭ではわかっていても、実際のコードでどう違うのか、どう危険なのかを体感したことがある人は意外と少ない。かくいう私も、セキュリティの本を読んで「ふーん」と思いながら、翌日には同じミスをやらかしていた口だ。

そこで今回、OWASP API Security Top 10の脆弱性を**実際に攻撃できる形**でRustにより実装してみた。「脆弱なエンドポイント」と「安全なエンドポイント」を並べて、攻撃がどう成功し、どう防げるのかを手を動かして確認できる。

## なぜBobとAliceなのか

セキュリティの例でやたらと「BobがAliceのデータを〜」という話が出てくる。なぜこの2人なのか。

これは1978年にRon Rivest、Adi Shamir、Leonard Adleman（RSA暗号のRSA）が書いた論文「A Method for Obtaining Digital Signatures and Public-Key Cryptosystems」に由来する。彼らは暗号通信の説明に「AさんがBさんにメッセージを送る」ではなく、「AliceがBobにメッセージを送る」と書いた。AとBで始まる名前を選んだだけだが、これが定着した。

その後、セキュリティの世界では登場人物が増えていった。

- **Alice & Bob**: 通信したい善良な2人（主人公）
- **Eve**: 盗聴者（Eavesdropperから。悪役その1）
- **Mallory**: 能動的攻撃者（Maliciousから。もっと悪い悪役）
- **Trent**: 信頼できる第三者（Trustedから）
- **Carol/Charlie**: 3人目の参加者が必要なとき

つまり、BobとAliceは何十年も同じ役を演じ続けている。

本記事でも、この伝統に従ってBobとAliceに登場してもらう。Bobには悪役を演じてもらうことになるが、本来のBobは悪い人ではない。「認可が不十分だと善良なBobでも悪いことができてしまう」というのが本質的な問題なのだ。

[https://en.wikipedia.org/wiki/Alice_and_Bob:embed:cite]


## なぜ「体験」が必要なのか

セキュリティの勉強で一番難しいのは、「危険性を実感すること」だ。

ドキュメントを読んで「BOLAは危険です」と書いてあっても、「へー、そうなんだ」で終わる。これは人間の性だ。交通事故のニュースを見ても「自分は大丈夫」と考えるのと同じで、実際にBobがAliceのデータを抜き取る瞬間を見ないと、その怖さは伝わらない。

このデモを作った動機は単純で、**自分が「あ、これ確かにヤバい」と冷や汗をかける教材が欲しかった**からだ。本を読んで「なるほど」と思っても、3日後には忘れている。でも、自分の手で攻撃を成功させた経験は忘れない。

ちなみに、このデモを作っている最中に「あれ、これ本番のコードにも似たようなのあったな...」と気づいて本当に冷や汗をかいた。勉強は大事。

## OWASP API Security Top 10 (2023) 一覧

まず、OWASP API Security Top 10の全体像を把握しておこう。本記事では、このうち主要な脆弱性を実際にRustで実装して体験する。

https://owasp.org/API-Security/editions/2023/en/0x11-t10/

[https://owasp.org/API-Security/editions/2023/en/0x11-t10/:embed:cite]

| リスク | 説明 |
|--------|------|
| **API1:2023 - Broken Object Level Authorization** | APIはオブジェクト識別子を扱うエンドポイントを公開しがちで、オブジェクトレベルのアクセス制御の問題が広い攻撃対象となる。ユーザーからのIDを使ってデータソースにアクセスするすべての関数で、オブジェクトレベルの認可チェックを考慮すべき。 |
| **API2:2023 - Broken Authentication** | 認証メカニズムは不正に実装されることが多く、攻撃者が認証トークンを侵害したり、実装の欠陥を悪用して一時的または永続的に他のユーザーになりすますことを可能にする。 |
| **API3:2023 - Broken Object Property Level Authorization** | このカテゴリはAPI3:2019の過度なデータ露出とAPI6:2019のMass Assignmentを統合し、根本原因であるオブジェクトプロパティレベルでの認可検証の欠如または不適切さに焦点を当てている。 |
| **API4:2023 - Unrestricted Resource Consumption** | APIリクエストの処理にはネットワーク帯域、CPU、メモリ、ストレージなどのリソースが必要。成功した攻撃はサービス拒否や運用コストの増加につながる可能性がある。 |
| **API5:2023 - Broken Function Level Authorization** | 異なる階層、グループ、ロールを持つ複雑なアクセス制御ポリシーと、管理機能と通常機能の不明確な分離は、認可の欠陥につながりやすい。 |
| **API6:2023 - Unrestricted Access to Sensitive Business Flows** | このリスクに脆弱なAPIは、自動化された方法で過度に使用された場合にビジネスを損なう可能性のある機能を補償せずにビジネスフローを公開している。 |
| **API7:2023 - Server Side Request Forgery** | SSRFの欠陥は、APIがユーザー提供のURIを検証せずにリモートリソースを取得する際に発生する可能性がある。ファイアウォールやVPNで保護されていても、攻撃者がアプリケーションに細工されたリクエストを予期しない宛先に送信させることができる。 |
| **API8:2023 - Security Misconfiguration** | APIとそれをサポートするシステムには通常、APIをよりカスタマイズ可能にするための複雑な構成が含まれている。ソフトウェアおよびDevOpsエンジニアがこれらの構成を見落としたり、セキュリティのベストプラクティスに従わない場合がある。 |
| **API9:2023 - Improper Inventory Management** | APIは従来のWebアプリケーションよりも多くのエンドポイントを公開する傾向があり、適切で更新されたドキュメントが非常に重要。非推奨のAPIバージョンや公開されたデバッグエンドポイントなどの問題を軽減するために、ホストとデプロイされたAPIバージョンの適切なインベントリも重要。 |
| **API10:2023 - Unsafe Consumption of APIs** | 開発者はサードパーティAPIから受信したデータをユーザー入力よりも信頼する傾向があり、より弱いセキュリティ基準を採用しがち。APIを侵害するために、攻撃者はターゲットAPIを直接侵害しようとするのではなく、統合されたサードパーティサービスを狙う。 |

本記事で実際に体験できる脆弱性は以下である。
- **前編（本記事）**: API1 (BOLA), API2 (Broken Authentication), API3 (Mass Assignment)
- **後編**: API4 (Rate Limit), API5 (BFLA), API7 (SSRF)

## デモの全体像

このデモは9つのバイナリで構成されている。それぞれが独立したWebサーバーとして起動する。

1. `/token/{user_id}` でテスト用JWTを取得
2. `/vulnerable/...` で脆弱なエンドポイントを叩く
3. `/...` で安全なエンドポイントを叩く

```
api-security-demo/
├── src/bin/
│   ├── bola.rs              # BOLA: オブジェクトレベル認可の不備
│   ├── bfla.rs              # BFLA: 機能レベル認可の不備
│   ├── mass_assignment.rs   # Mass Assignment: 一括代入の脆弱性
│   ├── broken_auth.rs       # Broken Auth: 認証の不備
│   ├── rate_limit.rs        # Rate Limit: リソース消費制限の不備
│   ├── ssrf.rs              # SSRF: サーバーサイドリクエストフォージェリ
│   ├── jwt.rs               # JWT: トークン操作のデモ
│   ├── observability.rs     # 攻撃検知システム
│   └── security_test.rs     # 自動セキュリティテスト
```

技術スタックはRust + axum。Rust 2024エディションで書いている。

### 前提条件

試してみたい方は以下が必要である。

- **Rust 1.85以上**（2024エディション対応）
- **curl** と **jq**（テスト用）

```bash
# リポジトリのクローン
git clone https://github.com/nwiizo/workspace_2025.git
cd workspace_2025/infrastructure/api-security-demo

# ビルド（初回は依存関係のダウンロードで時間がかかる）
cargo build --release
```

## 実装アーキテクチャの詳細

「デモを動かす」だけでなく「なぜこう実装したのか」を理解することで、自分のプロジェクトに応用できる。ここでは設計判断とその理由を詳しく説明する。

### プロジェクト構成

```
api-security-demo/
├── Cargo.toml              # Rust 2024エディション、依存関係定義
├── src/
│   ├── lib.rs              # ライブラリのエントリポイント
│   ├── auth.rs             # JWT認証・認可ロジック
│   ├── db.rs               # SQLiteデータベース操作
│   ├── error.rs            # エラー型定義
│   ├── models.rs           # データモデル定義
│   └── bin/                # 各デモのバイナリ
│       ├── bola.rs
│       ├── bfla.rs
│       └── ...
└── scripts/
    └── test_all.sh         # 全テスト実行スクリプト
```

共通ロジックは`src/`配下にライブラリとして切り出し、各デモは`src/bin/`配下の独立したバイナリとして実装している。これにより以下のメリットがある。

1. **コードの再利用**: 認証、DB操作、エラーハンドリングを全デモで共有
2. **単一責任**: 各バイナリは1つの脆弱性カテゴリに集中
3. **独立した起動**: `cargo run --bin bola-demo`で特定のデモだけ起動可能

### エラーハンドリング設計

Rustらしいエラー設計を採用した。`thiserror`クレートで列挙型エラーを定義し、axumの`IntoResponse`を実装した。

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication required")]
    Unauthorized,

    #[error("Access denied: {0}")]
    Forbidden(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),

    #[error("Database error: {0}")]
    DatabaseError(String),
}
```

なぜ`anyhow::Error`ではなく独自のエラー型なのか。

- **HTTPステータスコードの制御**: エラーの種類によって401、403、404、429などを返し分けたい
- **クライアントへのメッセージ制御**: 内部エラーの詳細は隠し、適切なメッセージだけ返したい
- **コンパイル時の網羅性チェック**: `match`で全ケースを処理しているか確認できる

`IntoResponse`の実装は以下の通りである。

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".to_string()),
            // ...
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}
```

これにより、ハンドラ関数で`?`演算子を使うだけで適切なHTTPレスポンスに変換される。

### 認証・認可の実装パターン

axumの`FromRequestParts`トレイトを実装したExtractorを使う。これがこのデモの核心部分だ。

```rust
/// Extractor for authenticated user claims (secure version)
#[derive(Debug, Clone)]
pub struct AuthenticatedUser(pub UserClaims);

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let result = extract_auth_from_parts(parts, false);
        async move { result.map(AuthenticatedUser) }
    }
}
```

Extractorパターンの利点は以下の通りである。

1. **宣言的**: 関数シグネチャに`AuthenticatedUser`があれば認証必須と一目でわかる
2. **再利用可能**: 同じExtractorを全エンドポイントで使い回せる
3. **テスト容易**: Extractorを差し替えてテスト可能
4. **失敗時の自動レスポンス**: 認証失敗時は自動で401を返す

「脆弱な」バージョンも用意している。

```rust
/// Extractor for user claims WITHOUT proper validation (vulnerable version)
#[derive(Debug, Clone)]
pub struct VulnerableAuthUser(pub UserClaims);
```

これは署名検証をスキップし、期限切れトークンも受け入れる。教育目的のみ。

### データベース層の設計

SQLiteを使い、認可の有無でメソッドを分けている。

```rust
/// Get order by ID (no authorization check - vulnerable)
pub fn get_order_by_id(&self, id: i64) -> Result<Option<Order>, AppError> {
    let conn = self.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, user, product, quantity FROM orders WHERE id = ?1"
    )?;
    // ...
}

/// Get order by ID with user check (secure)
pub fn get_order_by_id_for_user(&self, id: i64, user: &str) -> Result<Option<Order>, AppError> {
    let conn = self.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, user, product, quantity FROM orders WHERE id = ?1 AND user = ?2"
    )?;
    // ...
}
```

「なぜSQLで認可するのか。アプリケーション層でフィルタすればいいのでは」という疑問もあるだろう。

アプリケーション層でも可能だが、DB層で認可する利点がある。

1. **パフォーマンス**: 不要なデータをDBから取得しない
2. **防御の多層化**: アプリ層のバグがあってもDB層で防げる
3. **一貫性**: SQLで認可ロジックが一箇所に集約される

しかし、複雑な認可ルール（「自分のチームのデータ」など）はアプリ層で実装したほうが保守しやすい場合もある。

### 依存関係の選定理由

`Cargo.toml`から主要な依存関係とその理由を説明する。

```toml
# Web framework
axum = { version = "0.8", features = ["macros"] }
```
**axum**: Tokioチームが開発、型安全、Extractorパターン。Actix-webより新しく、モダンな設計。

```toml
# Authentication & Authorization
jsonwebtoken = "9"
argon2 = "0.5"
```
**jsonwebtoken**: Rustで最もポピュラーなJWTライブラリ。
**argon2**: パスワードハッシュの現行推奨アルゴリズム。bcryptより新しく、メモリハード。

```toml
# Error handling
thiserror = "2"
```
**thiserror**: 派生マクロでボイラープレートを削減。`#[error("...")]`でDisplay実装が自動生成される。

```toml
# Rate limiting
governor = "0.8"
```
**governor**: トークンバケットアルゴリズムの実装。非同期対応。

```toml
# Database
rusqlite = { version = "0.32", features = ["bundled"] }
```
**rusqlite**: SQLiteバインディング。`bundled`でSQLiteを同梱（環境依存を排除）。本番ではPostgreSQLやMySQLを推奨。

### テスト戦略

各モジュールにユニットテストを配置している。

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_authorization() {
        let db = Database::new_in_memory().unwrap();
        let order = db.create_order("alice", "Test Product", 5).unwrap();

        // Alice can access her order
        let result = db.get_order_by_id_for_user(order.id, "alice").unwrap();
        assert!(result.is_some());

        // Bob cannot access Alice's order
        let result = db.get_order_by_id_for_user(order.id, "bob").unwrap();
        assert!(result.is_none());
    }
}
```

より、`scripts/test_all.sh`でE2E的な統合テストを実行。各エンドポイントに実際にHTTPリクエストを送り、脆弱なエンドポイントで攻撃が成功すること、安全なエンドポイントで攻撃が失敗することを検証する。

---

## API1: BOLA - 最も危険で、最も見落とされやすい脆弱性

OWASP API Security Top 10の堂々第1位がBOLA（Broken Object Level Authorization）だ。日本語では「オブジェクトレベル認可の不備」。

https://owasp.org/API-Security/editions/2023/en/0xa1-broken-object-level-authorization/

[https://owasp.org/API-Security/editions/2023/en/0xa1-broken-object-level-authorization/:embed:cite]

名前が難しそうに見えるが、中身は簡単だ。要するに**「BobがAliceのデータを見られてしまう」**という、小学生でも「それダメでしょ」とわかる問題だ。しかし、驚くほど多くの本番システムにこれがある。人類は学ばない。

### なぜBOLAが最も危険なのか

BOLAが1位である理由は明確だ。

1. **発生頻度が非常に高い** - ほぼすべてのAPIがリソースIDを扱う。そのすべてで認可チェックが必要
2. **自動化しやすい** - 攻撃者はIDを1, 2, 3...と順に試すだけ。スクリプト数行で全データを列挙できる
3. **検出が困難** - 正規のリクエストと見分けがつかない。WAFでは防げない
4. **影響が甚大** - 顧客データ、取引履歴、個人情報がすべて漏洩する可能性

### 実際のインシデント事例

BOLAによる情報漏洩は数え切れないほど発生している。

- **2019年 First American Financial** - 不動産取引記録8億8500万件が流出。URLのIDを変えるだけで他人の書類にアクセス可能だった
- **2018年 Facebook** - View As機能の脆弱性で5000万アカウントのトークンが漏洩
- **多数のモバイルアプリ** - APIエンドポイントのID推測で他ユーザーのプロフィールにアクセス可能

これらに共通するのは「認証はしていたが、認可が不十分だった」という点だ。ログインしているからといって、すべてのデータにアクセスできるわけではない。この当たり前のことを、コードで正しく実装するのは意外と難しい。

### なぜ開発者はBOLAを生み出してしまうのか

1. **認証と認可の混同** - 「ログインしてるからOK」という思い込み
2. **フレームワークの過信** - 「認証ミドルウェアを通ってるから安全」という誤解
3. **テストの盲点** - 機能テストは自分のデータでしか行わない
4. **IDの予測可能性** - 連番IDは攻撃を容易にする（でもUUIDでも根本解決にならない）
5. **開発速度優先** - 「認可は後で追加する」と言いながら忘れる

### 脆弱なコード

```rust
/// VULNERABLE: Returns any order by ID without checking ownership
async fn vulnerable_get_order(
    State(state): State<Arc<AppState>>,
    _user: AuthenticatedUser, // 認証情報を受け取っているが...
    Path(order_id): Path<i64>,
) -> Result<Json<Order>, AppError> {
    // 使っていない。アンダースコアプレフィックスがそれを物語っている
    let order = state.db.get_order_by_id(order_id)?
        .ok_or_else(|| AppError::NotFound(format!("Order {} not found", order_id)))?;

    Ok(Json(order))
}
```

`_user`としてわざわざ認証情報を受け取っているのに、**アンダースコアつけて無視している**。これは「セキュリティチェックしてますよ」というアリバイ作りにすらなっていない。むしろ「チェックしようとして忘れた」という証拠だ。

### 安全なコード

```rust
/// SECURE: Returns order only if it belongs to the authenticated user
async fn secure_get_order(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,  // アンダースコアなし
    Path(order_id): Path<i64>,
) -> Result<Json<Order>, AppError> {
    let user_id = &user.0.sub;

    // 「注文ID」と「ユーザーID」の両方でDBを検索
    let order = state.db.get_order_by_id_for_user(order_id, user_id)?
        .ok_or_else(|| AppError::NotFound(format!(
            "Order {} not found or access denied", order_id
        )))?;

    Ok(Json(order))
}
```

違いは1行だけ。たった1行。でも、この1行が「情報漏洩インシデント発生」と「平穏な運用」の分かれ道だ。

### 微妙な脆弱性：一見正しそうに見えるバグ

本番環境で見つかる脆弱性の多くは、明らかな間違いではない。「一見正しそうに見える」コードに潜んでいる。このデモには3つの「微妙な脆弱性」エンドポイントを用意した。

#### 微妙な脆弱性 #1: クエリパラメータによる上書き

```rust
#[derive(Deserialize)]
struct UserIdQuery {
    user_id: Option<String>,
}

/// 「デバッグ用にuser_idをクエリパラメータで指定できるようにしよう」
/// という親切心から生まれた脆弱性
async fn subtle_vulnerable_get_order(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,  // ちゃんと認証してる！
    Path(order_id): Path<i64>,
    Query(query): Query<UserIdQuery>,
) -> Result<Json<Order>, AppError> {
    // BUG: クエリパラメータが認証情報を上書きしてしまう
    let user_id = query.user_id.unwrap_or_else(|| user.0.sub.clone());

    let order = state
        .db
        .get_order_by_id_for_user(order_id, &user_id)?  // user_idが攻撃者の指定した値に！
        .ok_or_else(|| AppError::NotFound("..."))?;

    Ok(Json(order))
}
```

攻撃方法は以下の通りである。

```bash
# Bobとして認証
BOB_TOKEN=$(curl -s http://localhost:8080/token/bob | jq -r .access_token)

# クエリパラメータでAliceになりすまし
curl -H "Authorization: Bearer $BOB_TOKEN" \
     "http://localhost:8080/subtle/orders/1?user_id=alice"
```

このパターンは実際のコードレビューでよく見る。「管理画面でユーザーを切り替えて確認したい」「サポート担当がユーザーの代わりに操作する機能が必要」などの要件から生まれがち。対策は「そもそもこの機能は必要か」を問い直すことと、必要なら別の認証フローを用意すること。

#### 微妙な脆弱性 #2: TOCTOU（Time-of-Check-Time-of-Use）

```rust
async fn race_condition_get_order(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Path(order_id): Path<i64>,
) -> Result<Json<Order>, AppError> {
    let user_id = &user.0.sub;

    // Step 1: 注文を取得（全件から）
    let order = state.db.get_order_by_id(order_id)?
        .ok_or_else(|| AppError::NotFound(...))?;
    // ↑ この時点で機密データがメモリに載っている！

    // Step 2: 所有者をチェック
    if order.user != *user_id {
        // エラーメッセージが情報を漏らす
        return Err(AppError::Forbidden(format!(
            "Order {} belongs to another user",  // 存在することを教えてしまう
            order_id
        )));
    }

    Ok(Json(order))
}
```

何が問題なのか。

1. **データをフェッチしてから認可チェック**している。認可が通らなくても、データは既にメモリ上にある
2. **エラーメッセージが情報を漏らす**。「存在しない」と「アクセス権がない」が区別できる
3. **ログに所有者情報が残る**。認可失敗時のログに`order_owner = order.user`を出力している

正しい順序は「認可チェック → データフェッチ」だが、「IDだけでは認可チェックできない」という理由でこの順序になりがち。解決策はDB層で`get_order_by_id_for_user`のように、フェッチと認可を一体化すること。

#### 微妙な脆弱性 #3: 認可前のログ出力

```rust
async fn logging_before_auth_get_order(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Path(order_id): Path<i64>,
) -> Result<Json<Order>, AppError> {
    // 「監査のために全リクエストをログに残す」という要件から
    let order = state.db.get_order_by_id(order_id)?;

    // 認可チェック前に詳細をログ出力
    if let Some(ref o) = order {
        tracing::info!(
            order_id = o.id,
            order_user = o.user,       // 誰の注文かログに残る
            order_product = o.product, // 何を買ったかログに残る
            requester = user.0.sub,
            "Order access attempted"
        );
    }

    // ここで認可チェック（でも遅い）
    let order = order.ok_or_else(|| AppError::NotFound(...))?;
    if order.user != user.0.sub {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    Ok(Json(order))
}
```

ログは「セキュリティのために残す」という意図だが、認可前にログを取ると**攻撃者がアクセスできないデータがログに残る**。これは情報漏洩だ。ログ収集基盤に脆弱性があった場合、このログから機密情報が漏れる。

正しいパターンは以下である。

1. 認可前のログは「誰が」「何にアクセスしようとしたか（IDのみ）」
2. 認可後のログは詳細情報を含めてOK

### 実際に攻撃してみる

```bash
# サーバー起動
cargo run --release --bin bola-demo

# Bobのトークンを取得
BOB_TOKEN=$(curl -s http://localhost:8080/token/bob | jq -r .access_token)

# 脆弱なエンドポイント: BobがAliceの注文(ID=1)を取得
curl -H "Authorization: Bearer $BOB_TOKEN" \
     http://localhost:8080/vulnerable/orders/1
```

結果は以下の通りである。

```json
{
  "id": 1,
  "user": "alice",
  "product": "Widget A",
  "quantity": 5
}
```

**Bobが、Aliceの注文データを取得できてしまった。** Aliceは知らない。Bobは黙っている。システムは何も気づいていない。これが現実のインシデントだったら、ニュースになるやつだ。

安全なエンドポイントでは以下のようになる。
```bash
curl -H "Authorization: Bearer $BOB_TOKEN" \
     http://localhost:8080/orders/1
```

結果は以下の通りである。

```json
{
  "error": "Order 1 not found or access denied"
}
```

404を返している点もポイントだ。「なんで403（Forbidden）じゃないのか」という疑問があるだろう。

- 403は「その注文は存在するよ。しかしお前には見せない」という意味である
- 404は「何の話だ。そんな注文知らないが」という意味である

403は「存在する」という情報を漏らしている。攻撃者にヒントを与えないためには404のほうが適切だ。

---

## API2: Broken Authentication - JWT検証の問題

「署名さえ正しければOK」という誤解を打ち砕くデモ。

https://owasp.org/API-Security/editions/2023/en/0xa2-broken-authentication/

[https://owasp.org/API-Security/editions/2023/en/0xa2-broken-authentication/:embed:cite]

### なぜJWT検証で失敗するのか

JWTは「署名で改ざんを検出できる」という特性から、安全だと誤解されやすい。しかし、JWTのセキュリティは署名検証だけでは不十分だ。以下の検証が**すべて**必要である。

| 検証項目 | 何をチェックするか | 省略するとどうなるか |
|---------|--------------|-------------------|
| 署名 (`signature`) | トークンが改ざんされていないか | 偽造トークンが通る |
| 有効期限 (`exp`) | トークンが期限内か | 永久に使えるトークンが発生 |
| 発行者 (`iss`) | 正当な発行者が作ったか | 他システムのトークンが通る |
| オーディエンス (`aud`) | このAPIで使うべきか | 別サービスのトークンが通る |
| Not Before (`nbf`) | まだ使用開始前ではないか | 未来のトークンが先に使える |

### JWTに関する危険な誤解

1. **「署名が正しければ安全」** → 署名は「改ざんされていない」だけで「使っていい」は別の話
2. **「JWTライブラリを使えば安全」** → デフォルト設定が安全とは限らない
3. **「短い有効期限だから大丈夫」** → `exp`チェックを無効にしていたら意味がない
4. **「リフレッシュトークンで更新するから」** → 古いアクセストークンが使えたら問題

```bash
cargo run --release --bin broken-auth-demo
```

### 脆弱な実装：署名以外を検証しない

```rust
/// VULNERABLE: Validates JWT signature but skips claim validation
async fn vulnerable_validate_token(headers: HeaderMap) -> Result<Json<TokenValidationResponse>, AppError> {
    // ...

    // VULNERABLE: Disable all validation except signature
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = false; // 有効期限チェックしない！
    validation.validate_aud = false; // audience チェックしない！
    validation.required_spec_claims.clear(); // 必須クレームなし！

    let result = decode::<UserClaims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &validation,
    );
    // ...
}
```

これが危険な理由：
- **期限切れトークン**が使い放題（退職した社員のトークンが永久に有効）
- **別サービス用のトークン**が使える（`aud`がチェックされないため）
- **なりすましトークン**が通る（`iss`がチェックされないため）

### 安全な実装：全クレームを検証

```rust
/// SECURE: Properly validates all JWT claims
async fn secure_validate_token(headers: HeaderMap) -> Result<Json<TokenValidationResponse>, AppError> {
    // ...

    // SECURE: Enable all validation
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&[JWT_AUDIENCE]);  // この API 用か？
    validation.set_issuer(&[JWT_ISSUER]);      // 正当な発行者か？
    validation.validate_exp = true;             // 期限内か？

    let result = decode::<UserClaims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &validation,
    );
    // ...
}
```

### テスト用トークン生成

このデモでは4種類のトークンを生成できる。

```rust
async fn generate_test_token(Path(token_type): Path<String>) -> Result<Json<TokenInfo>, AppError> {
    let (claims, description) = match token_type.as_str() {
        "valid" => {
            // 有効なトークン（1時間後に期限切れ）
            let claims = UserClaims {
                exp: (Utc::now() + Duration::hours(1)).timestamp() as usize,
                aud: Some(JWT_AUDIENCE.to_string()),
                iss: Some(JWT_ISSUER.to_string()),
                // ...
            };
            (claims, "Valid token - expires in 1 hour")
        }
        "expired" => {
            // 期限切れトークン（1時間前に期限切れ）
            let claims = UserClaims {
                exp: (Utc::now() - Duration::hours(1)).timestamp() as usize, // 過去！
                // ...
            };
            (claims, "Expired token - expired 1 hour ago")
        }
        "wrong-audience" => {
            // 別サービス用のトークン
            let claims = UserClaims {
                aud: Some("https://wrong-audience.com".to_string()), // 別のサービス！
                // ...
            };
            (claims, "Token with wrong audience")
        }
        "wrong-issuer" => {
            // 不正な発行者のトークン
            let claims = UserClaims {
                iss: Some("https://malicious-issuer.com".to_string()), // 偽者！
                // ...
            };
            (claims, "Token with wrong issuer")
        }
        // ...
    };
}
```

攻撃シナリオは以下の通りである。

```bash
# 期限切れトークンを取得
EXPIRED=$(curl -s http://localhost:8080/token/expired | jq -r .access_token)

# 脆弱なエンドポイント → 通る！
curl -H "Authorization: Bearer $EXPIRED" http://localhost:8080/vulnerable/validate

# 安全なエンドポイント → 401 Unauthorized
curl -H "Authorization: Bearer $EXPIRED" http://localhost:8080/validate
```

### 微妙な脆弱性：JWT検証の巧妙なバイパス

「全クレームを検証しているから安全」と思っていないだろうか。残念ながら、JWT検証にはもっと狡猾な問題がある。

#### 微妙な脆弱性 #1: アルゴリズム混同攻撃

```rust
/// 開発者の意図: 「RS256もHS256もサポートして柔軟に」
/// 現実: RS256の公開鍵をHS256の秘密鍵として使われる
async fn subtle_alg_confusion(headers: HeaderMap) -> Result<...> {
    let header = jsonwebtoken::decode_header(token)?;

    // BUG: トークンが主張するアルゴリズムを信用
    let mut validation = Validation::new(header.alg);  // ← header.alg を信用！
    validation.set_audience(&[JWT_AUDIENCE]);
    validation.set_issuer(&[JWT_ISSUER]);

    // 攻撃:
    // 1. サーバーのRS256公開鍵を取得（公開されてる）
    // 2. その公開鍵をHS256の秘密鍵として使ってトークン署名
    // 3. {"alg": "HS256"} としてサーバーに送信
    // 4. サーバーは公開鍵を「HS256の秘密鍵」として検証 → 成功！
    let result = decode::<UserClaims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &validation,
    );
}
```

対策：アルゴリズムは固定値で指定。トークンの`alg`ヘッダーを信用してはいけない。

#### 微妙な脆弱性 #2: Key ID (kid) インジェクション

```rust
/// 開発者の意図: 「kidヘッダーで鍵を選択」
/// 現実: kidに任意の値を入れられる
async fn subtle_kid_injection(headers: HeaderMap) -> Result<...> {
    let header = jsonwebtoken::decode_header(token)?;

    // BUG: kidを検証なしで使用
    let kid = header.kid.unwrap_or_else(|| "default".to_string());

    // 実際の脆弱なコード例：
    // SQLインジェクション: kid = "key1' OR '1'='1"
    // let key = db.query(f"SELECT key FROM keys WHERE id = '{kid}'");

    // パストラバーサル: kid = "../../../etc/passwd"
    // let key = fs::read(format!("/keys/{}.pem", kid));

    // NULLキー: kid = "../../dev/null"
    // 空のキーで署名検証 → 常に成功
}
```

kidは信頼できない入力。許可リスト方式でキーを選択するべき。

#### 微妙な脆弱性 #3: JKU (JWK Set URL) バイパス

```rust
/// 開発者の意図: 「JKUヘッダーから公開鍵を取得」
/// 現実: 攻撃者のサーバーから鍵を取得させられる
async fn subtle_jku_bypass(headers: HeaderMap) -> Result<...> {
    let header = jsonwebtoken::decode_header(token)?;

    if let Some(jku) = header.jku {
        // BUG: 弱いチェック
        let allowed_prefix = "https://auth.example.com";
        if jku.starts_with(allowed_prefix) {
            // 攻撃:
            // jku = "https://auth.example.com.attacker.com/keys"
            // jku = "https://auth.example.com@attacker.com/keys"
            // jku = "https://auth.example.com%2F@attacker.com/keys"
            // 全部 starts_with チェックを通過！

            let keys = fetch_jwks_from_url(&jku).await?;
            // 攻撃者の公開鍵を取得 → 攻撃者が署名したトークンが有効に
        }
    }
}
```

JKUは使わないか、完全一致でURLをチェックするべき。

#### 微妙な脆弱性 #4: Not-Before (nbf) 未検証

```rust
/// 開発者の意図: 「expさえチェックすれば大丈夫」
/// 現実: 未来用に発行されたトークンが今使える
async fn subtle_nbf_skip(headers: HeaderMap) -> Result<...> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&[JWT_AUDIENCE]);
    validation.set_issuer(&[JWT_ISSUER]);
    validation.validate_exp = true;
    validation.validate_nbf = false;  // BUG: nbfを検証しない

    // 攻撃シナリオ:
    // 1. 管理者が「来月1日から有効」なトークンを事前発行
    // 2. そのトークンが漏洩
    // 3. 攻撃者は今すぐそのトークンを使用 → nbf無視で成功

    // または:
    // 1. 内部犯行者が未来日付のトークンを大量に生成
    // 2. 退職後にそれらを使用
    // 3. expはチェックされるがnbfはスルー → アクセス成功
}
```

nbfクレームもexpと同様に重要。「まだ有効ではない」トークンを拒否しないと、事前発行されたトークンが悪用される。

### HS256 vs RS256

JWT認証では2つの主要なアルゴリズムがある。

```rust
// HS256: 同じ鍵で署名と検証（対称鍵）
const HS256_SECRET: &str = "your-256-bit-secret-key-here-must-be-long-enough";

// RS256: 秘密鍵で署名、公開鍵で検証（非対称鍵）
const RS256_PRIVATE_KEY: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASC...
-----END PRIVATE KEY-----"#;

const RS256_PUBLIC_KEY: &str = r#"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8A...
-----END PUBLIC KEY-----"#;
```

なぜRS256が推奨されるのか。
- **HS256**: 署名と検証に同じ鍵を使う → 検証側にも秘密鍵が必要 → 漏洩リスク高
- **RS256**: 署名に秘密鍵、検証に公開鍵 → 公開鍵は配布しても安全 → マイクロサービス向き

---

## API3: Mass Assignment - 見えないフィールドを操作される

これは個人的に「一番やらかしやすい」脆弱性だ。そして「やらかしても気づきにくい」という意味で最も厄介だろう。

https://owasp.org/API-Security/editions/2023/en/0xa3-broken-object-property-level-authorization/

[https://owasp.org/API-Security/editions/2023/en/0xa3-broken-object-property-level-authorization/:embed:cite]

### Mass Assignmentとは何か

Mass Assignment（一括代入）は、クライアントから送られてきたデータを、サーバー側のオブジェクトにそのまま「一括で」割り当ててしまうことで発生する脆弱性だ。

もともとはRuby on RailsやPHPのLaravelなど、「お手軽にCRUDを作れるフレームワーク」で頻発していた。これは「フォームのフィールドをそのままDBカラムにマッピング」する機能が便利すぎて、セキュリティを犠牲にしていた。

Rustは型付けが厳格なので「安全」と思われがちだが、`serde`でJSONをデシリアライズする際に同様の問題が発生しうる。

### なぜ開発者はこのミスを犯すのか

1. **便利さの誘惑** - 「リクエストとモデルの型を同じにすればコードが減る」
2. **フィールド追加時の見落とし** - DBに`status`カラムを追加 → Rustの構造体にも追加 → リクエスト型にも追加 → やらかし
3. **「デフォルト値があるから大丈夫」という誤解** - `#[serde(default)]`は「送られなかったら」デフォルト、「送られたら」その値
4. **テスト時の盲点** - 正常系では余分なフィールドを送らないので気づかない

### 操作される可能性のあるフィールド

攻撃者が狙う典型的なフィールドは以下である。

| フィールド | 本来の用途 | 攻撃による悪用 |
|------------|-----------|---------------|
| `status` | 処理状態管理 | `"pending"` → `"approved"` で承認をバイパス |
| `role` | 権限管理 | `"user"` → `"admin"` で権限昇格 |
| `is_verified` | 検証フラグ | `false` → `true` で検証をスキップ |
| `price` | 価格 | 1000 → 1 で値引き |
| `user_id` | 所有者 | 他人のIDを指定してなりすまし |
| `created_at` | 作成日時 | 過去の日付を指定して古いデータを偽装 |
| `id` | 主キー | 既存IDを指定して上書き攻撃 |

例えば、支払い作成APIで、ユーザーが送ってきたJSONをそのまま使ってしまうケースを見てみよう。

```rust
/// VULNERABLE: Accepts any fields from user input
#[derive(Deserialize)]
pub struct UnsafePaymentRequest {
    pub amount: f64,
    pub currency: String,
    #[serde(default)]
    pub status: Option<String>,  // ユーザーが設定可能になっている
}

async fn vulnerable_create_payment(
    Json(req): Json<UnsafePaymentRequest>,
) -> Json<Payment> {
    let payment = Payment {
        id: Uuid::new_v4().to_string(),
        amount: req.amount,
        currency: req.currency,
        status: req.status.unwrap_or_else(|| "pending".to_string()),
        // ↑ ユーザーが"approved"を送ってきたらそのまま使っちゃう
    };
    Json(payment)
}
```

攻撃は以下の通りである。

```bash
curl -X POST http://localhost:8080/vulnerable/payments \
     -H "Content-Type: application/json" \
     -d '{"amount": 100, "currency": "USD", "status": "approved"}'
```

結果は`"status": "approved"`であり、未払いの支払いが承認済みになった。

支払いステータスを「承認済み」に設定して、実際には支払いをしない。システムは何も気づかない。

### 対策: DTOを分ける

```rust
/// SECURE: Only accepts allowed fields
#[derive(Deserialize)]
pub struct CreatePaymentRequest {
    pub amount: f64,
    pub currency: String,
    // statusフィールドは存在しない
}

async fn secure_create_payment(
    Json(req): Json<CreatePaymentRequest>,
) -> Json<Payment> {
    let payment = Payment::new(req.amount, req.currency);
    // statusは常にサーバー側で"pending"に設定される
    Json(payment)
}
```

入力用のDTOと内部用のモデルを分ける。コード量は増える。型定義は増える。でも、これが「自由度の高いAPI」と「セキュアなAPI」の違いだ。自由には責任が伴う。

### 微妙なMass Assignment：serde flattenの罠

「入力DTOを分けた」と言っても、実装の仕方次第で脆弱になる。

#### 微妙な脆弱性 #1: `#[serde(flatten)]`の問題

```rust
#[derive(Deserialize, Serialize)]
struct FlattenedPaymentRequest {
    amount: f64,
    currency: String,
    // 「未知のフィールドをログに残したい」という意図
    #[serde(flatten)]
    extra_fields: HashMap<String, serde_json::Value>,
}

async fn subtle_flatten_payment(
    State(state): State<Arc<AppState>>,
    _user: AuthenticatedUser,
    Json(req): Json<FlattenedPaymentRequest>,
) -> Result<Json<Payment>, AppError> {
    let mut payment = Payment::new(req.amount, req.currency.clone());

    // 「extra_fieldsに有効なstatusがあれば使おう」
    // 開発者の意図：「クライアントの便宜を図る」
    // 現実：Mass Assignmentの再来
    if let Some(status) = req.extra_fields.get("status") {
        if let Some(s) = status.as_str() {
            if ["pending", "approved", "rejected"].contains(&s) {
                payment.status = s.to_string();  // approved も有効な値！
            }
        }
    }

    state.db.create_payment(&payment)?;
    Ok(Json(payment))
}
```

`#[serde(flatten)]`と`HashMap`の組み合わせは便利だが、「未知のフィールドを捕捉する」という性質が裏目に出る。コードレビューで`flatten`を見たら警戒しよう。

#### 微妙な脆弱性 #2: 部分更新の罠

PATCH（部分更新）エンドポイントは特に危険だ。

```rust
#[derive(Deserialize)]
struct PartialPaymentUpdate {
    amount: Option<f64>,
    currency: Option<String>,
    // 「ユーザーが自分でキャンセルできるように」status を追加
    #[serde(default)]
    status: Option<String>,
}

async fn subtle_update_payment(
    State(state): State<Arc<AppState>>,
    _user: AuthenticatedUser,
    Path(payment_id): Path<String>,
    Json(update): Json<PartialPaymentUpdate>,
) -> Result<Json<Payment>, AppError> {
    let mut payment = state.db.get_payment_by_id(&payment_id)?
        .ok_or_else(|| AppError::NotFound(...))?;

    // 部分更新ロジック
    if let Some(amount) = update.amount {
        payment.amount = amount;
    }
    if let Some(currency) = update.currency {
        payment.currency = currency;
    }

    // 「キャンセルは許可、でも承認は決済システム経由のみ」のつもり
    if let Some(status) = update.status {
        if payment.status == "pending" && status == "approved" {
            // 開発者：「pendingからapprovedへの遷移だけ許可」
            // 現実：これがまさに攻撃者がやりたいこと！
            payment.status = status;
        } else if payment.status == "pending" && status == "cancelled" {
            payment.status = status;
        }
    }

    Ok(Json(payment))
}
```

条件分岐で「許可する遷移」を書いたつもりが、攻撃者が欲しいものを許可している。ロジックが複雑になるほど、こういうミスは見つけにくくなる。

攻撃方法は以下の通りである。

```bash
# 支払いを作成
PAYMENT_ID=$(curl -s -X POST http://localhost:8080/payments \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"amount": 100, "currency": "USD"}' | jq -r .id)

# 部分更新でステータスを承認済みに
curl -X POST "http://localhost:8080/subtle/payments/$PAYMENT_ID" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"status": "approved"}'
```

---

## 実装で学んだこと

### 1. 認証と認可は別物

これは何度言っても足りない。

- 認証: 「あなたは誰か」 → 「私はBobです」
- 認可: 「Bobさん、あなたはこれをしていいのか」 → 「...ダメです」

JWTを検証して「このユーザーは本物だ」とわかっても、「このユーザーがこのリソースにアクセスしていいか」は全く別の問題だ。

会社のビルで例えると以下の通りである。

- 認証 = 社員証を見せて入館する
- 認可 = サーバールームに入れるかどうか

社員証を持っていても、全員がサーバールームに入れるわけではない。当たり前だ。でも、APIでは「認証してるから大丈夫」と言ってしまいがちなのだ。

### 2. 404 vs 403

認可エラーの際に403を返すか404を返すか。

- 403: リソースの存在を明かしつつアクセスを拒否
- 404: リソースの存在自体を隠す

セキュリティ的には404が安全だ。403は「存在する」という情報を漏らしている。

しかし、デバッグは困難になる。「404なんだけど、本当に存在しないのか、権限がないのか」がわからない。本番環境では404、開発環境では403にするとか、ログには詳細を残すとか、工夫が必要だ。

### 3. DTOの分離は面倒だが必要

入力用の構造体と内部用の構造体を分けるのは、確かに面倒だ。同じようなものを2回書くことになる。

しかし、Mass Assignment攻撃を防ぐには必要なコストだ。

Rustの場合、コンパイル時に型チェックされるので、「うっかりユーザー入力をそのまま使ってしまう」ミスは起きにくい。`CreatePaymentRequest`に`status`フィールドがなければ、コンパイラが「そんなフィールドないよ」と教えてくれる。これはRustの強みだ。動的型付け言語だと、こうはいかない。

---

**続きは[後編](./)へ** → API4 (Rate Limit), API5 (BFLA), API7 (SSRF), 動作確認、まとめ

---

## 参考リンク

### OWASP API Security Top 10 (2023)

公式ドキュメント。

[https://owasp.org/API-Security/editions/2023/en/0x11-t10/:embed:cite]

### axum - Rust Web Framework

本デモで使用しているWebフレームワーク。

[https://github.com/tokio-rs/axum:embed:cite]

### jsonwebtoken - Rust JWT Library

JWT認証の実装に使用。

[https://github.com/Keats/jsonwebtoken:embed:cite]

### thiserror - Rust Error Handling

エラー型の定義に使用。

[https://github.com/dtolnay/thiserror:embed:cite]

### JWT.io

JWTのデバッグ・検証ツール。

[https://jwt.io/:embed:cite]

### RFC 7519 - JSON Web Token (JWT)

JWTの仕様。

[https://datatracker.ietf.org/doc/html/rfc7519:embed:cite]

### CWE-639: Authorization Bypass Through User-Controlled Key

BOLAに関連するCWEエントリ。

[https://cwe.mitre.org/data/definitions/639.html:embed:cite]

### CWE-915: Improperly Controlled Modification of Dynamically-Determined Object Attributes

Mass Assignmentに関連するCWEエントリ。

[https://cwe.mitre.org/data/definitions/915.html:embed:cite]

