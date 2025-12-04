# 「このAPIは安全です」と言い切れますか？RustでOWASP API Security Top 10を体験する

先日、あるプロジェクトのコードレビューで「このエンドポイント、認証は通ってるけど認可は大丈夫？」と聞いたら、「認証してるから大丈夫でしょ」という返答が返ってきた。

その瞬間、私の脳内では警報が鳴り響いた。これはあれだ。「鍵がかかってるから金庫は安全」と言いながら、金庫の中身を誰でも見れる状態にしているやつだ。

認証（Authentication）と認可（Authorization）の違い。頭ではわかっていても、実際のコードでどう違うのか、どう危険なのかを体感したことがある人は意外と少ない。かくいう私も、セキュリティの本を読んで「ふーん」と思いながら、翌日には同じミスをやらかしていた口だ。

そこで今回、OWASP API Security Top 10の脆弱性を**実際に攻撃できる形**でRustで実装してみた。「脆弱なエンドポイント」と「安全なエンドポイント」を並べて、攻撃がどう成功し、どう防げるのかを手を動かして確認できる。百聞は一見にしかず。百見は一攻撃にしかず（？）。

## なぜBobとAliceなのか（セキュリティ界の永遠の主人公たち）

セキュリティの例でやたらと「BobがAliceのデータを〜」という話が出てくる。なぜこの2人なのか。

これは1978年にRon Rivest、Adi Shamir、Leonard Adleman（RSA暗号のRSA）が書いた論文「A Method for Obtaining Digital Signatures and Public-Key Cryptosystems」に由来する。彼らは暗号通信の説明に「AさんがBさんにメッセージを送る」ではなく、「AliceがBobにメッセージを送る」と書いた。AとBで始まる名前を選んだだけだが、これが定着した。

その後、セキュリティの世界では登場人物が増えていった：

- **Alice & Bob**: 通信したい善良な2人（主人公）
- **Eve**: 盗聴者（Eavesdropperから。悪役その1）
- **Mallory**: 能動的攻撃者（Maliciousから。もっと悪い悪役）
- **Trent**: 信頼できる第三者（Trustedから）
- **Carol/Charlie**: 3人目の参加者が必要なとき

つまり、BobとAliceは「セキュリティ界のサザエさん一家」みたいなものだ。何十年も同じ役を演じ続けている。Bobは何回Aliceのデータを盗み見てきたのか。Aliceは何回被害に遭ってきたのか。考えると少し切ない。

本記事でも、この伝統に従ってBobとAliceに登場してもらう。Bobには悪役を演じてもらうことになるが、本来のBobは悪い人ではない。「認可が不十分だと善良なBobでも悪いことができてしまう」というのが本質的な問題なのだ。

## なぜ「体験」が必要なのか

セキュリティの勉強で一番難しいのは、「危険性を実感すること」だと思う。

ドキュメントを読んで「BOLAは危険です」と書いてあっても、「へー、そうなんだ」で終わる。これは人間の性だ。交通事故のニュースを見ても「自分は大丈夫」と思うのと同じで、実際にBobがAliceのデータを抜き取る瞬間を見ないと、その怖さは伝わらない。

このデモを作った動機は単純で、**自分が「あ、これ確かにヤバい」と冷や汗をかける教材が欲しかった**からだ。本を読んで「なるほど」と思っても、3日後には忘れている。でも、自分の手で攻撃を成功させた経験は忘れない。

ちなみに、このデモを作っている最中に「あれ、これ本番のコードにも似たようなのあったな...」と気づいて本当に冷や汗をかいた。勉強は大事。

## デモの全体像

このデモは9つのバイナリで構成されている。それぞれが独立したWebサーバーとして起動し、まるで「セキュリティの体験型テーマパーク」のような構成だ：

1. `/token/{user_id}` でテスト用JWTを取得（受付でチケットをもらう）
2. `/vulnerable/...` で脆弱なエンドポイントを叩く（お化け屋敷で驚かされる）
3. `/...` で安全なエンドポイントを叩く（お化けが出てこない...つまらない？いや、それが正しい）

```
api-security-demo/
├── src/bin/
│   ├── bola.rs              # 他人のデータが見放題！（ダメ）
│   ├── bfla.rs              # 一般人が管理者に！（もっとダメ）
│   ├── mass_assignment.rs   # 支払いステータス勝手に変更！（犯罪）
│   ├── broken_auth.rs       # 期限切れチケットで入場！（せこい）
│   ├── rate_limit.rs        # パスワード総当たり！（根気）
│   ├── ssrf.rs              # サーバーを踏み台に！（頭いい、でもダメ）
│   ├── jwt.rs               # トークンいじくり回し
│   ├── observability.rs     # 不審者検知システム
│   └── security_test.rs     # 自己診断
```

技術スタックはRust + axum。Rust 2024エディションで書いている。「なぜRust？」と聞かれたら「型安全性がセキュリティにも寄与するから」と答えるが、本音は「Rustで書くとかっこいいから」だ。

### 前提条件

試してみたい方は以下が必要：

- **Rust 1.85以上**（2024エディション対応）
- **curl** と **jq**（テスト用）
- **好奇心**（必須）
- **良心**（本番環境で試さないという誓い）

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

共通ロジックは`src/`配下にライブラリとして切り出し、各デモは`src/bin/`配下の独立したバイナリとして実装している。これにより：

1. **コードの再利用**: 認証、DB操作、エラーハンドリングを全デモで共有
2. **単一責任**: 各バイナリは1つの脆弱性カテゴリに集中
3. **独立した起動**: `cargo run --bin bola-demo`で特定のデモだけ起動可能

### エラーハンドリング設計

Rustらしいエラー設計を採用した。`thiserror`クレートで列挙型エラーを定義し、axumの`IntoResponse`を実装：

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

なぜ`anyhow::Error`ではなく独自のエラー型か？

- **HTTPステータスコードの制御**: エラーの種類によって401、403、404、429などを返し分けたい
- **クライアントへのメッセージ制御**: 内部エラーの詳細は隠し、適切なメッセージだけ返したい
- **コンパイル時の網羅性チェック**: `match`で全ケースを処理しているか確認できる

`IntoResponse`の実装：

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

axumの`FromRequestParts`トレイトを実装したExtractorを使う。これがこのデモの核心部分だ：

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

Extractorパターンの利点：

1. **宣言的**: 関数シグネチャに`AuthenticatedUser`があれば認証必須と一目でわかる
2. **再利用可能**: 同じExtractorを全エンドポイントで使い回せる
3. **テスト容易**: Extractorを差し替えてテスト可能
4. **失敗時の自動レスポンス**: 認証失敗時は自動で401を返す

「脆弱な」バージョンも用意している：

```rust
/// Extractor for user claims WITHOUT proper validation (vulnerable version)
#[derive(Debug, Clone)]
pub struct VulnerableAuthUser(pub UserClaims);
```

これは署名検証をスキップし、期限切れトークンも受け入れる。教育目的のみ。

### データベース層の設計

SQLiteを使い、認可の有無でメソッドを分けている：

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

「なぜSQLで認可するの？アプリケーション層でフィルタすればいいのでは？」

良い質問だ。アプリケーション層でも可能だが、DB層で認可する利点がある：

1. **パフォーマンス**: 不要なデータをDBから取得しない
2. **防御の多層化**: アプリ層のバグがあってもDB層で防げる
3. **一貫性**: SQLで認可ロジックが一箇所に集約される

ただし、複雑な認可ルール（「自分のチームのデータ」など）はアプリ層で実装したほうが保守しやすい場合もある。

### JWT検証の実装

安全なバージョンと脆弱なバージョンを比較：

```rust
/// Validate a JWT token with HS256 (secure version with full validation)
pub fn validate_token_hs256(token: &str) -> Result<UserClaims, AppError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&[JWT_ISSUER]);      // issuerを検証
    validation.set_audience(&[JWT_AUDIENCE]);  // audienceを検証
    validation.validate_exp = true;             // 有効期限を検証

    let token_data = decode::<UserClaims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &validation,
    )?;

    Ok(token_data.claims)
}

/// Validate a JWT token WITHOUT proper validation (vulnerable version)
pub fn validate_token_vulnerable(token: &str) -> Result<UserClaims, AppError> {
    let mut validation = Validation::new(Algorithm::HS256);
    // VULNERABLE: Not validating issuer, audience, or expiration
    validation.validate_exp = false;
    validation.validate_aud = false;
    validation.insecure_disable_signature_validation();  // 署名すら検証しない！

    // ...
}
```

脆弱なバージョンは**署名を検証しない**。つまり、攻撃者が任意のペイロードを含むJWTを作成して送信できる。`{"sub": "admin", "permissions": ["admin"]}`というペイロードを持つトークンを自分で作れば、管理者になりすませる。

### 依存関係の選定理由

`Cargo.toml`から主要な依存関係とその理由：

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

各モジュールにユニットテストを配置：

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

さらに、`scripts/test_all.sh`でE2E的な統合テストを実行。各エンドポイントに実際にHTTPリクエストを送り、脆弱なエンドポイントで攻撃が成功すること、安全なエンドポイントで攻撃が失敗することを検証する。

## BOLA: 最も危険で、最も見落とされやすい脆弱性

OWASP API Security Top 10の堂々第1位がBOLA（Broken Object Level Authorization）だ。日本語では「オブジェクトレベル認可の不備」。

https://owasp.org/API-Security/editions/2023/en/0xa1-broken-object-level-authorization/

[https://owasp.org/API-Security/editions/2023/en/0xa1-broken-object-level-authorization/:embed:cite]

名前が難しそう？大丈夫、中身は簡単だ。要するに**「BobがAliceのデータを見れてしまう」**という、小学生でも「それダメでしょ」とわかる問題だ。でも、驚くほど多くの本番システムにこれがある。人類は学ばない。

### なぜBOLAが最も危険なのか

BOLAが1位である理由は明確だ：

1. **発生頻度が非常に高い** - ほぼすべてのAPIがリソースIDを扱う。そのすべてで認可チェックが必要
2. **自動化しやすい** - 攻撃者はIDを1, 2, 3...と順に試すだけ。スクリプト数行で全データを列挙できる
3. **検出が困難** - 正規のリクエストと見分けがつかない。WAFでは防げない
4. **影響が甚大** - 顧客データ、取引履歴、個人情報がすべて漏洩する可能性

### 実際のインシデント事例

BOLAによる情報漏洩は数え切れないほど発生している：

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

### 脆弱なコード（悪い例、真似しないでね）

```rust
/// VULNERABLE: Returns any order by ID without checking ownership
async fn vulnerable_get_order(
    State(state): State<Arc<AppState>>,
    _user: AuthenticatedUser, // おっ、認証してるじゃん！えらい！
    Path(order_id): Path<i64>,
) -> Result<Json<Order>, AppError> {
    // ...と思ったら使ってないんかーい！
    let order = state.db.get_order_by_id(order_id)?
        .ok_or_else(|| AppError::NotFound(format!("Order {} not found", order_id)))?;

    Ok(Json(order))
}
```

`_user`としてわざわざ認証情報を受け取っているのに、**アンダースコアつけて無視している**。これは「セキュリティチェックしてますよ」というアリバイ作りにすらなっていない。むしろ「チェックしようとして忘れた」という証拠だ。

### 安全なコード（こっちを真似してね）

```rust
/// SECURE: Returns order only if it belongs to the authenticated user
async fn secure_get_order(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,  // アンダースコアなし！ちゃんと使う！
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

違いは1行だけ。たった1行。でも、この1行が「情報漏洩インシデント発生」と「平和な日常」の分かれ道だ。1行の価値、プライスレス。

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

攻撃方法：
```bash
# Bobとして認証
BOB_TOKEN=$(curl -s http://localhost:8080/token/bob | jq -r .access_token)

# クエリパラメータでAliceになりすまし
curl -H "Authorization: Bearer $BOB_TOKEN" \
     "http://localhost:8080/subtle/orders/1?user_id=alice"
```

このパターンは実際のコードレビューでよく見る。「管理画面でユーザーを切り替えて確認したい」「サポート担当がユーザーの代わりに操作する機能が必要」などの要件から生まれがち。対策は「そもそもこの機能は必要か？」を問い直すことと、必要なら別の認証フローを用意すること。

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

何が問題か：
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

正しいパターン：
1. 認可前のログは「誰が」「何にアクセスしようとしたか（IDのみ）」
2. 認可後のログは詳細情報を含めてOK

### 実際に攻撃してみる（ハッカー気分を味わおう）

```bash
# サーバー起動
cargo run --release --bin bola-demo

# Bobのトークンを取得（Bobは善良な一般ユーザーのはずだった）
BOB_TOKEN=$(curl -s http://localhost:8080/token/bob | jq -r .access_token)

# 脆弱なエンドポイント: BobがAliceの注文(ID=1)を取得
# ※良い子は本番環境でやらないでね
curl -H "Authorization: Bearer $BOB_TOKEN" \
     http://localhost:8080/vulnerable/orders/1
```

結果：
```json
{
  "id": 1,
  "user": "alice",
  "product": "Widget A",
  "quantity": 5
}
```

**Bobが、Aliceの注文データを取得できてしまった。** Aliceは知らない。Bobは黙っている。システムは何も気づいていない。これが現実のインシデントだったら、ニュースになるやつだ。

安全なエンドポイントでは：
```bash
curl -H "Authorization: Bearer $BOB_TOKEN" \
     http://localhost:8080/orders/1
```

結果：
```json
{
  "error": "Order 1 not found or access denied"
}
```

404を返している点もポイントだ。「なんで403（Forbidden）じゃないの？」と思うかもしれない。

- 403: 「その注文は存在するよ。でもお前には見せない」
- 404: 「何の話？そんな注文知らないけど？」

403は正直すぎる。「存在する」という情報すら漏らさない404のほうがセキュリティ的には優秀だ。嘘も方便。セキュリティの世界では「知らないふり」は美徳なのだ。

## BFLA: 一般ユーザーが管理者になれてしまう問題

BFLA（Broken Function Level Authorization）は、BOLAの「機能版」だ。

https://owasp.org/API-Security/editions/2023/en/0xa5-broken-function-level-authorization/

[https://owasp.org/API-Security/editions/2023/en/0xa5-broken-function-level-authorization/:embed:cite]

BOLAが「他人のデータを見れる」なら、BFLAは「使えないはずの機能が使える」。例えば、一般ユーザーが管理者用のユーザー一覧APIを叩けてしまうケース。言ってみれば「平社員が社長の権限でシステムを操作できる」状態だ。

### BOLAとBFLAの違いを理解する

この2つは混同しやすいので、明確に区別しよう：

| 項目 | BOLA | BFLA |
|------|------|------|
| 何が壊れている？ | オブジェクト（データ）へのアクセス制御 | 機能（エンドポイント）へのアクセス制御 |
| 攻撃例 | BobがAliceの注文を見る | 一般ユーザーが管理者APIを叩く |
| チェック対象 | 「このデータは誰のもの？」 | 「この機能は誰が使える？」 |
| 典型的な対策 | リソースごとの所有者チェック | ロール/権限チェック |

例えで言えば：
- **BOLA** = 他人のロッカーを開けられる（同じ権限レベル内での越境）
- **BFLA** = 社員証がないのに役員室に入れる（権限レベルの越境）

### なぜBFLAが発生するのか

1. **エンドポイントの「発見」** - `/api/users`があるなら`/api/admin/users`もあるかも？と攻撃者は考える
2. **フロントエンドによる隠蔽への過信** - 「管理メニューは管理者にしか見せてないから大丈夫」→ APIは直接叩ける
3. **認証と認可の混同（再び）** - 「ログインしてるから管理APIも使えるはず」という誤った思い込み
4. **テスト不足** - 管理者機能は管理者アカウントでしかテストしない
5. **ドキュメント化されていない管理API** - 「隠しAPI」は攻撃者に見つかる

### 実際の被害パターン

BFLAによって可能になる攻撃：

- **ユーザー情報の一括取得** - 全ユーザーのメールアドレス、個人情報を抜き取る
- **権限昇格** - 自分のアカウントに管理者権限を付与する
- **システム設定の変更** - APIキーの再生成、課金設定の変更
- **データの一括削除** - 管理者用の一括削除機能を悪用
- **監査ログの改ざん** - 証拠隠滅のためにログを消去

```rust
/// VULNERABLE: No role check
/// （「認証さえ通れば誰でもウェルカム！」...それダメなやつ）
async fn vulnerable_list_users(user: AuthenticatedUser) -> Result<Json<Vec<UserInfo>>, AppError> {
    Ok(Json(vec![
        UserInfo {
            id: 1,
            email: "admin@example.com".to_string(),
            role: "admin".to_string(),
            ssn: "123-45-6789".to_string(), // SSNまで露出してる...
        },
        // 以下略（被害は続く）
    ]))
}

/// SECURE: Admin check（大人の対応）
async fn secure_list_users(user: AuthenticatedUser) -> Result<Json<Vec<SafeUserInfo>>, AppError> {
    if !is_admin(&user.0) {
        return Err(AppError::Forbidden("Admin permission required".to_string()));
        // 「お前は管理者じゃない。帰れ。」
    }
    // ...
}
```

`is_admin`のチェックは単純だ：

```rust
pub fn is_admin(claims: &UserClaims) -> bool {
    claims.permissions.iter().any(|p| p == "admin")
}
```

「これくらい誰でも書くでしょ」と思うかもしれない。でも、本番環境で「認証は通ってるから大丈夫」と言ってこのチェックを忘れる人が後を絶たないのだ。人間を信用してはいけない。

### 微妙な脆弱性：一見正しく見えるBFLAのバグ

「`is_admin`チェックさえ入れれば安全」と思っていないだろうか。残念ながら、そう単純ではない。

#### 微妙な脆弱性 #1: HTTPヘッダーを信用する

```rust
/// 開発者の意図: 「フロントエンドが送るX-User-Roleヘッダーを信用しよう」
/// 現実: curlでいくらでも偽装できる
async fn subtle_header_role_check(
    user: AuthenticatedUser,
    headers: HeaderMap,
) -> Result<Json<AdminResponse>, AppError> {
    // BUG: HTTPヘッダーを信用している！
    let role = headers
        .get("X-User-Role")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("user");

    if role != "admin" {
        return Err(AppError::Forbidden("Admin role required".to_string()));
    }
    // 攻撃: curl -H "X-User-Role: admin" ...
    Ok(Json(admin_data))
}
```

フロントエンドから「便利だから」とヘッダーでロール情報を送る設計を見たことがある。これは完全にアウトだ。HTTPヘッダーはクライアントが自由に設定できる。JWTのペイロードのように署名で保護されていない限り、信用してはいけない。

#### 微妙な脆弱性 #2: JWTクレームをDBと照合しない

```rust
/// 開発者の意図: 「JWTに権限が入っているから、それを使えばOK」
/// 現実: トークン発行後にユーザーが降格されたら？
async fn subtle_client_claims_check(
    user: AuthenticatedUser,
) -> Result<Json<AdminResponse>, AppError> {
    // これ、一見正しそう
    let has_admin = user.0.permissions.iter().any(|p| p == "admin");

    if !has_admin {
        return Err(AppError::Forbidden("Admin permission required".to_string()));
    }

    // 問題: ユーザーが管理者だったのは「トークン発行時」の話
    // トークン発行後に降格されていても、トークンが有効な限りアクセスできてしまう
    Ok(Json(admin_data))
}
```

JWTは便利だが、「トークン発行時点のスナップショット」に過ぎない。ユーザーの権限が変更されたら、古いトークンは無効にするか、DBで再確認する必要がある。

#### 微妙な脆弱性 #3: 大文字小文字の罠

```rust
/// 開発者の意図: 「adminをチェックすれば安全」
/// 現実: 「Admin」「ADMIN」「aDmIn」は？
let has_admin = user.0.permissions.iter().any(|p| p == "admin");
```

これ自体は問題ないが、トークン生成側で大文字小文字の統一が取れていないと問題になる。ある箇所では`"admin"`、別の箇所では`"Admin"`で権限が付与されていたら、チェックをすり抜けてしまう。

```rust
// 安全な実装: 大文字小文字を無視
let has_admin = user.0.permissions.iter()
    .any(|p| p.eq_ignore_ascii_case("admin"));
```

#### 微妙な脆弱性 #4: キャッシュされた権限チェック

```rust
/// 開発者の意図: 「ミドルウェアで権限チェック済みだから、エンドポイントでは確認不要」
/// 現実: そのキャッシュ、どこから来た？
async fn subtle_cached_permission_check(
    user: AuthenticatedUser,
    Query(query): Query<CachedCheckQuery>,
) -> Result<Json<AdminResponse>, AppError> {
    // BUG: クエリパラメータから「チェック済み」フラグを読んでいる！
    let is_verified_admin = query.permission_verified.unwrap_or(false);

    if is_verified_admin {
        // 攻撃: ?permission_verified=true
        return Ok(Json(admin_data));
    }

    // 本来のチェック
    if !is_admin(&user.0) {
        return Err(AppError::Forbidden("Admin permission required".to_string()));
    }
    Ok(Json(admin_data))
}
```

「ミドルウェアでチェック済み」というフラグをリクエストに含めるパターンは意外とある。でもそのフラグがクエリパラメータやヘッダーから来ていたら、攻撃者が自由に設定できる。

## Mass Assignment: 見えないフィールドを操作される

これは個人的に「一番やらかしやすい」と思っている脆弱性だ。そして「やらかしても気づきにくい」という意味で最も厄介かもしれない。

https://owasp.org/API-Security/editions/2023/en/0xa3-broken-object-property-level-authorization/

[https://owasp.org/API-Security/editions/2023/en/0xa3-broken-object-property-level-authorization/:embed:cite]

### Mass Assignmentとは何か

Mass Assignment（一括代入）は、クライアントから送られてきたデータを、サーバー側のオブジェクトにそのまま「一括で」割り当ててしまうことで発生する脆弱性だ。

もともとはRuby on RailsやPHPのLaravelなど、「お手軽にCRUDを作れるフレームワーク」で頻発していた。これらは「フォームのフィールドをそのままDBカラムにマッピング」する機能が便利すぎて、セキュリティを犠牲にしていた。

Rustは型付けが厳格なので「安全」と思われがちだが、`serde`でJSONをデシリアライズする際に同様の問題が発生しうる。

### なぜ開発者はこのミスを犯すのか

1. **便利さの誘惑** - 「リクエストとモデルの型を同じにすればコードが減る」
2. **フィールド追加時の見落とし** - DBに`status`カラムを追加 → Rustの構造体にも追加 → リクエスト型にも追加 → やらかし
3. **「デフォルト値があるから大丈夫」という誤解** - `#[serde(default)]`は「送られなかったら」デフォルト、「送られたら」その値
4. **テスト時の盲点** - 正常系では余分なフィールドを送らないので気づかない

### 操作される可能性のあるフィールド

攻撃者が狙う典型的なフィールド：

| フィールド | 本来の用途 | 攻撃による悪用 |
|------------|-----------|---------------|
| `status` | 処理状態管理 | `"pending"` → `"approved"` で承認をバイパス |
| `role` | 権限管理 | `"user"` → `"admin"` で権限昇格 |
| `is_verified` | 検証フラグ | `false` → `true` で検証をスキップ |
| `price` | 価格 | 1000 → 1 で値引き |
| `user_id` | 所有者 | 他人のIDを指定してなりすまし |
| `created_at` | 作成日時 | 過去の日付を指定して古いデータを偽装 |
| `id` | 主キー | 既存IDを指定して上書き攻撃 |

例えば、支払い作成APIで、ユーザーが送ってきたJSONをそのまま使ってしまうケース：

```rust
/// VULNERABLE: Accepts any fields from user input
/// （「お客様は神様です」を文字通り実装した結果）
#[derive(Deserialize)]
pub struct UnsafePaymentRequest {
    pub amount: f64,
    pub currency: String,
    #[serde(default)]
    pub status: Option<String>,  // えっ、ユーザーが設定できちゃうの？
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

攻撃（悪い人のやり方）：
```bash
curl -X POST http://localhost:8080/vulnerable/payments \
     -H "Content-Type: application/json" \
     -d '{"amount": 100, "currency": "USD", "status": "approved"}'
```

結果：`"status": "approved"` — 未払いの支払いが承認済みになった。

100ドルの商品を買って、支払いステータスを「承認済み」にして、タダで持っていく。これはもう詐欺だ。そしてシステムは何も気づかない。開発者も気づかない。経理が「あれ？」と思うまで気づかない。

### 対策: DTOを分ける（面倒だけど、やれ）

```rust
/// SECURE: Only accepts allowed fields
/// （「お客様は神様」ではない。適切に疑え）
#[derive(Deserialize)]
pub struct CreatePaymentRequest {
    pub amount: f64,
    pub currency: String,
    // statusフィールドは存在しない。ユーザーに触らせない。
}

async fn secure_create_payment(
    Json(req): Json<CreatePaymentRequest>,
) -> Json<Payment> {
    let payment = Payment::new(req.amount, req.currency);
    // statusは常にサーバー側で"pending"に設定される
    // ユーザーが何を送ってきても知ったことではない
    Json(payment)
}
```

入力用のDTOと内部用のモデルを分ける。コード量は増える。型定義は増える。でも、これが「自由度の高いAPI」と「セキュアなAPI」の違いだ。自由には責任が伴う。

### 微妙なMass Assignment：serde flattenの罠

「入力DTOを分けました！」と言っても、実装の仕方次第で脆弱になる。

#### 微妙な脆弱性 #1: `#[serde(flatten)]`の落とし穴

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

PATCH（部分更新）エンドポイントは特に危険だ：

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

攻撃方法：
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

## SSRF: サーバーを踏み台にする

SSRF（Server-Side Request Forgery）は、サーバーに「代わりにリクエストを送らせる」攻撃だ。

https://owasp.org/API-Security/editions/2023/en/0xa7-server-side-request-forgery/

[https://owasp.org/API-Security/editions/2023/en/0xa7-server-side-request-forgery/:embed:cite]

「え、それの何が問題？」と思うかもしれない。問題は、サーバーは内部ネットワークにアクセスできるということだ。外からは見えない場所に、サーバー経由で到達できてしまう。いわば「内部犯行」をサーバーにやらせるようなものだ。

### SSRFの危険性を理解する

SSRFが特に危険な理由：

1. **ファイアウォールをバイパス** - 外部からは遮断されていても、内部からのリクエストは通る
2. **クラウドメタデータにアクセス** - AWS/GCPの`169.254.169.254`から認証情報を取得可能
3. **内部サービスの探索** - ポートスキャンや内部APIの発見に悪用
4. **認証のバイパス** - 「内部ネットワークからのアクセスは信頼」という設計を悪用

### クラウド環境での致命的な被害

クラウド環境でのSSRFは特に危険だ。2019年のCapital One事件では、SSRFを使ってAWSのメタデータサービスにアクセスし、1億人以上の顧客データが漏洩した。

攻撃の流れ：
```
1. 攻撃者: http://169.254.169.254/latest/meta-data/iam/security-credentials/ にアクセスさせる
2. サーバー: 内部からのリクエストなので通常通り処理
3. AWSメタデータ: IAMロールの一時認証情報を返す
4. 攻撃者: その認証情報でS3バケットにアクセス → 大量のデータを取得
```

### SSRFが発生しやすい機能

以下のような機能はSSRFの温床になりやすい：

- **URLプレビュー/OGP取得** - 「このURLのタイトルと画像を表示」
- **Webhook送信** - 「指定されたURLにPOSTリクエストを送る」
- **PDF生成** - 「このURLの内容をPDFにする」（ヘッドレスブラウザがURLを開く）
- **画像のリサイズ/変換** - 「このURLの画像をサムネイルにする」
- **インポート機能** - 「このURLからデータをインポート」

例えば、「URLを指定したらそのページの内容を取得する」機能があったとする：

```rust
/// VULNERABLE: Fetches any URL
/// （「どんなURLでも取ってきますよ！」...それ、内部URLも？）
async fn vulnerable_fetch(Json(req): Json<FetchUrlRequest>) -> Result<String, AppError> {
    let response = reqwest::get(&req.url).await?;
    Ok(response.text().await?)
}
```

攻撃者は内部ネットワークのURLを指定する：
```bash
curl -X POST http://localhost:8080/vulnerable/fetch \
     -d '{"url":"http://localhost:8080/internal/secrets"}'
```

`/internal/secrets` は本来、外部からアクセスできない内部APIだ。でも、サーバー自身が「localhost」にアクセスするのは当然許可されている。結果、攻撃者はサーバーを「内部協力者」として使い、機密情報を引き出す。

サーバーは「言われたことを忠実に実行する」だけだ。それが悪意あるリクエストだとは気づかない。真面目に働くほど危ないという、皮肉な状況。

### 対策: 許可リストとプロトコル制限（信用するな、検証しろ）

```rust
async fn secure_fetch(Json(req): Json<FetchUrlRequest>) -> Result<String, AppError> {
    let url = Url::parse(&req.url)
        .map_err(|_| AppError::BadRequest("Invalid URL".to_string()))?;

    // HTTPSのみ許可（HTTPは時代遅れ）
    if url.scheme() != "https" {
        return Err(AppError::BadRequest("Only HTTPS URLs are allowed".to_string()));
    }

    // 許可されたドメインのみ（ホワイトリスト最強）
    let allowed_domains = ["api.example.com", "cdn.example.com"];
    let host = url.host_str()
        .ok_or_else(|| AppError::BadRequest("Invalid host".to_string()))?;

    if !allowed_domains.contains(&host) {
        return Err(AppError::BadRequest("Domain not in allowlist".to_string()));
    }

    // ここまで生き残ったURLだけが許される
    // ...
}
```

「なんでも取ってくる」から「許可されたものだけ取ってくる」へ。自由度は下がるが、セキュリティは上がる。トレードオフの世界へようこそ。

### 微妙な脆弱性：SSRFの巧妙なバイパス手法

「ホワイトリストでドメインチェックしてるから安全」と思っていないだろうか。残念ながら、SSRFは想像以上に狡猾だ。

#### 微妙な脆弱性 #1: リダイレクトを追跡してしまう

```rust
/// 開発者の意図: 「最初のURLを検証すればOK」
/// 現実: リダイレクト先は検証されていない
async fn subtle_redirect_ssrf(Json(req): Json<FetchUrlRequest>) -> Result<String, AppError> {
    let parsed_url = Url::parse(&req.url)?;

    // 最初のURLは検証する
    if !ALLOWED_DOMAINS.contains(&parsed_url.host_str().unwrap()) {
        return Err(AppError::BadRequest("Domain not allowed".to_string()));
    }

    // BUG: リダイレクトを10回まで追跡する
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()?;

    // 攻撃:
    // 1. パートナーサイト webhook.partner.com を許可リストに追加
    // 2. パートナーが webhook.partner.com/redirect?to=http://localhost/internal を設定
    // 3. 最初は検証を通過、リダイレクトで内部サーバーにアクセス
    let response = client.get(&req.url).send().await?;
    Ok(response.text().await?)
}
```

パートナーサイトやCDNを許可リストに入れていて、そこにオープンリダイレクトがあったら終わり。リダイレクト先も検証するか、リダイレクトを無効にするべき。

#### 微妙な脆弱性 #2: DNSリバインディング

```rust
/// 開発者の意図: 「DNSで解決されたIPをチェックすれば内部アクセスを防げる」
/// 現実: DNSの応答は変わりうる
async fn subtle_dns_rebinding(Json(req): Json<FetchUrlRequest>) -> Result<String, AppError> {
    let host = Url::parse(&req.url)?.host_str().unwrap().to_string();

    // 最初のDNS解決（ここでは外部IP）
    let ips = tokio::net::lookup_host(format!("{}:80", host)).await?;
    for ip in ips {
        if ip.ip().to_string().starts_with("127.") {
            return Err(AppError::BadRequest("Internal IP blocked".to_string()));
        }
    }

    // BUG: 実際のリクエスト時には別のDNS解決が行われる可能性
    // 攻撃者のDNSサーバー:
    // 1回目のクエリ → 1.2.3.4（外部IP、チェック通過）
    // 2回目のクエリ → 127.0.0.1（内部IP！）
    tokio::time::sleep(Duration::from_millis(100)).await;  // この間にDNSが変わる
    let response = reqwest::get(&req.url).await?;
    Ok(response.text().await?)
}
```

DNSリバインディング攻撃は、DNSの応答を時間差で変えることで検証をすり抜ける。対策は「解決したIPを直接使う」か「DNSピンニング」を実装すること。

#### 微妙な脆弱性 #3: URLパーサーの差異を悪用

```rust
/// 開発者の意図: 「URLをパースしてホストを検証」
/// 現実: 検証時と実際のリクエスト時でパーサーが違う
async fn subtle_parser_differential(Json(req): Json<FetchUrlRequest>) -> Result<String, AppError> {
    // url クレートでパース
    let parsed_url = Url::parse(&req.url)?;
    let host = parsed_url.host_str().unwrap();

    if !ALLOWED_DOMAINS.contains(&host) {
        return Err(AppError::BadRequest("Domain not allowed".to_string()));
    }

    // BUG: reqwest内部のHTTPクライアントが別のパースをする可能性
    // 攻撃例:
    // "https://api.github.com@localhost/internal/secrets"
    //   → url クレート: github.com がホスト
    //   → 一部のHTTPクライアント: localhost がホスト
    let response = reqwest::get(&req.url).await?;
    Ok(response.text().await?)
}
```

URLの解釈は実装によって微妙に異なる。ユーザー情報（`user@host`）、フラグメント（`#`）、エンコーディングの扱いなど、差異を悪用される可能性がある。

#### 微妙な脆弱性 #4: プロトコル/エンコーディングの罠

```rust
/// 開発者の意図: 「エンコードされたURLもサポートしよう」
/// 現実: 検証するURLとリクエストするURLが違う
async fn subtle_protocol_smuggling(Json(req): Json<EncodedUrlRequest>) -> Result<String, AppError> {
    let url_to_validate = if req.decode_first.unwrap_or(false) {
        // URLデコードしてから検証
        naive_percent_decode(&req.url)
    } else {
        req.url.clone()
    };

    // デコード後のURLを検証
    let parsed = Url::parse(&url_to_validate)?;
    // ... validation ...

    // BUG: オリジナルのURL（デコード前）でリクエスト！
    let response = reqwest::get(&req.url).await?;  // ← url_to_validate じゃない！
    Ok(response.text().await?)
}
```

検証に使うURLとリクエストに使うURLが一致していないと、検証をバイパスできる。「便利だから」と入力を加工するときは、必ず加工後の値を一貫して使うこと。

## その他のデモ：実装詳細解説

残り5つのデモも、それぞれ重要なセキュリティ概念を実装している。ここでは各脆弱性の背景と、なぜ見落としやすいのかを詳しく解説する。

### broken_auth: JWT検証の落とし穴（API2: Broken Authentication）

「署名さえ正しければOK」という誤解を打ち砕くデモ。

https://owasp.org/API-Security/editions/2023/en/0xa2-broken-authentication/

[https://owasp.org/API-Security/editions/2023/en/0xa2-broken-authentication/:embed:cite]

#### なぜJWT検証で失敗するのか

JWTは「署名で改ざんを検出できる」という特性から、安全だと誤解されやすい。しかし、JWTのセキュリティは署名検証だけでは不十分だ。以下の検証が**すべて**必要：

| 検証項目 | 何をチェック？ | 省略するとどうなる？ |
|---------|--------------|-------------------|
| 署名 (`signature`) | トークンが改ざんされていないか | 偽造トークンが通る |
| 有効期限 (`exp`) | トークンが期限内か | 永久に使えるトークンが発生 |
| 発行者 (`iss`) | 正当な発行者が作ったか | 他システムのトークンが通る |
| オーディエンス (`aud`) | このAPIで使うべきか | 別サービスのトークンが通る |
| Not Before (`nbf`) | まだ使用開始前ではないか | 未来のトークンが先に使える |

#### JWTに関する危険な誤解

1. **「署名が正しければ安全」** → 署名は「改ざんされていない」だけで「使っていい」は別の話
2. **「JWTライブラリを使えば安全」** → デフォルト設定が安全とは限らない
3. **「短い有効期限だから大丈夫」** → `exp`チェックを無効にしていたら意味がない
4. **「リフレッシュトークンで更新するから」** → 古いアクセストークンが使えたら問題

```bash
cargo run --release --bin broken-auth-demo
```

#### 脆弱な実装：署名以外を検証しない

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

#### 安全な実装：全クレームを検証

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

#### テスト用トークン生成

このデモでは4種類のトークンを生成できる：

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

攻撃シナリオ：
```bash
# 期限切れトークンを取得
EXPIRED=$(curl -s http://localhost:8080/token/expired | jq -r .access_token)

# 脆弱なエンドポイント → 通る！
curl -H "Authorization: Bearer $EXPIRED" http://localhost:8080/vulnerable/validate

# 安全なエンドポイント → 401 Unauthorized
curl -H "Authorization: Bearer $EXPIRED" http://localhost:8080/validate
```

#### 微妙な脆弱性：JWT検証の巧妙なバイパス

「全クレームを検証しているから安全」と思っていないだろうか。残念ながら、JWT検証にはもっと狡猾な落とし穴がある。

##### 微妙な脆弱性 #1: アルゴリズム混同攻撃

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

##### 微妙な脆弱性 #2: Key ID (kid) インジェクション

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

##### 微妙な脆弱性 #3: JKU (JWK Set URL) バイパス

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

##### 微妙な脆弱性 #4: Not-Before (nbf) 未検証

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

### rate_limit: 総当たり攻撃対策（API4: Unrestricted Resource Consumption）

パスワードクラッキングの現実を体験できるデモ。

https://owasp.org/API-Security/editions/2023/en/0xa4-unrestricted-resource-consumption/

[https://owasp.org/API-Security/editions/2023/en/0xa4-unrestricted-resource-consumption/:embed:cite]

#### なぜレート制限が重要なのか

レート制限がないAPIは「無限に試行できる」ことを意味する：

| 攻撃手法 | 被害 | レート制限での防御 |
|---------|------|------------------|
| パスワード総当たり | アカウント乗っ取り | 試行回数制限 |
| クレデンシャルスタッフィング | 流出パスワードでの不正ログイン | IPベースのブロック |
| OTPブルートフォース | 2FA/SMS認証のバイパス | アカウントロック |
| APIの過剰呼び出し | サービス停止（DoS） | グローバルレート制限 |
| スクレイピング | データの大量取得 | リクエスト間隔の強制 |

#### パスワードクラッキングの数学

4桁のPINコードを総当たりする場合：
- 組み合わせ: 10^4 = 10,000通り
- 毎秒10回の試行 → 約17分で全組み合わせを試行
- **レート制限なし** → 毎秒1000回で10秒

8文字のパスワード（小文字+数字）：
- 組み合わせ: 36^8 ≒ 2.8兆通り
- 毎秒1000回でも約89年かかる
- **でも**、辞書攻撃なら数万語 → 数分で完了

レート制限は「総当たりを現実的に不可能にする」ための防御だ。

```bash
cargo run --release --bin rate-limit-demo
```

#### 二層の防御：IP追跡とアカウント追跡

```rust
/// Tracks login attempts per IP address
#[derive(Debug, Clone)]
struct LoginAttemptTracker {
    /// IP -> (attempt_count, first_attempt_time)
    ip_attempts: Arc<RwLock<HashMap<String, (u32, Instant)>>>,
    /// Email -> (attempt_count, first_attempt_time)
    account_attempts: Arc<RwLock<HashMap<String, (u32, Instant)>>>,
    /// Blocked IPs
    blocked_ips: Arc<RwLock<Vec<String>>>,
    /// Locked accounts
    locked_accounts: Arc<RwLock<Vec<String>>>,
}
```

なぜ二層必要か？

- **IP追跡のみ**だと、攻撃者がVPNやTorでIP変えながら攻撃できる
- **アカウント追跡のみ**だと、1つのIPから多数のアカウントを攻撃できる
- **両方**で、どちらのパターンも防げる

#### スライディングウィンドウの実装

```rust
fn record_attempt(&self, ip: &str, email: &str) -> (u32, u32) {
    let window = Duration::from_secs(300); // 5分間のウィンドウ
    let now = Instant::now();

    // Track IP attempts
    let ip_count = {
        let mut attempts = self.ip_attempts.write().unwrap();
        let entry = attempts.entry(ip.to_string()).or_insert((0, now));
        if now.duration_since(entry.1) > window {
            // 5分経過したらリセット
            *entry = (1, now);
        } else {
            entry.0 += 1;
        }
        entry.0
    };

    // Block IP after 10 attempts
    if ip_count >= 10 {
        let mut blocked = self.blocked_ips.write().unwrap();
        if !blocked.contains(&ip.to_string()) {
            blocked.push(ip.to_string());
            tracing::warn!(ip = ip, "IP blocked due to too many attempts");
        }
    }

    // Lock account after 5 attempts
    if account_count >= 5 {
        // ...
    }

    (ip_count, account_count)
}
```

#### governorクレートによるグローバルレート制限

```rust
// Global rate limiter: 10 requests per second
let rate_limiter = Arc::new(RateLimiter::direct(Quota::per_second(
    NonZeroU32::new(10).unwrap(),
)));
```

`governor`はトークンバケットアルゴリズムを実装している。バケットに毎秒10トークン補充され、リクエストごとに1トークン消費。バケットが空になったら429を返す。

#### 脆弱 vs 安全

```rust
/// VULNERABLE: Login endpoint without rate limiting
async fn vulnerable_login(Json(req): Json<LoginRequest>) -> Result<Json<LoginResponse>, AppError> {
    // 何回でも試行可能！
    if req.email == "user@example.com" && req.password == "password123" {
        Ok(Json(LoginResponse { /* ... */ }))
    } else {
        Err(AppError::Unauthorized)
    }
}

/// SECURE: Login endpoint with rate limiting and lockout
async fn secure_login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<RateLimitError>)> {
    let ip = addr.ip().to_string();

    // 1. グローバルレート制限
    if state.rate_limiter.check().is_err() {
        return Err((StatusCode::TOO_MANY_REQUESTS, /* ... */));
    }

    // 2. IPブロック確認
    if state.tracker.is_ip_blocked(&ip) {
        return Err((StatusCode::TOO_MANY_REQUESTS, /* ... */));
    }

    // 3. アカウントロック確認
    if state.tracker.is_account_locked(&req.email) {
        return Err((StatusCode::TOO_MANY_REQUESTS, /* ... */));
    }

    // 4. 認証処理
    if req.email == "user@example.com" && req.password == "password123" {
        state.tracker.reset_on_success(&ip, &req.email); // 成功したらカウンターリセット
        Ok(Json(LoginResponse { /* ... */ }))
    } else {
        state.tracker.record_attempt(&ip, &req.email); // 失敗を記録
        Err((StatusCode::UNAUTHORIZED, /* ... */))
    }
}
```

#### 微妙な脆弱性：レート制限のバイパス手法

「レート制限を実装したから安全」と思っていないだろうか。残念ながら、レート制限にもバイパス手法がたくさんある。

##### 微妙な脆弱性 #1: X-Forwarded-Forを信用する

```rust
/// 開発者の意図: 「ロードバランサーの後ろにいるから、X-Forwarded-Forを使わないと」
/// 現実: 攻撃者もX-Forwarded-Forを設定できる
async fn subtle_xff_bypass(headers: HeaderMap, ...) -> Result<...> {
    // BUG: X-Forwarded-Forを無条件に信用
    let ip = headers
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| addr.ip().to_string());

    // 攻撃: curl -H "X-Forwarded-For: 1.2.3.4" ...
    //       curl -H "X-Forwarded-For: 5.6.7.8" ...
    // 毎回違うIPとしてカウントされる！
    if state.tracker.is_ip_blocked(&ip) { /* ... */ }
}
```

X-Forwarded-Forは信頼できるプロキシからのみ受け入れるべき。信頼チェーンを確立せずにXFFを使うと、攻撃者がIPを自由に偽装できる。

##### 微妙な脆弱性 #2: 大文字小文字の不一致

```rust
/// 開発者の意図: 「メールアドレスでアカウントロックを追跡」
/// 現実: 大文字小文字で別アカウント扱い
async fn subtle_case_sensitivity(...) -> Result<...> {
    // BUG: アカウントロックは大文字小文字を区別
    if state.tracker.is_account_locked(&req.email) {
        return Err(...);
    }

    // でも認証は大文字小文字を無視
    let email_lower = req.email.to_lowercase();
    if email_lower == "user@example.com" && req.password == "password123" {
        // ...
    }

    // 攻撃:
    // user@example.com で5回失敗 → ロック
    // User@example.com で5回失敗 → 別カウント！
    // USER@example.com で5回失敗 → また別カウント！
    // 結果: 15回試行できる
}
```

アカウント識別子の正規化を一貫して行わないと、レート制限を回避される。

##### 微妙な脆弱性 #3: タイミングリーク

```rust
/// 開発者の意図: 「ロックされたアカウントは早期リターン」
/// 現実: レスポンス時間でアカウントの存在がわかる
async fn subtle_timing_leak(...) -> Result<...> {
    // ロック済みアカウントは即座に拒否（速い！）
    if state.tracker.is_account_locked(&req.email) {
        return Err(/* 数マイクロ秒 */);
    }

    // パスワードハッシュ検証（遅い！）
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 存在するアカウントは追加処理（もっと遅い！）
    if account_exists(&req.email) {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // 攻撃: レスポンス時間を測定
    // 即座に返る → ロック済み（= 存在するアカウント）
    // 100ms → 存在しないアカウント
    // 150ms → 存在するが間違ったパスワード
}
```

レスポンス時間を均一にしないと、アカウント列挙攻撃に使われる。

##### 微妙な脆弱性 #4: TOCTOU競合

```rust
/// 開発者の意図: 「カウンターを確認してから処理」
/// 現実: 確認と更新の間に別のリクエストが入る
async fn subtle_race_condition(...) -> Result<...> {
    // Step 1: カウンター読み取り（ロック解放）
    let current_count = {
        let attempts = state.tracker.ip_attempts.read().unwrap();
        attempts.get(&ip).map(|(count, _)| *count).unwrap_or(0)
    }; // ← ここでロック解放

    // この間に並行リクエストが！
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Step 2: 制限チェック（古い値で判断）
    if current_count >= 10 {
        return Err(...);
    }

    // Step 3: 処理後にカウンター更新
    state.tracker.record_attempt(&ip, &req.email);

    // 攻撃: 100並行リクエストを同時送信
    // 全員が current_count = 0 で通過！
}
```

チェックと更新はアトミックに行うべき。`RwLock`ではなくアトミック操作や、チェックと更新を1つのロック内で行う必要がある。

### jwt: JWTの仕組みを理解する

HS256とRS256の違い、そしてなぜRS256が推奨されるかを学ぶデモ。

```bash
cargo run --release --bin jwt-demo
```

#### HS256（対称鍵）vs RS256（非対称鍵）

```rust
// HS256: 同じ鍵で署名と検証
const HS256_SECRET: &str = "your-256-bit-secret-key-here-must-be-long-enough";

// RS256: 秘密鍵で署名、公開鍵で検証
const RS256_PRIVATE_KEY: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASC...
-----END PRIVATE KEY-----"#;

const RS256_PUBLIC_KEY: &str = r#"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8A...
-----END PUBLIC KEY-----"#;
```

#### トークン生成の実装

```rust
async fn generate_token_for_user(
    Path((algorithm, user_id)): Path<(String, String)>,
) -> Result<Json<TokenResponse>, StatusCode> {
    let now = Utc::now();
    let expires_in = 3600; // 1 hour

    let claims = Claims {
        sub: user_id,
        exp: (now + Duration::seconds(expires_in)).timestamp() as usize,
        iat: now.timestamp() as usize,
        iss: ISSUER.to_string(),
        aud: AUDIENCE.to_string(),
        permissions: vec!["read".to_string(), "write".to_string()],
        email: Some("user@example.com".to_string()),
    };

    let (token, alg_name) = match algorithm.to_lowercase().as_str() {
        "hs256" => {
            let header = Header::new(Algorithm::HS256);
            let token = encode(
                &header,
                &claims,
                &EncodingKey::from_secret(HS256_SECRET.as_bytes()),
            )?;
            (token, "HS256")
        }
        "rs256" => {
            let header = Header::new(Algorithm::RS256);
            let token = encode(
                &header,
                &claims,
                &EncodingKey::from_rsa_pem(RS256_PRIVATE_KEY.as_bytes())?,
            )?;
            (token, "RS256")
        }
        _ => return Err(StatusCode::BAD_REQUEST),
    };
    // ...
}
```

#### なぜRS256が推奨されるか

```rust
async fn list_algorithms() -> Json<AlgorithmsResponse> {
    Json(AlgorithmsResponse {
        supported: vec![
            AlgorithmInfo {
                name: "HS256".to_string(),
                description: "HMAC using SHA-256 (symmetric)".to_string(),
                key_type: "Shared secret".to_string(),
            },
            AlgorithmInfo {
                name: "RS256".to_string(),
                description: "RSA using SHA-256 (asymmetric)".to_string(),
                key_type: "RSA key pair".to_string(),
            },
        ],
        recommended: "RS256".to_string(),
        note: "RS256 is recommended for production as it allows public key distribution \
               without exposing the signing key".to_string(),
    })
}
```

- **HS256**: 署名と検証に同じ鍵を使う → 検証側にも秘密鍵が必要 → 漏洩リスク高
- **RS256**: 署名に秘密鍵、検証に公開鍵 → 公開鍵は配布しても安全 → マイクロサービス向き

### observability: 攻撃検知システム

攻撃を「防ぐ」だけでなく「検知する」ことの重要性を示すデモ。

```bash
cargo run --release --bin observability-demo
```

#### セキュリティメトリクス

```rust
#[derive(Debug, Default)]
struct SecurityMetrics {
    total_requests: AtomicU64,
    failed_auth_attempts: AtomicU64,
    suspicious_requests: AtomicU64,
    blocked_requests: AtomicU64,
    sql_injection_attempts: AtomicU64,
    xss_attempts: AtomicU64,
}
```

これらのメトリクスを`/metrics`エンドポイントで取得できる。Prometheus等で収集して、ダッシュボードで監視する想定。

#### 攻撃パターン検知ミドルウェア

```rust
async fn security_logging_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    let query = uri.query().map(|q| q.to_string());
    let client_ip = addr.ip().to_string();

    state.metrics.increment_total();

    let mut security_flags = Vec::new();

    if let Some(ref q) = query {
        // SQL injection patterns
        let sqli_patterns = [
            "'", "\"", "--", ";", "union", "select", "drop", "insert", "delete",
        ];
        let q_lower = q.to_lowercase();
        for pattern in sqli_patterns {
            if q_lower.contains(pattern) {
                security_flags.push("potential_sqli");
                state.metrics.increment_sqli();
                break;
            }
        }

        // XSS patterns
        let xss_patterns = ["<script", "javascript:", "onerror", "onload", "onclick"];
        for pattern in xss_patterns {
            if q_lower.contains(pattern) {
                security_flags.push("potential_xss");
                state.metrics.increment_xss();
                break;
            }
        }
    }

    if !security_flags.is_empty() {
        state.metrics.increment_suspicious();
        warn!(
            target: "security",
            event = "suspicious_request",
            client_ip = %client_ip,
            method = %method,
            path = %uri.path(),
            query = ?query,
            flags = ?security_flags,
            "Suspicious request detected"
        );
    }

    // Process request and log result
    let response = next.run(request).await;
    let status = response.status().as_u16();

    // Track auth failures
    if status == 401 || status == 403 {
        state.metrics.increment_failed_auth();
        warn!(
            target: "security",
            event = "auth_failure",
            client_ip = %client_ip,
            path = %uri.path(),
            status = status,
            "Authentication/authorization failure"
        );
    }

    response
}
```

攻撃を検知したときのログ出力（JSON形式）：
```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "level": "WARN",
  "target": "security",
  "event": "suspicious_request",
  "client_ip": "192.168.1.100",
  "method": "GET",
  "path": "/api/data",
  "query": "id=1' OR 1=1--",
  "flags": ["potential_sqli"],
  "message": "Suspicious request detected"
}
```

#### 構造化ログの重要性

```rust
tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer().json())  // JSON形式で出力
    .init();
```

JSON形式にすることで：
- Elasticsearch、Loki等のログ基盤に取り込みやすい
- フィルタリング・集計が容易
- アラート設定が簡単（「sql_injection_attempts > 10/分」でSlack通知など）

### security_test: 自動セキュリティテスト

脆弱性の有無を自動的にテストするデモ。CI/CDに組み込むイメージ。

```bash
cargo run --release --bin security-test-demo
curl http://localhost:8080/test/run-all
```

#### テスト結果の構造

```rust
#[derive(Serialize)]
struct TestResults {
    total: usize,
    passed: usize,
    failed: usize,
    tests: Vec<TestResult>,
}

#[derive(Serialize)]
struct TestResult {
    name: String,
    description: String,
    passed: bool,
    vulnerable_endpoint: String,
    secure_endpoint: String,
    details: String,
}
```

#### 各テスト項目

```rust
async fn run_security_tests(State(state): State<AppState>) -> Json<TestResults> {
    let mut results = Vec::new();

    // Test 1: Excessive Data Exposure
    results.push(test_data_exposure(&state).await);

    // Test 2: Input Validation
    results.push(test_input_validation(&state).await);

    // Test 3: SQL Injection Pattern
    results.push(test_sql_injection_pattern(&state).await);

    // Test 4: Internal Data Exposure
    results.push(test_internal_data_exposure(&state).await);

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;

    Json(TestResults {
        total: results.len(),
        passed,
        failed,
        tests: results,
    })
}
```

#### 入力検証テストの実装

```rust
/// SECURE: Input validation and filtered response
async fn secure_get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<UserPublic>, (StatusCode, String)> {
    // SECURE: Validate input format
    if !id.chars().all(|c| c.is_ascii_digit()) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid user ID format".to_string(),
        ));
    }

    let id: i64 = id
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".to_string()))?;

    // SECURE: Validate range
    if !(1..=1_000_000).contains(&id) {
        return Err((StatusCode::BAD_REQUEST, "User ID out of range".to_string()));
    }

    // ...
}
```

#### 検索エンドポイントの入力検証

```rust
/// SECURE: Parameterized search (simulated)
async fn secure_search(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<SearchResponse>, (StatusCode, String)> {
    let query = params.get("q").cloned().unwrap_or_default();

    // SECURE: Validate and sanitize input
    if query.len() > 100 {
        return Err((StatusCode::BAD_REQUEST, "Query too long".to_string()));
    }

    // SECURE: Only allow alphanumeric and common characters
    if !query
        .chars()
        .all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' || c == '@' || c == '.')
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid characters in query".to_string(),
        ));
    }

    // ...
}
```

`' OR 1=1--`のようなSQLインジェクションパターンは、`'`が許可文字リストにないため弾かれる

## 全テストの実行

20のセキュリティテストを一括で実行できる：

```bash
./scripts/test_all.sh
```

```
==========================================
API Security Demo - Vulnerability Tests
OWASP API Security Top 10
==========================================

[PASS] Vulnerable EP: Bob accessed Alice's order (HTTP 200)  ← 攻撃成功！（喜ぶな）
[PASS] Secure EP: Access denied (HTTP 404)                   ← 攻撃失敗！（喜べ）
...
==========================================
Test Results Summary
==========================================
PASS: 20
FAIL: 0

All security tests passed!
```

「脆弱なエンドポイントで攻撃が成功すること」と「安全なエンドポイントで攻撃が失敗すること」の両方をテストしている。「攻撃が成功してPASS」というのは変な感じがするが、これは「脆弱性のデモとして正しく動作している」ことの確認だ。

## 実装で学んだこと（血と汗と涙の記録）

### 1. 認証と認可は別物（100回言う）

これは何度言っても足りない。声を大にして言いたい。壁に貼っておきたい。Tシャツにプリントしたい。

- 認証: 「あなたは誰？」 → 「私はBobです」
- 認可: 「Bobさん、あなたはこれをしていいの？」 → 「...ダメです」

JWTを検証して「このユーザーは本物だ」とわかっても、「このユーザーがこのリソースにアクセスしていいか」は全く別の問題だ。

会社のビルで例えると：
- 認証 = 社員証を見せて入館する
- 認可 = サーバールームに入れるかどうか

社員証を持っていても、全員がサーバールームに入れるわけではない。当たり前だ。でも、APIでは「認証してるから大丈夫」と言ってしまいがちなのだ。

### 2. 404 vs 403（嘘つきは泥棒の始まり...ではない）

認可エラーの際に403を返すか404を返すか。これは「正直者」か「用心深い人」かの違いだ。

- 403: 「そのリソースは存在するよ。でも、あなたには見せない」（正直）
- 404: 「何それ？知らない」（嘘つき、でも賢い）

セキュリティ的には404が安全だ。403は「存在する」という情報を漏らしている。攻撃者は「存在する」とわかれば、別の方法でアクセスしようとするかもしれない。

でも、デバッグは地獄になる。「404なんだけど、本当に存在しないの？それとも権限がないの？」がわからない。本番環境では404、開発環境では403にするとか、ログには詳細を残すとか、工夫が必要だ。

### 3. DTOの分離は面倒だが必要（筋トレと同じ）

入力用の構造体と内部用の構造体を分けるのは、正直面倒だ。「同じようなものを2回書くの？」と思う。思う気持ちはわかる。

でも、Mass Assignment攻撃を防ぐには必要なコストだ。ジムに行くのが面倒でも、健康のためには必要なのと同じ。

Rustの場合、コンパイル時に型チェックされるので、「うっかりユーザー入力をそのまま使ってしまう」ミスは起きにくい。`CreatePaymentRequest`に`status`フィールドがなければ、コンパイラが「そんなフィールドないよ」と教えてくれる。これはRustの強みだ。動的型付け言語だと、こうはいかない。

## 動作確認：実際に脆弱性を突いてみる

理論だけでは実感が湧かない。実際にcurlでリクエストを投げて、脆弱性が動作することを確認してみよう。

### BOLA（API1）の動作確認

```bash
# サーバー起動
cargo run --release --bin bola-demo

# Bobのトークンを取得
BOB_TOKEN=$(curl -s http://localhost:8080/token/bob | jq -r .access_token)

# 脆弱なエンドポイント：BobがAliceの注文を見れてしまう
curl -H "Authorization: Bearer $BOB_TOKEN" http://localhost:8080/vulnerable/orders/1
# 結果: {"id":1,"user_id":"alice","product":"Widget A","amount":100,...}
# → BobがAliceの注文情報を取得できた！

# セキュアなエンドポイント：適切に拒否される
curl -H "Authorization: Bearer $BOB_TOKEN" http://localhost:8080/orders/1
# 結果: {"error":"Order 1 not found or access denied"}

# Subtle脆弱性：クエリパラメータでuser_idを上書き
curl -H "Authorization: Bearer $BOB_TOKEN" "http://localhost:8080/subtle/orders/1?user_id=alice"
# 結果: {"id":1,"user_id":"alice","product":"Widget A",...}
# → クエリパラメータでオーナーチェックをバイパス！
```

### Mass Assignment（API3）の動作確認

```bash
# サーバー起動
cargo run --release --bin mass-assignment-demo

# 脆弱なエンドポイント：statusを注入
curl -X POST http://localhost:8080/vulnerable/payments \
  -H "Content-Type: application/json" \
  -d '{"user_id":"attacker","amount":1000,"status":"approved"}'
# 結果: {"id":"...","user_id":"attacker","amount":1000,"status":"approved",...}
# → 攻撃者がstatusを"approved"に設定できた！

# セキュアなエンドポイント：statusは無視される
curl -X POST http://localhost:8080/payments \
  -H "Content-Type: application/json" \
  -d '{"user_id":"user","amount":1000,"status":"approved"}'
# 結果: {"id":"...","user_id":"user","amount":1000,"status":"pending",...}
# → statusはサーバー側で"pending"に設定される

# Subtle脆弱性：serde(flatten)でHashMapに余分なフィールドが入る
curl -X POST http://localhost:8080/subtle/payments/flatten \
  -H "Content-Type: application/json" \
  -d '{"user_id":"user","amount":500,"status":"approved","id":"my-custom-id"}'
# 結果: statusが"approved"、idも上書きされる可能性
# → flatten + HashMapの危険性
```

### BFLA（API5）の動作確認

```bash
# サーバー起動
cargo run --release --bin bfla-demo

# 一般ユーザーのトークンを取得
USER_TOKEN=$(curl -s http://localhost:8080/token/user | jq -r .access_token)

# 脆弱なエンドポイント：一般ユーザーでも管理者機能にアクセス
curl -H "Authorization: Bearer $USER_TOKEN" http://localhost:8080/vulnerable/admin
# 結果: {"message":"Welcome to admin panel","admin_data":{"total_revenue":567890.12,...}}
# → 一般ユーザーが管理者データを取得！

# セキュアなエンドポイント：適切に拒否
curl -H "Authorization: Bearer $USER_TOKEN" http://localhost:8080/admin
# 結果: {"error":"Admin permission required"}

# Subtle脆弱性1：HTTPヘッダーのロールを信頼
curl -H "Authorization: Bearer $USER_TOKEN" \
     -H "X-User-Role: admin" \
     http://localhost:8080/subtle/admin/role-in-header
# 結果: アクセス成功！
# → ヘッダーを追加するだけでadminになれる

# Subtle脆弱性2：キャッシュされた権限チェックを信頼
curl -H "Authorization: Bearer $USER_TOKEN" \
     "http://localhost:8080/subtle/admin/cached-check?permission_verified=true"
# 結果: アクセス成功！
# → クエリパラメータで権限チェックをバイパス
```

### SSRF（API7）の動作確認

```bash
# サーバー起動
cargo run --release --bin ssrf-demo

# 脆弱なエンドポイント：内部サービスにアクセス
curl "http://localhost:8080/vulnerable/fetch?url=http://localhost:8080/internal/secrets"
# 結果: {"secrets":["DATABASE_URL=postgres://admin:password@db:5432",...]}
# → 内部の機密情報を取得！

# セキュアなエンドポイント：localhost は拒否
curl "http://localhost:8080/fetch?url=http://localhost:8080/internal/secrets"
# 結果: {"error":"Access to internal addresses is not allowed"}

# Subtle脆弱性：URLパーサーの差異を悪用
curl "http://localhost:8080/subtle/fetch/parser-diff?url=http://localhost%2523@evil.com/"
# → 異なるパーサーで解釈が変わり、バイパス可能
```

### Rate Limit（API4）の動作確認

```bash
# サーバー起動
cargo run --release --bin rate-limit-demo

# 正常なレート制限：5回でロック
for i in {1..6}; do
  curl -X POST http://localhost:8080/login \
    -H "Content-Type: application/json" \
    -d '{"email":"test@example.com","password":"wrong"}'
  echo ""
done
# 6回目: {"error":"Account locked. Too many failed attempts."}

# Subtle脆弱性1：X-Forwarded-For でIPを偽装
for i in {1..10}; do
  curl -X POST http://localhost:8080/subtle/login/xff \
    -H "Content-Type: application/json" \
    -H "X-Forwarded-For: 10.0.0.$i" \
    -d '{"email":"victim@example.com","password":"attempt$i"}'
done
# → 毎回異なるIPとしてカウントされ、ロックされない！

# Subtle脆弱性2：メールアドレスの大文字小文字
curl -X POST http://localhost:8080/subtle/login/case \
  -H "Content-Type: application/json" \
  -d '{"email":"User@Example.COM","password":"wrong"}'
# → user@example.com とは別のエントリとしてカウント

# Subtle脆弱性3：タイミング攻撃
# 存在するユーザー（高速レスポンス）
time curl -X POST http://localhost:8080/subtle/login/timing \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"x"}'
# → ~10ms

# 存在しないユーザー（遅いレスポンス）
time curl -X POST http://localhost:8080/subtle/login/timing \
  -H "Content-Type: application/json" \
  -d '{"email":"nobody@example.com","password":"x"}'
# → ~110ms（意図的な遅延）
# → レスポンス時間の差でユーザーの存在を推測可能！
```

### Broken Auth（API2）の動作確認

```bash
# サーバー起動
cargo run --release --bin broken-auth-demo

# 期限切れトークンを取得
EXPIRED_TOKEN=$(curl -s http://localhost:8080/token/expired | jq -r .access_token)

# 脆弱なエンドポイント：期限切れトークンを受け入れる
curl -H "Authorization: Bearer $EXPIRED_TOKEN" \
     http://localhost:8080/vulnerable/validate
# 結果: {"message":"Token accepted","token_type":"expired"}
# → 期限切れなのにアクセス成功！

# セキュアなエンドポイント：適切に拒否
curl -H "Authorization: Bearer $EXPIRED_TOKEN" \
     http://localhost:8080/validate
# 結果: {"error":"Token validation failed: ExpiredSignature"}

# Subtle脆弱性：nbf（not before）をスキップ
FUTURE_TOKEN=$(curl -s http://localhost:8080/token/future | jq -r .access_token)
curl -H "Authorization: Bearer $FUTURE_TOKEN" \
     http://localhost:8080/subtle/validate/nbf-skip
# 結果: まだ有効期間前なのにアクセス成功
# → nbfのチェック漏れ
```

### 動作確認のポイント

これらのテストで確認できる重要な点：

1. **脆弱なエンドポイント vs セキュアなエンドポイント**
   - 同じリクエストでも、実装によって結果が全く異なる
   - セキュアな実装は「デフォルト拒否」の原則に従う

2. **Subtle脆弱性の危険性**
   - コードを見ただけでは問題に気づきにくい
   - 「動いているから大丈夫」では見逃す
   - セキュリティテストで初めて発覚することが多い

3. **攻撃者の視点**
   - 攻撃者は正常系だけでなく、エッジケースを狙う
   - ヘッダー追加、大文字小文字変換、URL エンコードなど
   - 「そんなリクエスト来ないでしょ」は通用しない

## まとめ

セキュリティは「知っている」と「実感している」の間に大きな溝がある。

このデモを作って、自分で攻撃を試して、初めて「あ、これ確かにヤバい」と腑に落ちた。ドキュメントを読むだけでは得られない理解だったと思う。そして、自分の過去のコードを見直すきっかけにもなった（いくつか冷や汗案件があった。詳細は言えない）。

コードは[GitHub](https://github.com/nwiizo/workspace_2025/tree/main/infrastructure/api-security-demo)で公開している。`cargo run --release --bin bola-demo`で起動して、実際に攻撃を試してみてほしい。BobになってAliceのデータを覗く背徳感を味わってほしい。そして、その後で「これ、本番でやられたらヤバいな」と思ってほしい。

最後に、冒頭の話に戻る。「認証してるから大丈夫でしょ」—この言葉を聞いたら、このデモのことを思い出してほしい。そして、穏やかに、でも断固として、「認可は？」と聞き返してほしい。

認証は玄関のチェックに過ぎない。中に入った後、どの部屋に入れるかを制御するのが認可だ。その違いを、コードで、手を動かして、体感してほしい。そして、二度と「認証してるから大丈夫」とは言わないでほしい。

...言わないよね？

## 参考リンク

### OWASP API Security Top 10 (2023)

公式。全エンジニア必読。

https://owasp.org/API-Security/editions/2023/en/0x11-t10/

[https://owasp.org/API-Security/editions/2023/en/0x11-t10/:embed:cite]

### OWASP API Security Project

プロジェクトのホームページ

https://owasp.org/www-project-api-security/

[https://owasp.org/www-project-api-security/:embed:cite]

### 本記事のソースコード

実際に動かしてみてほしい

https://github.com/nwiizo/workspace_2025/tree/main/infrastructure/api-security-demo

[https://github.com/nwiizo/workspace_2025/tree/main/infrastructure/api-security-demo:embed:cite]

### Alice and Bob - Wikipedia

BobとAliceの歴史をもっと知りたい人向け

https://en.wikipedia.org/wiki/Alice_and_Bob

[https://en.wikipedia.org/wiki/Alice_and_Bob:embed:cite]
