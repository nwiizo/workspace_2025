# WAFによるネットワーク攻撃の検知 - 実装ガイド

## 1. 目的

Kubernetes環境において、Nginx Ingress ControllerとModSecurityを利用してWAF（Web Application Firewall）機能を実装し、アプリケーションレベルでのセキュリティ対策を実現する。

### 1.1 達成目標
- OWASP Top10を含む一般的なWeb攻撃からの保護
- アプリケーションレベルでのセキュリティ制御
- WAFログの収集と監視

### 1.2 前提条件
- Lima がインストール済みであること
- Kind がインストール済みであること
- kubectl コマンドが使用可能であること
- Helm v3以上がインストール済みであること

## 2. システム構成

### 2.1 使用コンポーネント
- Lima (macOS用のLinux仮想マシンマネージャ)
- Kind (Kubernetes in Docker)
- Nginx Ingress Controller
- ModSecurity (WAFエンジン)
- OWASP ModSecurity Core Rule Set (CRS)

### 2.2 ファイル構成
```
waf-test/
├── README.md                      # プロジェクトの概要
├── cluster/
│   └── kind-config.yaml          # Kindクラスタ設定
├── ingress/
│   ├── values.yaml               # Nginx Ingress設定
│   └── ingress-resource.yaml     # Ingressリソース定義
├── modsecurity/
│   └── custom-modsecurity.conf   # ModSecurity設定
├── test-app/
│   ├── deployment.yaml           # テストアプリ定義
│   └── service.yaml              # テストアプリサービス
└── tests/
    ├── deploy.sh                 # デプロイスクリプト
    └── attack-patterns.sh        # 攻撃パターンテスト
```

## 3. 環境構築手順

### 3.1 Kindクラスタの作成
```yaml
# cluster/kind-config.yaml
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
name: waf-test
nodes:
- role: control-plane
  kubeadmConfigPatches:
  - |
    kind: InitConfiguration
    nodeRegistration:
      kubeletExtraArgs:
        node-labels: "ingress-ready=true"
  extraPortMappings:
  - containerPort: 80
    hostPort: 80
    protocol: TCP
    listenAddress: "0.0.0.0"
  - containerPort: 443
    hostPort: 443
    protocol: TCP
    listenAddress: "0.0.0.0"
```

```bash
# クラスタの作成
kind create cluster --config cluster/kind-config.yaml
```

### 3.2 ModSecurity設定
```bash
# ディレクトリの作成
mkdir -p modsecurity

# ModSecurityの推奨設定をダウンロード
curl -L -O https://raw.githubusercontent.com/SpiderLabs/ModSecurity/v3/master/modsecurity.conf-recommended

# 設定ファイルをコピー
cp modsecurity.conf-recommended modsecurity/custom-modsecurity.conf

# 設定の変更（macOS/Linux共通）
sed -i.bak 's/^SecRuleEngine DetectionOnly/SecRuleEngine On/' modsecurity/custom-modsecurity.conf
sed -i.bak 's/^SecAuditLog \/var\/log\/modsec_audit.log/SecAuditLog \/dev\/stdout/' modsecurity/custom-modsecurity.conf
sed -i.bak 's/^SecUnicodeMapFile unicode.mapping 20127/SecUnicodeMapFile \/etc\/nginx\/modsecurity\/unicode.mapping 20127/' modsecurity/custom-modsecurity.conf
sed -i.bak 's/^SecStatusEngine On/SecStatusEngine Off/' modsecurity/custom-modsecurity.conf

# 追加設定の追記
echo "SecAuditLogFormat JSON" >> modsecurity/custom-modsecurity.conf
echo "SecRuleRemoveById 920350" >> modsecurity/custom-modsecurity.conf

# バックアップファイルの削除
rm modsecurity/*.bak
```

### 3.3 Nginx Ingressのインストール
```yaml
# ingress/values.yaml
controller:
  config:
    enable-modsecurity: "true"
    enable-owasp-modsecurity-crs: "true"
    modsecurity-snippet: |
      Include /etc/nginx/owasp-modsecurity-crs/custom/custom-modsecurity.conf

  extraVolumeMounts:
    - name: modsecurity-config
      mountPath: /etc/nginx/owasp-modsecurity-crs/custom/
  extraVolumes:
    - name: modsecurity-config
      configMap:
        name: modsecurity-config
```

```bash
# namespaceの作成
kubectl create namespace ingress-nginx

# ConfigMapの作成
kubectl -n ingress-nginx create configmap modsecurity-config \
  --from-file=custom-modsecurity.conf=modsecurity/custom-modsecurity.conf

# Helmリポジトリの追加
helm repo add ingress-nginx https://kubernetes.github.io/ingress-nginx
helm repo update

# Nginx Ingress Controllerのインストール
helm install ingress-nginx ingress-nginx/ingress-nginx \
  --namespace ingress-nginx \
  --values ingress/values.yaml
```

## 4. テスト環境の構築

### 4.1 テストアプリケーションのデプロイ
```yaml
# test-app/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: test-app
spec:
  replicas: 1
  selector:
    matchLabels:
      app: test-app
  template:
    metadata:
      labels:
        app: test-app
    spec:
      containers:
      - name: nginx
        image: nginx:1.21
        ports:
        - containerPort: 80
        resources:
          requests:
            memory: "64Mi"
            cpu: "100m"
          limits:
            memory: "128Mi"
            cpu: "200m"
```

```yaml
# test-app/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: test-app
spec:
  selector:
    app: test-app
  ports:
  - protocol: TCP
    port: 80
    targetPort: 80
```

```yaml
# test-app/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: test-app-ingress
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "false"
spec:
  ingressClassName: nginx
  rules:
  - host: test-app.localdev.me
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: test-app
            port:
              number: 80
```

### 4.2 デプロイスクリプト
```bash
#!/bin/bash
# tests/deploy.sh
set -ex

echo "Creating test application namespace..."
kubectl create namespace test-app || true

echo "Applying test application manifests..."
kubectl apply -f test-app/deployment.yaml -n test-app
kubectl apply -f test-app/service.yaml -n test-app
kubectl apply -f test-app/ingress.yaml -n test-app

echo "Waiting for deployment to be ready..."
kubectl wait --for=condition=available deployment/test-app -n test-app --timeout=60s

echo "Testing the application..."
kubectl port-forward -n ingress-nginx service/ingress-nginx-controller 8080:80
```

### 4.3 セキュリティテスト
```bash
#!/bin/bash
# tests/attack-patterns.sh
set -e

PORT=8080
HOST="test-app.localdev.me"
CURL_OPTS="-H Host:${HOST}"

# port-forwardを開始（バックグラウンドで実行）
echo "Starting port forward..."
kubectl port-forward -n ingress-nginx service/ingress-nginx-controller ${PORT}:80 &
FORWARD_PID=$!

# プロセスが終了時にport-forwardを停止
trap "kill $FORWARD_PID" EXIT

# port-forwardが準備できるまで少し待機
sleep 5

echo "Running security tests against localhost:${PORT}"

# 各種攻撃パターンのテスト
echo -e "\n1. Testing normal access..."
curl -s ${CURL_OPTS} "http://localhost:${PORT}/" | head -n 5

echo -e "\n2. Testing XSS attack..."
curl -s ${CURL_OPTS} -X POST -d "payload=<script>alert(1)</script>" "http://localhost:${PORT}/"

echo -e "\n3. Testing SQL injection..."
curl -s ${CURL_OPTS} "http://localhost:${PORT}/?id=1'+OR+'1'='1"

echo -e "\n4. Testing directory traversal..."
curl -s ${CURL_OPTS} "http://localhost:${PORT}/../../../etc/passwd"

echo -e "\n5. Testing command injection..."
curl -s ${CURL_OPTS} "http://localhost:${PORT}/?cmd=cat%20/etc/passwd"

# ログの確認
PODNAME=$(kubectl -n ingress-nginx get pod -l app.kubernetes.io/component=controller -o=jsonpath='{.items[0].metadata.name}')
echo -e "\nChecking ModSecurity logs..."
kubectl -n ingress-nginx logs $PODNAME | grep ModSecurity: | tail -n 10
```

## 5. WAFの動作確認

### 5.1 テストの実行方法
```bash
# デプロイの実行
./tests/deploy.sh

# 別のターミナルで攻撃パターンテストを実行
./tests/attack-patterns.sh
```

### 5.2 期待される結果
WAFは以下のような攻撃を検知し、403 Forbiddenレスポンスを返します：

1. SQLインジェクション攻撃:
```
GET /?id=1'+OR+'1'='1
Anomaly Score: 5
```

2. XSS攻撃:
```
POST / with payload=<script>alert(1)</script>
Anomaly Score: 20
```

3. コマンドインジェクション:
```
GET /?cmd=cat%20/etc/passwd
Anomaly Score: 10
```

## 6. トラブルシューティング

### 6.1 デプロイの問題
- クラスタの状態確認: `kubectl cluster-info`
- Podの状態確認: `kubectl get pods -n ingress-nginx`
- ログの確認: `kubectl logs -n ingress-nginx deployment/ingress-nginx-controller`

### 6.2 WAFの動作確認
```bash
# ModSecurityの設定確認
kubectl -n ingress-nginx exec -it \
  $(kubectl -n ingress-nginx get pods -l app.kubernetes.io/component=controller -o name) \
  -- nginx -T | grep -i modsecurity
```

## 7. 制限事項・注意点

### 7.1 環境の制限
- Lima/Kind環境のため、本番環境とは設定が異なる場合がある
- ポートフォワーディングを使用したテスト環境のため、実際の運用とは異なる

## 8. 参考文献
- [Lima Documentation](https://github.com/lima-vm/lima)
- [Kind Documentation](https://kind.sigs.k8s.io/)
- [Nginx Ingress Controller Documentation](https://kubernetes.github.io/ingress-nginx/)
- [ModSecurity GitHub](https://github.com/SpiderLabs/ModSecurity)
- [OWASP ModSecurity Core Rule Set](https://coreruleset.org/)
