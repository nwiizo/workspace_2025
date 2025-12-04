# RustでOWASP API Security Top 10を体験する（後編）：リソース制御と攻撃検知

**[前編](./blog_01.md)からの続き** ← API1 (BOLA), API2 (Broken Authentication), API3 (Mass Assignment)の解説はこちら

---

前編では認証・認可の基礎とデータ保護について解説した。後編では、リソース消費制御、機能レベルの認可、そしてサーバーサイド攻撃について体験していく。

## API4: Rate Limit - 総当たり攻撃対策

パスワードクラッキングの現実を体験できるデモ。

[https://owasp.org/API-Security/editions/2023/en/0xa4-unrestricted-resource-consumption/:embed:cite]

### なぜレート制限が重要なのか

レート制限がないAPIは「無限に試行できる」ことを意味する。

| 攻撃手法 | 被害 | レート制限での防御 |
|---------|------|------------------|
| パスワード総当たり | アカウント乗っ取り | 試行回数制限 |
| クレデンシャルスタッフィング | 流出パスワードでの不正ログイン | IPベースのブロック |
| OTPブルートフォース | 2FA/SMS認証のバイパス | アカウントロック |
| APIの過剰呼び出し | サービス停止（DoS） | グローバルレート制限 |
| スクレイピング | データの大量取得 | リクエスト間隔の強制 |

### パスワードクラッキングの数学

4桁のPINコードを総当たりする時は以下のようになる。
- 組み合わせ: 10^4 = 10,000通り
- 毎秒10回の試行 → 約17分で全組み合わせを試行
- **レート制限なし** → 毎秒1000回で10秒

8文字のパスワード（小文字+数字）の時は以下のようになる。
- 組み合わせ: 36^8 ≒ 2.8兆通り
- 毎秒1000回でも約89年かかる
- **でも**、辞書攻撃なら数万語 → 数分で完了

レート制限は「総当たりを現実的に不可能にする」ための防御だ。

```bash
cargo run --release --bin rate-limit-demo
```

### 二層の防御：IP追跡とアカウント追跡

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

なぜ二層必要なのか。

- **IP追跡のみ**だと、攻撃者がVPNやTorでIP変えながら攻撃できる
- **アカウント追跡のみ**だと、1つのIPから多数のアカウントを攻撃できる
- **両方**で、どちらのパターンも防げる

### スライディングウィンドウの実装

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

### governorクレートによるグローバルレート制限

```rust
// Global rate limiter: 10 requests per second
let rate_limiter = Arc::new(RateLimiter::direct(Quota::per_second(
    NonZeroU32::new(10).unwrap(),
)));
```

`governor`はトークンバケットアルゴリズムを実装している。バケットに毎秒10トークン補充され、リクエストごとに1トークン消費。バケットが空になったら429を返す。

### 脆弱 vs 安全

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

### 微妙な脆弱性：レート制限のバイパス手法

「レート制限を実装したから安全」と思っていないだろうか。残念ながら、レート制限にもバイパス手法がたくさんある。

#### 微妙な脆弱性 #1: X-Forwarded-Forを信用する

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

#### 微妙な脆弱性 #2: 大文字小文字の不一致

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

#### 微妙な脆弱性 #3: タイミングリーク

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

#### 微妙な脆弱性 #4: TOCTOU競合

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

---

## API5: BFLA - 一般ユーザーが管理者になれてしまう問題

BFLA（Broken Function Level Authorization）は、BOLA（前編で解説）の「機能版」だ。

[https://owasp.org/API-Security/editions/2023/en/0xa5-broken-function-level-authorization/:embed:cite]

BOLAが「他人のデータを見られる」なら、BFLAは「使えないはずの機能が使える」。例えば、一般ユーザーが管理者用のユーザー一覧APIを叩けてしまうケース。言ってみれば「平社員が社長の権限でシステムを操作できる」状態だ。

### BOLAとBFLAの違いを理解する

この2つは混同しやすいので、明確に区別しよう。

| 項目 | BOLA | BFLA |
|------|------|------|
| 何が壊れているか | オブジェクト（データ）へのアクセス制御 | 機能（エンドポイント）へのアクセス制御 |
| 攻撃例 | BobがAliceの注文を見る | 一般ユーザーが管理者APIを叩く |
| チェック対象 | 「このデータは誰のものか」 | 「この機能は誰が使えるか」 |
| 典型的な対策 | リソースごとの所有者チェック | ロール/権限チェック |

例えで言えば以下の通りである。

- **BOLA** = 他人のロッカーを開けられる（同じ権限レベル内での越境）
- **BFLA** = 社員証がないのに役員室に入れる（権限レベルの越境）

### なぜBFLAが発生するのか

1. **エンドポイントの「発見」** - `/api/users`があるなら`/api/admin/users`もあるだろうと攻撃者は考える
2. **フロントエンドによる隠蔽への過信** - 「管理メニューは管理者にしか見せてないから大丈夫」→ APIは直接叩ける
3. **認証と認可の混同（再び）** - 「ログインしてるから管理APIも使えるはず」という誤った思い込み
4. **テスト不足** - 管理者機能は管理者アカウントでしかテストしない
5. **ドキュメント化されていない管理API** - 「隠しAPI」は攻撃者に見つかる

### 実際の被害パターン

BFLAによって可能になる攻撃は以下の通りである。

- **ユーザー情報の一括取得** - 全ユーザーのメールアドレス、個人情報を抜き取る
- **権限昇格** - 自分のアカウントに管理者権限を付与する
- **システム設定の変更** - APIキーの再生成、課金設定の変更
- **データの一括削除** - 管理者用の一括削除機能を悪用
- **監査ログの改ざん** - 証拠隠滅のためにログを消去

```rust
/// VULNERABLE: No role check
async fn vulnerable_list_users(user: AuthenticatedUser) -> Result<Json<Vec<UserInfo>>, AppError> {
    Ok(Json(vec![
        UserInfo {
            id: 1,
            email: "admin@example.com".to_string(),
            role: "admin".to_string(),
            ssn: "123-45-6789".to_string(), // SSNまで露出
        },
        // ...
    ]))
}

/// SECURE: Admin check
async fn secure_list_users(user: AuthenticatedUser) -> Result<Json<Vec<SafeUserInfo>>, AppError> {
    if !is_admin(&user.0) {
        return Err(AppError::Forbidden("Admin permission required".to_string()));
    }
    // ...
}
```

`is_admin`のチェックは単純だ。

```rust
pub fn is_admin(claims: &UserClaims) -> bool {
    claims.permissions.iter().any(|p| p == "admin")
}
```

「これくらい誰でも書く」と考えるだろう。しかし、本番環境で「認証は通ってるから大丈夫」と言ってこのチェックを忘れる人が後を絶たない。

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

---

## API7: SSRF - サーバーを踏み台にする

SSRF（Server-Side Request Forgery）は、サーバーに「代わりにリクエストを送らせる」攻撃だ。

[https://owasp.org/API-Security/editions/2023/en/0xa7-server-side-request-forgery/:embed:cite]

問題は、サーバーは内部ネットワークにアクセスできるということだ。外からは見えない場所に、サーバー経由で到達できてしまう。

### SSRFの危険性を理解する

SSRFが特に危険な理由は以下の通りである。

1. **ファイアウォールをバイパス** - 外部からは遮断されていても、内部からのリクエストは通る
2. **クラウドメタデータにアクセス** - AWS/GCPの`169.254.169.254`から認証情報を取得可能
3. **内部サービスの探索** - ポートスキャンや内部APIの発見に悪用
4. **認証のバイパス** - 「内部ネットワークからのアクセスは信頼」という設計を悪用

### クラウド環境での致命的な被害

クラウド環境でのSSRFは特に危険だ。2019年のCapital One事件では、SSRFを使ってAWSのメタデータサービスにアクセスし、1億人以上の顧客データが漏洩した。

攻撃の流れは以下の通りである。

```
1. 攻撃者: http://169.254.169.254/latest/meta-data/iam/security-credentials/ にアクセスさせる
2. サーバー: 内部からのリクエストなので通常通り処理
3. AWSメタデータ: IAMロールの一時認証情報を返す
4. 攻撃者: その認証情報でS3バケットにアクセス → 大量のデータを取得
```

### SSRFが発生しやすい機能

以下のような機能はSSRFの温床になりやすい。

- **URLプレビュー/OGP取得** - 「このURLのタイトルと画像を表示」
- **Webhook送信** - 「指定されたURLにPOSTリクエストを送る」
- **PDF生成** - 「このURLの内容をPDFにする」（ヘッドレスブラウザがURLを開く）
- **画像のリサイズ/変換** - 「このURLの画像をサムネイルにする」
- **インポート機能** - 「このURLからデータをインポート」

例えば、「URLを指定したらそのページの内容を取得する」機能があったとする。

```rust
/// VULNERABLE: Fetches any URL
async fn vulnerable_fetch(Json(req): Json<FetchUrlRequest>) -> Result<String, AppError> {
    let response = reqwest::get(&req.url).await?;
    Ok(response.text().await?)
}
```

攻撃者は内部ネットワークのURLを指定する。
```bash
curl -X POST http://localhost:8080/vulnerable/fetch \
     -d '{"url":"http://localhost:8080/internal/secrets"}'
```

`/internal/secrets` は本来、外部からアクセスできない内部APIだ。しかし、サーバー自身が「localhost」にアクセスするのは許可されている。結果、攻撃者はサーバーを経由して機密情報を引き出す。

サーバーは「言われたことを忠実に実行する」だけだ。それが悪意あるリクエストだとは気づかない。

### 対策: 許可リストとプロトコル制限

```rust
async fn secure_fetch(Json(req): Json<FetchUrlRequest>) -> Result<String, AppError> {
    let url = Url::parse(&req.url)
        .map_err(|_| AppError::BadRequest("Invalid URL".to_string()))?;

    // HTTPSのみ許可
    if url.scheme() != "https" {
        return Err(AppError::BadRequest("Only HTTPS URLs are allowed".to_string()));
    }

    // 許可されたドメインのみ
    let allowed_domains = ["api.example.com", "cdn.example.com"];
    let host = url.host_str()
        .ok_or_else(|| AppError::BadRequest("Invalid host".to_string()))?;

    if !allowed_domains.contains(&host) {
        return Err(AppError::BadRequest("Domain not in allowlist".to_string()));
    }

    // 許可リストを通過したURLのみ処理
    // ...
}
```

「なんでも取ってくる」から「許可されたものだけ取ってくる」へ。自由度は下がるが、セキュリティは上がる。

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

---

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

これらのテストで確認できる重要な点は以下の通りである。

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

---

## 全テストの実行

20のセキュリティテストを一括で実行できる。

```bash
./scripts/test_all.sh
```

```
==========================================
API Security Demo - Vulnerability Tests
OWASP API Security Top 10
==========================================

[PASS] Vulnerable EP: Bob accessed Alice's order (HTTP 200)  ← 攻撃成功
[PASS] Secure EP: Access denied (HTTP 404)                   ← 攻撃失敗
...
==========================================
Test Results Summary
==========================================
PASS: 20
FAIL: 0

All security tests passed!
```

「脆弱なエンドポイントで攻撃が成功すること」と「安全なエンドポイントで攻撃が失敗すること」の両方をテストしている。「攻撃が成功してPASS」というのは変な感じがするが、これは「脆弱性のデモとして正しく動作している」ことの確認だ。

---

## その他のデモ

### observability: 攻撃検知システム

攻撃を「防ぐ」だけでなく「検知する」ことの重要性を示すデモ。

```bash
cargo run --release --bin observability-demo
```

セキュリティメトリクスを収集し、攻撃パターン（SQLインジェクション、XSSなど）を検知してログ出力する。Prometheus等で収集して、ダッシュボードで監視する想定。

### security_test: 自動セキュリティテスト

脆弱性の有無を自動的にテストするデモ。CI/CDに組み込むイメージ。

```bash
cargo run --release --bin security-test-demo
curl http://localhost:8080/test/run-all
```

---

## まとめ

セキュリティは「知っている」と「実感している」の間に大きな溝がある。

このデモを作って、自分で攻撃を試して、初めて「あ、これ確かにヤバい」と腑に落ちた。ドキュメントを読むだけでは得られない理解だった。

コードは[GitHub](https://github.com/nwiizo/workspace_2025/tree/main/infrastructure/api-security-demo)で公開している。`cargo run --release --bin bola-demo`で起動して、実際に攻撃を試してみてほしい。

最後に、冒頭の話に戻る。「認証してるから大丈夫でしょ」—この言葉を聞いたら、このデモのことを思い出してほしい。そして「認可は」と聞き返してほしい。

認証は玄関のチェックに過ぎない。中に入った後、どの部屋に入れるかを制御するのが認可だ。

## 参考リンク

### OWASP API Security Top 10 (2023)

公式ドキュメント。

[https://owasp.org/API-Security/editions/2023/en/0x11-t10/:embed:cite]

### OWASP API Security Project

プロジェクトのホームページ。

[https://owasp.org/www-project-api-security/:embed:cite]

### 本記事のソースコード

[https://github.com/nwiizo/workspace_2025/tree/main/infrastructure/api-security-demo:embed:cite]

### Alice and Bob - Wikipedia

BobとAliceの歴史。

[https://en.wikipedia.org/wiki/Alice_and_Bob:embed:cite]

### governor - Rust Rate Limiting Library

レート制限の実装に使用。

[https://github.com/antifuchs/governor:embed:cite]

### CWE-918: Server-Side Request Forgery (SSRF)

SSRFに関連するCWEエントリ。

[https://cwe.mitre.org/data/definitions/918.html:embed:cite]

### CWE-770: Allocation of Resources Without Limits or Throttling

レート制限不足に関連するCWEエントリ。

[https://cwe.mitre.org/data/definitions/770.html:embed:cite]

### CWE-285: Improper Authorization

BFLAに関連するCWEエントリ。

[https://cwe.mitre.org/data/definitions/285.html:embed:cite]

### PortSwigger - Server-side request forgery (SSRF)

SSRFの詳細な解説とラボ環境。

[https://portswigger.net/web-security/ssrf:embed:cite]

### OWASP Cheat Sheet - Authorization

認可に関するベストプラクティス。

[https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html:embed:cite]

### OWASP Cheat Sheet - Authentication

認証に関するベストプラクティス。

[https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html:embed:cite]

### Capital One Data Breach (2019)

SSRFによる大規模情報漏洩事例。

[https://en.wikipedia.org/wiki/2019_Capital_One_data_breach:embed:cite]

### AWS IMDSv2

AWSメタデータサービスのセキュリティ強化。SSRF対策として重要。

[https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/configuring-instance-metadata-service.html:embed:cite]
