# GitOpsの実践ガイド - Argo CD

## 目次
1. [GitOpsとArgo CDの概要](#gitopsとargo-cdの概要)
2. [Argo CDのインストール](#argo-cdのインストール)
3. [アプリケーションのデプロイ](#アプリケーションのデプロイ)
4. [課題](#課題)
5. [高度な設定](#高度な設定)
6. [トラブルシューティング](#トラブルシューティング)
7. [参考資料](#参考資料)

## GitOpsとArgo CDの概要

### GitOpsとは
GitOpsは、インフラストラクチャとアプリケーションの構成をバージョン管理し、Git自体を単一の信頼できる情報源として使用する実践方法です。GitOpsを採用することで、以下のメリットが得られます：

- インフラストラクチャの変更履歴の追跡
- 変更のレビュープロセスの標準化
- 環境の一貫性の確保
- ロールバックの容易さ
- コンプライアンスとアクセス制御の強化

### Argo CDの主要コンポーネント
Argo CDは以下の主要コンポーネントで構成されています：

- API Server：Argo CD APIを提供
- Repository Server：Gitリポジトリの管理
- Application Controller：デプロイメントの監視と同期

![Argo CD Architecture](https://argo-cd.readthedocs.io/en/stable/assets/argocd_architecture.png)

## Argo CDのインストール

### 基本的なインストール
```bash
# Namespaceの作成
kubectl create namespace argocd

# Argo CDのインストール
kubectl apply -n argocd -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml

# ingressの作成
cat << EOF > argocd-ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: argocd-server
  namespace: argocd
  annotations:
    nginx.ingress.kubernetes.io/force-ssl-redirect: "false"
    nginx.ingress.kubernetes.io/backend-protocol: "HTTP"
spec:
  ingressClassName: nginx
  rules:
  - host: argocd.local
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: argocd-server
            port:
              number: 80
EOF

kubectl apply -f argocd-ingress.yaml

# 管理者パスワードの取得
kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath="{.data.password}" | base64 -d
```

### /etc/hostsの設定
```bash
# /etc/hostsに以下を追加
127.0.0.1 argocd.local
```

## 課題

### サンプルアプリケーションのデプロイ
以下の例は、先ほど作成したOpenTelemetryが実装されたワークショップアプリケーションをデプロイしてください。

## 高度な設定

### マルチクラスター管理
複数のクラスターを管理する場合の設定例：

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: prod-cluster-secret
  namespace: argocd
  labels:
    argocd.argoproj.io/secret-type: cluster
type: Opaque
stringData:
  name: prod-cluster
  server: https://kubernetes.default.svc
  config: |
    {
      "bearerToken": "<token>",
      "tlsClientConfig": {
        "insecure": false,
        "caData": "<base64-encoded-ca-cert>"
      }
    }
```

### リソースの健全性チェックのカスタマイズ
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: workshop-app
spec:
  ignoreDifferences:
  - group: apps
    kind: Deployment
    jsonPointers:
    - /spec/replicas
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
    - Validate=false
    retry:
      limit: 5
      backoff:
        duration: 5s
        factor: 2
        maxDuration: 3m
```

### カスタムヘルスチェック
```lua
health.lua
hs = {}
function hs.nginx(obj)
  if obj.status ~= nil then
    if obj.status.availableReplicas == obj.spec.replicas then
      return { status = "Healthy", message = "All replicas are available" }
    else
      return { status = "Progressing", message = "Waiting for replicas" }
    end
  end
  return { status = "Unknown", message = "Unable to determine health" }
end
return hs
```

### RBAC設定
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: argocd-rbac-cm
  namespace: argocd
data:
  policy.default: role:readonly
  policy.csv: |
    p, role:org-admin, applications, *, */*, allow
    p, role:org-admin, clusters, get, *, allow
    p, role:org-admin, repositories, get, *, allow
    p, role:org-admin, repositories, create, *, allow
    p, role:org-admin, repositories, update, *, allow
    p, role:org-admin, repositories, delete, *, allow
    g, org-admin, role:org-admin
```

## トラブルシューティング

### 一般的な問題と解決方法

#### 1. アプリケーションの同期失敗
症状：
- アプリケーションのステータスが`OutOfSync`
- 同期操作が失敗

解決方法：
1. アプリケーションのイベントとログの確認
```bash
kubectl logs -n argocd deployment/argocd-application-controller
```

2. アプリケーションの詳細な状態確認
```bash
kubectl describe application -n argocd workshop-app
```

#### 2. リポジトリ接続の問題
症状：
- リポジトリに接続できない
- SSL/TLS証明書の問題

解決方法：
```bash
# リポジトリの状態確認
kubectl get secret -n argocd argocd-repo-<your-repo-name>

# SSLの問題を回避する設定（テスト環境のみ）
kubectl patch cm argocd-cm -n argocd --type merge -p '{"data":{"repository.credentials":"- url: git@github.com\n  insecureIgnoreHostKey: true"}}'
```

#### 3. パフォーマンスの問題
症状：
- UI/APIの応答が遅い
- メモリ使用量が高い

解決方法：
```yaml
# リソース制限の調整
apiVersion: apps/v1
kind: Deployment
metadata:
  name: argocd-server
spec:
  template:
    spec:
      containers:
      - name: argocd-server
        resources:
          requests:
            memory: "256Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
```

## ベストプラクティス

### 1. リポジトリ管理
- 環境ごとに異なるブランチやフォルダを使用
- Kustomizeを活用した設定管理
- プライベートリポジトリのクレデンシャル管理

### 2. セキュリティ
- RBACの適切な設定
- Secretの暗号化
- 監査ログの有効化

### 3. パフォーマンス最適化
- リソース制限の適切な設定
- キャッシュの活用
- 自動同期の設定

### 4. 監視とメンテナンス
- ログレベルの適切な設定
- メトリクスの監視
- 定期的なバックアップ

## 参考資料

### 公式ドキュメント
- [Argo CD Documentation](https://argo-cd.readthedocs.io/)
- [Argo CD Operator Manual](https://argo-cd.readthedocs.io/en/stable/operator-manual/)
- [Argo CD User Guide](https://argo-cd.readthedocs.io/en/stable/user-guide/)
- [Argo CD Best Practices](https://argo-cd.readthedocs.io/en/stable/user-guide/best_practices/)

### チュートリアルとガイド
- [Getting Started with Argo CD](https://argo-cd.readthedocs.io/en/stable/getting_started/)
- [Argo CD Configuration](https://argo-cd.readthedocs.io/en/stable/operator-manual/declarative-setup/)
- [GitOps with Argo CD](https://www.gitops.tech/)
- [Kustomize Integration](https://argo-cd.readthedocs.io/en/stable/user-guide/kustomize/)

### コミュニティリソース
- [Argo CD GitHub](https://github.com/argoproj/argo-cd)
- [Argo Project](https://argoproj.github.io/)
- [CNCF Argo Project](https://www.cncf.io/projects/argo/)
- [Argo Community](https://argoproj.github.io/community/)
