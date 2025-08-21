# コンテナ技術実践ワークショップ 2025年冬 001

## 概要
このワークショップでは、最新のコンテナ技術とKubernetesの基礎から実践的な使用方法まで学びます。2025年の開発現場で必要とされるスキルを身につけることを目指します。

## 前提条件
以下のツールをインストールしてください：
* Git (最新版)
* Docker cli (27.0以降)
* kubectl (1.32以降)
* Go (1.23以降)
* Helm (3.16以降)

## セクション1: ローカル開発環境の構築
### KinD (Kubernetes in Docker) のセットアップ
```bash
# インストール
export GOPATH=/usr/local/bin/go/bin
export PATH=$PATH:$GOPATH
go get -u sigs.k8s.io/kind

# クラスター作成
kind create cluster --name workshop-2025
```

### クラスター設定
```bash
kind get kubeconfig > kubeconfig.yaml
export KUBECONFIG=./kubeconfig.yaml:~/.kube/config
kubectl cluster-info
```
### デプロイのテスト [What happens when ... Kubernetes](https://github.com/jamiehannaford/what-happens-when-k8s) 風
詳細が知りたければ[what happens when k8s journy](https://speakerdeck.com/nnao45/what-happens-when-k8s-journy) を読んでください。

#### 実行
今、デプロイしたKubernetes cluster にnginx をデプロイします
```bash
kubectl run nginx --image=nginx --replicas=3
kubectl run --generator=deployment/apps.v1 は非推奨であり、将来のバージョンで削除されます。代わりに kubectl run --generator=run-pod/v1 または kubectl create を使用してください。
deployment.apps/nginx created
```

#### 確認
今、デプロイしたものを確認します
```bash
kubectl get deploy
NAME              READY   UP-TO-DATE   AVAILABLE   AGE
nginx             3/3     3            3           49s

kubectl get rs
NAME                         DESIRED   CURRENT   READY   AGE
nginx-6db489d4b7             3         3         3       68s

kubectl get pod
NAME                               READY   STATUS    RESTARTS   AGE
nginx-6db489d4b7-2djdm             1/1     Running   0          5m44s
nginx-6db489d4b7-2vhs8             1/1     Running   0          5m44s
nginx-6db489d4b7-lrgcd             1/1     Running   0          5m44s
```

#### 削除
Pod を削除する
```bash
kubectl delete pod nginx-6db489d4b7-2djdm nginx-6db489d4b7-2vhs8 nginx-6db489d4b7-lrgcd
pod "nginx-6db489d4b7-2djdm" deleted
pod "nginx-6db489d4b7-2vhs8" deleted
pod "nginx-6db489d4b7-lrgcd" deleted
```
削除したリソースを確認する
```
kubectl get pod
NAME                               READY   STATUS    RESTARTS   AGE
nginx-6db489d4b7-6sgnz             1/1     Running   0          16s
nginx-6db489d4b7-nhfvh             1/1     Running   0          16s
nginx-6db489d4b7-tl7m8             1/1     Running   0          16s
```
Podを削除しても上位のリソースが存在しているので残り続ける。次にreplicaset を削除する
```
kubectl get rs
NAME                         DESIRED   CURRENT   READY   AGE
nginx-6db489d4b7             3         3         3       29m

# 次はrs を削除する
kubectl delete rs/nginx-6db489d4b7
replicaset.apps "nginx-6db489d4b7" deleted

kubectl get rs
NAME                         DESIRED   CURRENT   READY   AGE
nginx-6db489d4b7             3         3         3       78s

# 上位リソースが削除されたのでpod も削除されました
kubectl get pod
NAME                               READY   STATUS    RESTARTS   AGE
nginx-6db489d4b7-8cq4x             1/1     Running   0          74s
nginx-6db489d4b7-cgknc             1/1     Running   0          74s
nginx-6db489d4b7-ghm7w             1/1     Running   0          74s
# 区切るのが面倒なのでこのまま進める
#kubectl get deployments.apps
NAME              READY   UP-TO-DATE   AVAILABLE   AGE
nginx             3/3     3            3           34m

# deployments の削除
kubectl delete deployment/nginx
deployment.apps "nginx" deleted

#他のリソースも削除されているので確認してみてください
```

#### install yamls 
nginx-deployment.yaml をデプロイします。
```
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nginx-deployment
  labels:
    app: nginx
spec:
  replicas: 3
  selector:
    matchLabels:
      app: nginx
  template:
    metadata:
      labels:
        app: nginx
    spec:
      containers:
      - name: nginx
        image: nginx:1.7.9
        ports:
        - containerPort: 80
```

resourceの取得
```
kubectl get pods,rs
```

詳細の取得
```
kubectl describe pod/<name>
kubectl describe deployment <name>
```

resourceの削除
```
kubectl delete pods,rs
```


## セクション2: はじめてのデプロイメント
### 実践課題1: シンプルなWebアプリケーション
以下のGoアプリケーションをコンテナ化し、Dockerfileをビルドしてデプロイします。

```go
package main

import (
    "fmt"
    "net/http"
    "os"
    "time"
)

func main() {
    http.HandleFunc("/", handler)
    http.HandleFunc("/health", healthHandler)
    http.HandleFunc("/time", timeHandler)
    http.HandleFunc("/env", envHandler)
    http.ListenAndServe(":8080", nil)
}

func timeHandler(w http.ResponseWriter, r *http.Request) {
    currentTime := time.Now().Format(time.RFC3339)
    fmt.Fprintf(w, "Current Server Time: %s", currentTime)
}

func envHandler(w http.ResponseWriter, r *http.Request) {
    envVars := make(map[string]string)
    for _, e := range os.Environ() {
        pair := strings.SplitN(e, "=", 2)
        envVars[pair[0]] = pair[1]
    }
    w.Header().Set("Content-Type", "application/json")
    json.NewEncoder(w).Encode(envVars)
}

func handler(w http.ResponseWriter, r *http.Request) {
    msg := os.Getenv("APP_MESSAGE")
    if msg == "" {
        msg = "Welcome to Workshop 2025!"
    }
    fmt.Fprintf(w, msg)
}

func healthHandler(w http.ResponseWriter, r *http.Request) {
    w.WriteHeader(http.StatusOK)
    fmt.Fprintf(w, "Healthy: %s", time.Now().Format(time.RFC3339))
}
```

### Dockerfile
```dockerfile
FROM golang:1.22-alpine AS builder

WORKDIR /app
COPY main.go .
RUN go build -o webservice .

FROM alpine:3.19

WORKDIR /app
COPY --from=builder /app/webservice .
EXPOSE 8080
CMD ["./webservice"]
```

### ビルドとデプロイ
```bash
docker build -t webservice:v1 .
docker run -d -p 8080:8080 webservice:v1
```

### テスト
```bash
curl http://localhost:8080
curl http://localhost:8080/health
curl http://localhost:8080/time
### 追い課題:Dockerfile に環境変数を追加してみてください
curl http://localhost:8080/env
```

# セクション3: Kubernetes リソース管理実践

## 実践課題2: Kubernetes マニフェストファイルの作成

### 目的
- [Kubernetes基本リソース](https://kubernetes.io/docs/concepts/overview/working-with-objects/kubernetes-objects/)の理解と作成
- マニフェストファイルの適切な構造化
- 環境変数管理の実践

### 必要なリソース
1. [Deployment](https://kubernetes.io/docs/concepts/workloads/controllers/deployment/)
2. [Service (ClusterIP)](https://kubernetes.io/docs/concepts/services-networking/service/)
3. [ConfigMap](https://kubernetes.io/docs/concepts/configuration/configmap/)

### 手順詳細

#### 1. ConfigMapの作成
まず、[アプリケーションの設定を管理するConfigMap](https://kubernetes.io/docs/tasks/configure-pod-container/configure-pod-configmap/)を作成します。

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: myapp-config
  labels:
    app: myapp
data:
  APP_MESSAGE: "Hello from ConfigMap!"
  APP_ENV: "development"
```

#### 2. Deploymentの作成
[アプリケーションのDeployment](https://kubernetes.io/docs/concepts/workloads/controllers/deployment/#creating-a-deployment)を定義します。以下の要件を満たすように設定してください：

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp
  labels:
    app: myapp
spec:
  replicas: 3
  selector:
    matchLabels:
      app: myapp
  template:
    metadata:
      labels:
        app: myapp
    spec:
      containers:
      - name: myapp
        image: myapp:latest
        ports:
        - containerPort: 8080
        envFrom:
        - configMapRef:
            name: myapp-config
        resources:
          requests:
            cpu: "100m"
            memory: "128Mi"
          limits:
            cpu: "200m"
            memory: "256Mi"
```

#### 3. Serviceの作成
[アプリケーションへのアクセスを提供するService](https://kubernetes.io/docs/concepts/services-networking/service/#defining-a-service)を作成します：

```yaml
apiVersion: v1
kind: Service
metadata:
  name: myapp-service
  labels:
    app: myapp
spec:
  type: ClusterIP
  ports:
  - port: 80
    targetPort: 8080
    protocol: TCP
  selector:
    app: myapp
```

### 動作確認手順

1. [マニフェストの適用](https://kubernetes.io/docs/concepts/cluster-administration/manage-deployment/#using-kubectl-apply)
```bash
kubectl apply -f configmap.yaml
kubectl apply -f deployment.yaml
kubectl apply -f service.yaml
```

2. [リソースの確認](https://kubernetes.io/docs/reference/kubectl/cheatsheet/#viewing-and-finding-resources)
```bash
kubectl get configmap,deployment,service -l app=myapp
kubectl describe deployment myapp -l app=myapp
kubectl get pods -l app=myapp
```

3. [アプリケーションへのアクセス](https://kubernetes.io/docs/tasks/access-application-cluster/access-cluster/#manually-constructing-apiserver-proxy-urls)
```bash
kubectl proxyを使用したアクセス
kubectl proxy &
curl http://localhost:8001/api/v1/namespaces/default/services/myapp-service/proxy/
```

## 実践課題3: Helmチャートの作成

### 目的
- [Helmの基本概念](https://helm.sh/docs/intro/using_helm/)の理解
- [テンプレート化](https://helm.sh/docs/chart_template_guide/getting_started/)による設定の柔軟性確保
- リソース管理の効率化

### Helmとは
[Helm](https://helm.sh/)は、Kubernetesアプリケーションのパッケージマネージャーです。主な利点として：

- [複数のKubernetesリソースをまとめて管理](https://helm.sh/docs/topics/charts/)
- [環境ごとの設定値の柔軟な変更](https://helm.sh/docs/chart_template_guide/values_files/)
- [バージョン管理とロールバック](https://helm.sh/docs/topics/charts_hooks/)の容易さ
- [再利用可能なパッケージ](https://helm.sh/docs/topics/library_charts/)の作成

### チャート作成手順

1. [チャートの初期化](https://helm.sh/docs/helm/helm_create/)
```bash
helm create myapp
```

2. [values.yamlの設定](https://helm.sh/docs/chart_template_guide/values_files/)
```yaml
# values.yaml
image:
  repository: myapp
  tag: latest
  pullPolicy: IfNotPresent

replicaCount: 3

service:
  type: ClusterIP
  port: 80

config:
  message: "Hello from Helm!"
  environment: "development"

resources:
  requests:
    cpu: 100m
    memory: 128Mi
  limits:
    cpu: 200m
    memory: 256Mi

probes:
  livenessProbe:
    httpGet:
      path: /health
      port: http
    initialDelaySeconds: 30
    periodSeconds: 10
  readinessProbe:
    httpGet:
      path: /health
      port: http
    initialDelaySeconds: 5
    periodSeconds: 5
```

3. [テンプレートの作成](https://helm.sh/docs/chart_template_guide/getting_started/)
必要なテンプレートファイルを `templates/` ディレクトリに作成します：
- ConfigMap (`templates/configmap.yaml`)
- Deployment (`templates/deployment.yaml`)
- Service (`templates/service.yaml`)

### 実装すべき機能

1. [環境変数のテンプレート化](https://helm.sh/docs/chart_template_guide/accessing_files/)
```yaml
# templates/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ include "myapp.fullname" . }}-config
data:
  APP_MESSAGE: {{ .Values.config.message | quote }}
  APP_ENV: {{ .Values.config.environment | quote }}
```

2. [リソース制限の設定](https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/)
```yaml
# templates/deployment.yaml (一部抜粋)
resources:
{{ toYaml .Values.resources | indent 10 }}
```

3. [Probeの設定](https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/)
```yaml
# templates/deployment.yaml (一部抜粋)
livenessProbe:
{{ toYaml .Values.probes.livenessProbe | indent 10 }}
readinessProbe:
{{ toYaml .Values.probes.readinessProbe | indent 10 }}
```

### チャートのテストとデプロイ

1. [チャートの検証](https://helm.sh/docs/topics/charts_tests/)
```bash
helm lint myapp
helm template myapp ./myapp
```

2. [チャートのインストール](https://helm.sh/docs/helm/helm_install/)
```bash
helm install myapp-release ./myapp
```

3. [デプロイの確認](https://helm.sh/docs/helm/helm_status/)
```bash
helm list
kubectl get all -l app=myapp
```

### 発展課題
1. [複数環境向けの値ファイル](https://helm.sh/docs/chart_template_guide/values_files/)の作成
2. [カスタムヘルパー関数](https://helm.sh/docs/chart_template_guide/functions_and_pipelines/)の実装
3. [テストの追加](https://helm.sh/docs/topics/charts_tests/)（`tests/` ディレクトリ）
4. [CI/CDパイプライン](https://helm.sh/docs/topics/charts_testing/)への統合

## 評価基準
- [ ] [マニフェストファイルの適切な構造化](https://kubernetes.io/docs/concepts/overview/working-with-objects/kubernetes-objects/)
- [ ] [環境変数の適切な管理](https://kubernetes.io/docs/tasks/inject-data-application/define-environment-variable-container/)
- [ ] [リソース制限の適切な設定](https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/)
- [ ] [Probeの適切な実装](https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/)
- [ ] [テンプレート化の適切な実装](https://helm.sh/docs/chart_template_guide/getting_started/)
- [ ] [エラーハンドリングの考慮](https://kubernetes.io/docs/tasks/debug/debug-application/)
- [ ] [セキュリティ設定の考慮](https://kubernetes.io/docs/concepts/security/pod-security-standards/)

## 参考文献
- [Kubernetes公式ドキュメント](https://kubernetes.io/docs/home/)
- [Helm公式ドキュメント](https://helm.sh/docs/)
- [Kubernetesパターン](https://k8spatterns.io/)
- [12 Factor App](https://12factor.net/)
