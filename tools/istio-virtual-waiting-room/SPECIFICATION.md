# Istio Virtual Waiting Room System Specification

## Overview

Istio と Rust を使用した仮想待合室システム。限定商品の販売サイトにおける高トラフィック対策として、キュー管理と在庫管理を実装。

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Istio Gateway                             │
│              (Traffic Management / mTLS / Rate Limit)            │
└────────────────────────────┬────────────────────────────────────┘
                             │
           ┌─────────────────┼─────────────────┐
           │                 │                 │
           ▼                 ▼                 ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│ waiting-room-app│ │ main-application│ │   product-api   │
│  (Queue Manager)│ │ (Shop Frontend) │ │ (Stock Manager) │
│    Port: 8080   │ │   Port: 8080    │ │   Port: 8080    │
│   Replicas: 1   │ │   Replicas: 1   │ │   Replicas: 2   │
└─────────────────┘ └────────┬────────┘ └────────┬────────┘
                             │                   │
                             │     ┌─────────────┘
                             │     │
                             ▼     ▼
                    ┌─────────────────────┐
                    │     PostgreSQL      │
                    │  (Shared Database)  │
                    │     Port: 5432      │
                    │   mTLS Required     │
                    └─────────────────────┘
```

## Components

### 1. waiting-room-app (キュー管理)

**機能:**
- FIFO (First In, First Out) キュー管理
- Cookie ベースのユーザー識別
- リアルタイム待ち時間推定
- 自動リダイレクト (JavaScript ポーリング)

**仕様:**

| パラメータ | 値 | 説明 |
|-----------|-----|------|
| MAX_CONCURRENT_USERS | 5 | 同時にショップにアクセスできる最大ユーザー数 |
| SECONDS_PER_USER | 30 | 1ユーザーあたりの推定処理時間 |
| Cookie名 | waiting_room_id | ユーザー識別用UUID |
| ポーリング間隔 | 3秒 | APIステータス確認間隔 |

**キュー動作:**

1. **参加処理 (`join`)**
   - 既にキューに存在する場合: 現在の位置を返す
   - アクティブユーザーの場合: 0を返す (入室済み)
   - 新規ユーザー: キュー末尾に追加

2. **入室判定 (`can_enter`)**
   - アクティブユーザー数 < MAX_CONCURRENT_USERS かつ
   - キューの先頭にいる場合: true
   - それ以外: false

3. **待ち時間推定 (`estimated_wait`)**
   - 計算式: `(position - 1) * SECONDS_PER_USER`
   - 例: 3番目の場合 → (3-1) * 30 = 60秒

**エンドポイント:**

| Method | Path | 説明 |
|--------|------|------|
| GET | `/` | 待合室ページ表示 |
| GET | `/waiting-room` | 待合室ページ表示 (alias) |
| GET | `/api/status` | キュー状態JSON API |
| GET | `/health` | ヘルスチェック |

**キュー状態API レスポンス:**

```json
{
  "position": 3,
  "total_waiting": 5,
  "active_users": 5,
  "max_concurrent": 5,
  "estimated_wait_seconds": 60,
  "can_enter": false,
  "redirect_url": null
}
```

**待合室UI機能:**
- リアルタイムカウントダウンタイマー
- 進捗バー表示
- 待機人数とショップ内人数の表示
- 入室可能時の自動リダイレクト通知

### 2. product-api (在庫管理)

**機能:**
- PostgreSQL ベースの商品管理
- アトミックな在庫管理
- 複数レプリカ間での在庫同期
- 在庫0の商品は購入不可

**仕様:**

| 商品ID | 商品名 | 価格 | 初期在庫 |
|--------|--------|------|----------|
| 1 | 限定スニーカー | ¥29,800 | 3 |
| 2 | コラボTシャツ | ¥8,900 | 5 |
| 3 | コレクターズウォッチ | ¥158,000 | 2 |
| 4 | レアレコード | ¥12,800 | 4 |
| 5 | デザイナーバッグ | ¥89,000 | 1 |

**在庫管理ルール:**

1. **購入処理**
   - SQL: `UPDATE products SET stock = stock - 1 WHERE id = $1 AND stock > 0 RETURNING name, stock`
   - アトミック操作により競合状態を防止
   - 在庫 > 0: 購入成功 (HTTP 200)
   - 在庫 = 0: 購入失敗 (HTTP 409 Conflict)
   - 商品不存在: HTTP 404 Not Found

2. **データベース接続**
   - deadpool-postgres による接続プール管理
   - 環境変数でDB接続設定

**エンドポイント:**

| Method | Path | 説明 |
|--------|------|------|
| GET | `/api/products` | 商品一覧取得 |
| POST | `/api/products/:id/purchase` | 商品購入 |
| POST | `/api/products/reset` | 在庫リセット (テスト用) |
| GET | `/health` | ヘルスチェック (DB接続確認含む) |

**購入レスポンス:**

```json
// 成功時 (HTTP 200)
{
  "success": true,
  "message": "限定スニーカーを購入しました！",
  "remaining_stock": 2
}

// 在庫切れ (HTTP 409)
{
  "success": false,
  "message": "申し訳ありません。この商品は売り切れです。",
  "remaining_stock": 0
}
```

### 3. PostgreSQL (データベース)

**機能:**
- 商品マスタ管理
- 在庫状態の永続化
- 複数APIレプリカ間での一貫性保証

**スキーマ:**

```sql
CREATE TABLE products (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  description TEXT,
  price VARCHAR(50) NOT NULL,
  stock INTEGER NOT NULL DEFAULT 0
);
```

**Istio アクセス制御:**
- mTLS STRICT モード
- product-api からのみアクセス許可

### 4. main-application (ショップフロントエンド)

**機能:**
- 商品一覧表示 (ページネーション付き)
- 購入ボタンUI
- 在庫状態の視覚的表示
- product-api との連携

**仕様:**

| パラメータ | 値 | 説明 |
|-----------|-----|------|
| PRODUCTS_PER_PAGE | 10 | 1ページあたりの商品表示数 |
| PRODUCT_API_URL | http://product-api | 商品API接続先 |

**UI状態:**

1. **在庫あり**
   - 「購入する」ボタン表示 (有効)
   - 残り在庫数表示

2. **在庫なし (stock = 0)**
   - 「売り切れ」ボタン表示 (無効化・グレーアウト)
   - 商品カード全体を半透明化 (opacity: 0.6)

## Istio Traffic Management

### Gateway

```yaml
apiVersion: networking.istio.io/v1alpha3
kind: Gateway
metadata:
  name: waiting-room-gateway
spec:
  selector:
    istio: ingressgateway
  servers:
    - port:
        number: 80
        name: http
        protocol: HTTP
      hosts:
        - "*"
```

### VirtualService - Smart Routing

6段階の優先順位付きルーティングルール:

| 優先度 | 条件 | 宛先 | 機能 |
|--------|------|------|------|
| 1 | URI = /health | main-application | ヘルスチェックバイパス |
| 2 | URI prefix = /api/products | product-api | API直接ルーティング |
| 3 | URI = /shop | main-application | ショップページ |
| 4 | Cookie: waiting_room_passed=true | main-application | 認証済みユーザー |
| 5 | URI prefix = /waiting-room | waiting-room-app | 待合室 |
| 6 | デフォルト | 80% waiting-room / 20% main | トラフィック分散 |

### DestinationRule - Circuit Breaker

```yaml
apiVersion: networking.istio.io/v1alpha3
kind: DestinationRule
metadata:
  name: all-services-dr
spec:
  host: "*.default.svc.cluster.local"
  trafficPolicy:
    outlierDetection:
      consecutive5xxErrors: 5
      interval: 10s
      baseEjectionTime: 30s
      maxEjectionPercent: 50
    loadBalancer:
      simple: ROUND_ROBIN
```

### Security

**PeerAuthentication (mTLS):**
- デフォルト: PERMISSIVE
- PostgreSQL: STRICT (強制)

**AuthorizationPolicy:**
- PostgreSQL へのアクセスを product-api に限定

```yaml
apiVersion: security.istio.io/v1beta1
kind: AuthorizationPolicy
metadata:
  name: postgres-access
spec:
  selector:
    matchLabels:
      app: postgres
  action: ALLOW
  rules:
    - from:
        - source:
            principals: ["cluster.local/ns/default/sa/default"]
      to:
        - operation:
            ports: ["5432"]
```

## User Experience Flow

### 待合室フロー

```
ユーザーアクセス
      │
      ▼
┌───────────────────────────────────────────────┐
│         Istio Gateway (80/20 split)           │
└───────────────────┬───────────────────────────┘
                    │
       ┌────────────┴────────────┐
       │ 80%                20% │
       ▼                        ▼
┌──────────────┐        ┌──────────────┐
│   待合室      │        │   ショップ   │
│              │        │              │
│ ・順番表示    │        │ ・商品一覧   │
│ ・カウントダウン│  ───▶  │ ・在庫表示   │
│ ・進捗バー    │ 自動   │ ・購入機能   │
│ ・リアルタイム │ リダイレクト│            │
└──────────────┘        └──────────────┘
```

### キュー状態の例

| 状態 | ショップ内 | 待機中 | ユーザーX の順番 | 推定待ち時間 |
|------|-----------|--------|-----------------|-------------|
| 初期 | 0/5 | 0 | 1 (入室可) | 0秒 |
| 5人入室 | 5/5 | 0 | - | - |
| 6人目 | 5/5 | 1 | 1 | 0秒 |
| 7人目 | 5/5 | 2 | 2 | 30秒 |
| 8人目 | 5/5 | 3 | 3 | 60秒 |

## Environment Variables

| Variable | Component | Default | Description |
|----------|-----------|---------|-------------|
| PORT | all | 8080 | サーバーポート |
| PRODUCT_API_URL | main-application | http://product-api | 商品APIのURL |
| MAIN_APP_URL | waiting-room-app | /shop | リダイレクト先URL |
| POSTGRES_HOST | product-api | postgres | PostgreSQLホスト |
| POSTGRES_PORT | product-api | 5432 | PostgreSQLポート |
| POSTGRES_DB | product-api | shop | データベース名 |
| POSTGRES_USER | product-api | shopuser | DBユーザー |
| POSTGRES_PASSWORD | product-api | shoppass | DBパスワード |

## Kubernetes Deployment

### Resource Configuration

| Component | Replicas | Image |
|-----------|----------|-------|
| waiting-room-app | 1 | waiting-room-app:latest |
| main-application | 1 | main-application:latest |
| product-api | 2 | product-api:latest |
| postgres | 1 | postgres:15-alpine |

## Testing

### Istio Gateway 経由

```bash
# ヘルスチェック
curl http://localhost:31645/health

# 商品一覧
curl http://localhost:31645/api/products

# 購入
curl -X POST http://localhost:31645/api/products/1/purchase

# 在庫リセット
curl -X POST http://localhost:31645/api/products/reset

# 待合室状態
curl http://localhost:31645/api/status -b "waiting_room_id=test-user-1"
```

### キュー動作確認

```bash
# 複数ユーザーでキューテスト
for i in $(seq 1 10); do
  curl -s http://localhost:31645/api/status \
    -b "waiting_room_id=user-$i" | jq -c
done
```

## Build & Deploy

### Lima + nerdctl

```bash
# イメージビルド
tar -cf - waiting-room-app | limactl shell k8s -- tar -xf - -C /tmp/
limactl shell k8s -- sudo nerdctl build -t waiting-room-app:latest /tmp/waiting-room-app

# k8s.io namespace にロード
limactl shell k8s -- sudo nerdctl save waiting-room-app:latest | \
  limactl shell k8s -- sudo nerdctl --namespace k8s.io load

# デプロイ
limactl cp k8s/*.yaml k8s:/tmp/
limactl shell k8s -- sudo kubectl apply -f /tmp/postgresql.yaml
limactl shell k8s -- sudo kubectl apply -f /tmp/waiting-room.yaml
limactl shell k8s -- sudo kubectl apply -f /tmp/product-api.yaml
limactl shell k8s -- sudo kubectl apply -f /tmp/istio-rules.yaml
```

## Version

- Rust: 1.85
- axum: 0.7
- tokio-postgres: 0.7
- deadpool-postgres: 0.14
- PostgreSQL: 15
- Istio: demo profile
- Kubernetes: 1.35
