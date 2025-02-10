# ModSecurity詳細解説

## 1. ModSecurityの基本アーキテクチャ

### 1.1 概要
ModSecurityは、Webアプリケーションファイアウォール（WAF）エンジンとして、主に以下の形態で導入されます：

1. Webサーバー組み込み型（Apache、Nginx）
2. スタンドアロン型
3. Kubernetes環境でのIngress Controller統合型

### 1.2 Kubernetes環境での導入アーキテクチャ

```plaintext
[クライアント] → [Ingress] → [ModSecurity] → [アプリケーション]
                     ↓
              [ModSecurity Logs]
```

## 2. Kubernetes環境での具体的な実装方法

### 2.1 コンポーネント構成

1. **Nginx Ingress Controller**
   - ModSecurityモジュールを内包
   - トラフィックの入り口として機能
   - HTTPリクエスト/レスポンスの制御

2. **ModSecurityエンジン**
   - Nginxモジュールとして動作
   - リクエストの検査と制御を実行
   - OWASP Core Rule Set（CRS）の適用

3. **設定管理**
   - ConfigMapによる設定管理
   - カスタムルールの適用
   - ログ出力の設定

### 2.2 実装手順詳細

1. **ModSecurity設定の準備**
```bash
# 基本設定ファイルの作成
modsecurity/custom-modsecurity.conf
```

2. **Nginx Ingress Controllerの設定**
```yaml
controller:
  config:
    enable-modsecurity: "true"
    enable-owasp-modsecurity-crs: "true"
```

3. **ConfigMapによる設定の適用**
```bash
kubectl create configmap modsecurity-config \
  --from-file=custom-modsecurity.conf
```

### 2.3 設定の詳細解説

```plaintext
[ModSecurity Configuration]
┣━ SecRuleEngine On        # WAFエンジンの有効化
┣━ SecRequestBodyAccess On # リクエストボディの検査有効化
┣━ SecAuditLog            # 監査ログの出力先設定
┗━ SecAuditLogFormat JSON # JSON形式でのログ出力
```

## 3. 動作フロー

### 3.1 リクエスト処理フロー

1. **リクエスト受信**
   ```plaintext
   Client → Ingress Controller
   ```

2. **ModSecurityによる検査**
   ```plaintext
   [Phase 1: Request Headers]
   ↓
   [Phase 2: Request Body]
   ↓
   [Phase 3: Response Headers]
   ↓
   [Phase 4: Response Body]
   ```

3. **ルール評価とアクション**
   ```plaintext
   If (Rule Match) {
     Execute Action (Block/Alert)
   } Else {
     Forward Request
   }
   ```

### 3.2 ログ処理フロー

```plaintext
[ModSecurity Engine]
      ↓
[Audit Log Generation]
      ↓
[JSON Format Output]
      ↓
[stdout / Log Collection]
```

## 4. 設定のカスタマイズ

### 4.1 基本設定

```conf
# ModSecurity基本設定
SecRuleEngine On
SecRequestBodyAccess On
SecResponseBodyAccess On
SecResponseBodyMimeType text/plain text/html text/xml
```

### 4.2 ルールの適用

```conf
# OWASP CRSの適用
Include /etc/nginx/owasp-modsecurity-crs/crs-setup.conf
Include /etc/nginx/owasp-modsecurity-crs/rules/*.conf
```

## 5. 監視とログ収集

### 5.1 ログ形式
```json
{
  "transaction": {
    "time": "2024-02-06T10:00:00Z",
    "client_ip": "192.168.1.1",
    "request": {
      "method": "GET",
      "uri": "/api/data",
      "headers": {}
    },
    "violations": []
  }
}
```

### 5.2 監視設定

1. **メトリクス収集**
   - ModSecurityルール適用数
   - ブロックされたリクエスト数
   - 異常スコア分布

2. **アラート設定**
   - 高リスク攻撃の検知
   - 異常トラフィックの検知
   - ルール違反の集計

## 6. パフォーマンスの最適化

### 6.1 リソース設定

```yaml
resources:
  requests:
    cpu: "100m"
    memory: "128Mi"
  limits:
    cpu: "500m"
    memory: "512Mi"
```

### 6.2 チューニングポイント

1. **リクエストボディサイズ制限**
```conf
SecRequestBodyLimit 13107200
SecRequestBodyNoFilesLimit 131072
```

2. **監査ログの最適化**
```conf
SecAuditEngine RelevantOnly
SecAuditLogRelevantStatus "^(?:5|4(?!04))"
```

## 7. トラブルシューティング

### 7.1 一般的な問題と解決策

1. **誤検知の対応**
   - ルールの調整
   - ホワイトリストの設定
   - カスタムルールの作成

2. **パフォーマンス問題**
   - リソース使用量の監視
   - ルールの最適化
   - キャッシュ設定の調整

### 7.2 デバッグ手法

```bash
# ログの確認
kubectl logs -n ingress-nginx \
  $(kubectl get pods -n ingress-nginx -l app.kubernetes.io/component=controller -o name)

# 設定の確認
kubectl exec -it \
  $(kubectl get pods -n ingress-nginx -l app.kubernetes.io/component=controller -o name) \
  -- nginx -T
```

## 8. セキュリティベストプラクティス

1. **定期的なルール更新**
2. **監査ログの定期的なレビュー**
3. **カスタムルールの適切な管理**
4. **パフォーマンスモニタリング**
5. **インシデント対応手順の整備**

## 9. まとめ

ModSecurityのKubernetes環境への導入は、Nginx Ingress Controllerを介して行われ、ConfigMapによる柔軟な設定管理が可能です。適切な設定とモニタリングにより、効果的なWebアプリケーション保護を実現できます。
