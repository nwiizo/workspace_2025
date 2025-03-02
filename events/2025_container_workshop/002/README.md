# Kubernetesにおけるモニタリングスタックの実装ガイド

## 目次
1. [概要](#概要)
2. [Prometheusの実装](#prometheusの実装)
3. [課題](#課題)
4. [Grafanaの設定](#grafanaの設定)
5. [アラート設定](#アラート設定)
6. [高度な設定とチューニング](#高度な設定とチューニング)
7. [トラブルシューティング](#トラブルシューティング)
8. [参考資料](#参考資料)

## 概要

### モニタリングスタックとは
Kubernetesクラスターでは、システムの健全性、パフォーマンス、可用性を監視するために、包括的なモニタリングソリューションが必要です。本ガイドでは、以下のコンポーネントを使用した完全なモニタリングスタックの実装方法を説明します：

- **Prometheus**: メトリクス収集とストレージ
- **Grafana**: データの可視化とダッシュボード
- **AlertManager**: アラート管理と通知
- **kube-state-metrics**: Kubernetesのメタデータ収集
- **node-exporter**: ノードレベルのメトリクス収集

### アーキテクチャ概要
```
                                    ┌─────────────┐
                                    │             │
                                    │  Grafana    │
                                    │             │
                                    └───────┬─────┘
                                            │
┌──────────────┐                   ┌───────┴─────┐
│              │                   │             │
│ AlertManager │◄──────────────────┤ Prometheus  │
│              │                   │             │
└──────────────┘                   └───────┬─────┘
                                           │
                           ┌───────────────┴───────────────┐
                           │                               │
                    ┌──────┴──────┐               ┌────────┴────────┐
                    │             │               │                 │
               node-exporter  kube-state-metrics  application-metrics
```

## Prometheusの実装

### kube-prometheus-stackのインストール
kube-prometheus-stackは、Prometheus Operatorを中心としたモニタリングスタックをKubernetes上に展開します。

```bash
# Helmリポジトリの追加
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo update

# 名前空間の作成
kubectl create namespace monitoring

# values.yamlの作成
cat << EOF > monitoring-values.yaml
prometheus:
  prometheusSpec:
    retention: 30d
    storageSpec:
      volumeClaimTemplate:
        spec:
          storageClassName: standard
          accessModes: ["ReadWriteOnce"]
          resources:
            requests:
              storage: 50Gi
    resources:
      requests:
        cpu: 1
        memory: 2Gi
      limits:
        cpu: 2
        memory: 4Gi
    additionalScrapeConfigs:
      - job_name: 'kubernetes-service-endpoints'
        kubernetes_sd_configs:
          - role: endpoints
        relabel_configs:
          - source_labels: [__meta_kubernetes_service_annotation_prometheus_io_scrape]
            action: keep
            regex: true

alertmanager:
  config:
    global:
      resolve_timeout: 5m
    route:
      group_wait: 30s
      group_interval: 5m
      repeat_interval: 12h
      receiver: 'slack'
      routes:
      - match:
          severity: critical
        receiver: 'slack'
    receivers:
    - name: 'slack'
      slack_configs:
      - channel: '#alerts'
        api_url: 'https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK'

grafana:
  persistence:
    enabled: true
    size: 10Gi
  admin:
    existingSecret: grafana-admin-credentials
    userKey: admin-user
    passwordKey: admin-password
  dashboardProviders:
    dashboardproviders.yaml:
      apiVersion: 1
      providers:
      - name: 'default'
        orgId: 1
        folder: ''
        type: file
        disableDeletion: false
        editable: true
        options:
          path: /var/lib/grafana/dashboards
EOF

# スタックのインストール
helm install monitoring prometheus-community/kube-prometheus-stack \
  --namespace monitoring \
  -f monitoring-values.yaml
```

### カスタムメトリクスの設定
アプリケーション固有のメトリクスを収集するためのServiceMonitor設定：

```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: app-metrics
  namespace: monitoring
  labels:
    release: monitoring
spec:
  selector:
    matchLabels:
      app: your-app
  namespaceSelector:
    matchNames:
      - default
  endpoints:
  - port: metrics
    interval: 15s
    path: /metrics
    honorLabels: true
    metricRelabelings:
    - sourceLabels: [__name__]
      regex: 'go_.*'
      action: drop
```

### 高度なPrometheusルール
重要なメトリクスに対するアラートルールの設定：

```yaml
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: kubernetes-apps
  namespace: monitoring
  labels:
    release: monitoring
spec:
  groups:
  - name: kubernetes-apps
    rules:
    - alert: PodCrashLooping
      expr: rate(kube_pod_container_status_restarts_total{job="kube-state-metrics"}[15m]) * 60 * 5 > 0
      for: 15m
      labels:
        severity: warning
      annotations:
        summary: Pod {{ $labels.namespace }}/{{ $labels.pod }} is crash looping
        description: Pod {{ $labels.namespace }}/{{ $labels.pod }} is restarting {{ $value }} times / 5 minutes

    - alert: PodNotReady
      expr: sum by (namespace, pod) (max by(namespace, pod) (kube_pod_status_phase{phase=~"Pending|Unknown"}) * on(namespace, pod) group_left(owner_kind) max by(namespace, pod, owner_kind) (kube_pod_owner{owner_kind!="Job"})) > 0
      for: 15m
      labels:
        severity: warning
      annotations:
        summary: Pod {{ $labels.namespace }}/{{ $labels.pod }} is not ready
        description: Pod {{ $labels.namespace }}/{{ $labels.pod }} has been in a non-ready state for longer than 15 minutes

    - alert: ContainerHighCPU
      expr: sum(rate(container_cpu_usage_seconds_total{container!=""}[5m])) by (namespace, pod, container) > 0.8
      for: 10m
      labels:
        severity: warning
      annotations:
        summary: Container {{ $labels.container }} in pod {{ $labels.pod }} has high CPU usage
        description: Container {{ $labels.container }} in pod {{ $labels.namespace }}/{{ $labels.pod }} is using more than 80% CPU for the last 10 minutes

    - alert: ContainerHighMemory
      expr: sum(container_memory_working_set_bytes{container!=""}) by (namespace, pod, container) / sum(container_spec_memory_limit_bytes{container!=""}) by (namespace, pod, container) * 100 > 80
      for: 10m
      labels:
        severity: warning
      annotations:
        summary: Container {{ $labels.container }} in pod {{ $labels.pod }} has high memory usage
        description: Container {{ $labels.container }} in pod {{ $labels.namespace }}/{{ $labels.pod }} is using more than 80% of its memory limit for the last 10 minutes
```

## 課題

### 1. メトリクスの収集
- Goアプリケーションにメトリクスを追加するための実装してください。

### 2. ブラックボックス監視
- blackbox-exporterを使用して、サービスの可用性を監視する設定を行ってください。

## Grafanaの設定

### 基本設定
Grafanaの基本設定とセキュリティ設定：

```yaml
grafana:
  persistence:
    enabled: true
    size: 10Gi
  admin:
    existingSecret: grafana-admin-credentials
    userKey: admin-user
    passwordKey: admin-password
  grafana.ini:
    security:
      allow_embedding: true
      cookie_secure: true
      disable_gravatar: true
    auth:
      disable_login_form: false
    auth.anonymous:
      enabled: true
      org_role: Viewer
    smtp:
      enabled: true
      host: smtp.gmail.com:587
      user: your-email@gmail.com
      password: your-app-specific-password
```

### カスタムダッシュボードの作成

#### クラスターオーバービューダッシュボード
```json
{
  "annotations": {
    "list": [
      {
        "builtIn": 1,
        "datasource": "-- Grafana --",
        "enable": true,
        "hide": true,
        "iconColor": "rgba(0, 211, 255, 1)",
        "name": "Annotations & Alerts",
        "type": "dashboard"
      }
    ]
  },
  "editable": true,
  "gnetId": null,
  "graphTooltip": 0,
  "id": 1,
  "links": [],
  "panels": [
    {
      "datasource": null,
      "fieldConfig": {
        "defaults": {
          "color": {
            "mode": "thresholds"
          },
          "mappings": [],
          "thresholds": {
            "mode": "absolute",
            "steps": [
              {
                "color": "green",
                "value": null
              },
              {
                "color": "red",
                "value": 80
              }
            ]
          }
        },
        "overrides": []
      },
      "gridPos": {
        "h": 8,
        "w": 12,
        "x": 0,
        "y": 0
      },
      "id": 2,
      "options": {
        "reduceOptions": {
          "calcs": [
            "lastNotNull"
          ],
          "fields": "",
          "values": false
        },
        "showThresholdLabels": false,
        "showThresholdMarkers": true
      },
      "pluginVersion": "7.5.2",
      "targets": [
        {
          "exemplar": true,
          "expr": "sum(kube_pod_container_resource_requests{resource=\"cpu\"}) / sum(kube_node_status_allocatable{resource=\"cpu\"}) * 100",
          "interval": "",
          "legendFormat": "",
          "refId": "A"
        }
      ],
      "title": "Cluster CPU Usage",
      "type": "gauge"
    }
  ],
  "schemaVersion": 27,
  "style": "dark",
  "tags": [],
  "templating": {
    "list": []
  },
  "time": {
    "from": "now-6h",
    "to": "now"
  },
  "timepicker": {},
  "timezone": "",
  "title": "Cluster Overview",
  "uid": "cluster-overview",
  "version": 1
}
```

### アラート通知の設定
Grafanaのアラート通知チャンネルの設定：

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: grafana-notification-channels
  namespace: monitoring
data:
  notification.yaml: |
    notifiers:
      - name: slack-notifications
        type: slack
        uid: slack1
        org_id: 1
        is_default: true
        send_reminder: true
        frequency: 1h
        disable_resolve_message: false
        settings:
          url: https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK
          recipient: "#alerts"
          uploadImage: true
          
      - name: email-notifications
        type: email
        uid: email1
        org_id: 1
        is_default: false
        settings:
          addresses: alerts@your-domain.com
```

## 高度な設定とチューニング

### Prometheusのスケーリングとパフォーマンスチューニング

```yaml
prometheus:
  prometheusSpec:
    retention: 30d
    retentionSize: "50GB"
    walCompression: true
    resources:
      requests:
        cpu: 2
        memory: 4Gi
      limits:
        cpu: 4
        memory: 8Gi
    storageSpec:
      volumeClaimTemplate:
        spec:
          storageClassName: fast-ssd
          resources:
            requests:
              storage: 100Gi
    additionalArgs:
      - --query.max-samples=50000000
      - --query.timeout=2m
      - --query.max-concurrency=20
```

### カスタムレコーディングルール
頻繁に使用されるクエリのパフォーマンス最適化：

```yaml
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: recording-rules
  namespace: monitoring
spec:
  groups:
  - name: recording-rules
    rules:
    - record: job:http_requests_total:rate5m
      expr: sum(rate(http_requests_total[5m])) by (job)
      
    - record: job:http_errors_total:rate5m
      expr: sum(rate(http_requests_total{status=~"5.."}[5m])) by (job)
      
    - record: job:http_latency:p95
      expr: histogram_quantile(0.95, sum(rate(http_request_duration_seconds_bucket[5m])) by (job, le))
```

## トラブルシューティング

### よくある問題と解決方法

#### 1. Prometheusのメモリ不足
症状：
- Prometheusポッドが再起動を繰り返す
- OOMKilled エラーが発生

解決方法：
```yaml
prometheus:
  prometheusSpec:
    resources:
      requests:
        memory: "4Gi"
      limits:
        memory: "8Gi"
    # メモリ使用量を抑えるための設定
    query:
      maxSamples: 50000000
      timeout: 2m
      maxConcurrency: 20
```

#### 2. メトリクスの収集遅延
症状：
- メトリクスの更新が遅い
- スクレイプのタイムアウトエラー

解決方法：
```yaml
prometheus:
  prometheusSpec:
    scrapeInterval: 30s
    scrapeTimeout: 20s
    evaluationInterval: 30s
    # スクレイプのパフォーマンス向上
    additionalArgs:
      - --storage.tsdb.min-block-duration=2h
      - --storage.tsdb.max-block-duration=2h
```

#### 3. Grafanaのダッシュボード読み込み遅延
症状：
- ダッシュボードの読み込みが遅い
- タイムアウトエラー

解決方法：
```yaml
grafana:
  grafana.ini:
    panels:
      disable_sanitize_html: true
    dataproxy:
      timeout: 300
      dial_timeout: 10
      keep_alive_seconds: 300
    database:
      cache_mode: "server"
      cache_cleanup_job_period: 60
```

#### 4. AlertManagerの通知問題
症状：
- アラート通知が届かない
- 重複通知が発生

解決方法：
```yaml
alertmanager:
  config:
    route:
      group_by: ['alertname', 'cluster', 'service']
      group_wait: 30s
      group_interval: 5m
      repeat_interval: 4h
      receiver: 'slack'
    receivers:
    - name: 'slack'
      slack_configs:
      - channel: '#alerts'
        send_resolved: true
        api_url: 'https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK'
        title: |-
          [{{ .Status | toUpper }}{{ if eq .Status "firing" }}:{{ .Alerts.Firing | len }}{{ end }}] {{ .CommonLabels.alertname }}
        text: >-
          {{ range .Alerts }}
            *Alert:* {{ .Annotations.summary }}
            *Description:* {{ .Annotations.description }}
            *Severity:* {{ .Labels.severity }}
            *Instance:* {{ .Labels.instance }}
            *Duration:* {{ duration .StartsAt .EndsAt }}
          {{ end }}
```

## ベストプラクティスを考えてみましょう

### 1. リソース管理
- Prometheusのストレージサイズは、保持期間とメトリクス量に基づいて計算
- メモリ使用量は収集するメトリクス数に比例して増加
- CPU使用量はクエリの複雑さと頻度に依存

### 2. パフォーマンス最適化
- 不要なメトリクスのドロップ
- レコーディングルールの活用
- 効率的なラベル設計

### 3. セキュリティ
- Grafanaの認証設定
- メトリクスエンドポイントのセキュリティ
- RBAC設定の適切な管理

### 4. バックアップと復旧
- Prometheusデータの定期バックアップ
- Grafanaダッシュボードの設定バックアップ
- 障害復旧手順の文書化

## 参考資料

### 公式ドキュメント
- [Prometheus Official Documentation](https://prometheus.io/docs/introduction/overview/)
- [Grafana Documentation](https://grafana.com/docs/)
- [kube-prometheus-stack GitHub](https://github.com/prometheus-community/helm-charts/tree/main/charts/kube-prometheus-stack)
- [Prometheus Operator Documentation](https://prometheus-operator.dev/)

### チュートリアルとガイド
- [Prometheus Best Practices](https://prometheus.io/docs/practices/naming/)
- [Grafana Dashboard Best Practices](https://grafana.com/docs/grafana/latest/best-practices/)
- [PromQL Tutorial](https://prometheus.io/docs/prometheus/latest/querying/basics/)
- [AlertManager Configuration Guide](https://prometheus.io/docs/alerting/latest/configuration/)

### コミュニティリソース
- [Awesome Prometheus](https://github.com/roaldnefs/awesome-prometheus)
- [Grafana Labs Blog](https://grafana.com/blog/)
- [Prometheus Community GitHub](https://github.com/prometheus-community)
- [CNCF Prometheus Project](https://www.cncf.io/projects/prometheus/)

### 有用なツール
- [PromQL Query Builder](https://prometheus.io/docs/visualization/browser/)
- [Grafana Dashboard Gallery](https://grafana.com/grafana/dashboards/)
- [kube-prometheus-stack Values Generator](https://artifacthub.io/packages/helm/prometheus-community/kube-prometheus-stack)
- [Prometheus Config Validator](https://prometheus.io/docs/prometheus/latest/configuration/unit_testing_rules/)
