# OpenTelemetryによる分散トレーシングの実装ガイド

## 目次
1. [概要](#概要)
2. [アプリケーションへのOpenTelemetry実装](#アプリケーションへのopentelemetry実装)
3. [OpenTelemetry Collectorの設定](#opentelemetry-collectorの設定)
4. [バックエンドの設定](#バックエンドの設定)
5. [課題](#課題)
6. [高度な設定](#高度な設定)
7. [トラブルシューティング](#トラブルシューティング)
8. [参考資料](#参考資料)

## 概要

### OpenTelemetryとは
OpenTelemetryは、分散システムにおけるトレース、メトリクス、ログを統合的に扱うためのオープンソースフレームワークです。本ガイドでは、GoアプリケーションにOpenTelemetryを実装し、分散トレーシングを実現する方法を説明します。

### アーキテクチャ
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Application    │    │   Collector     │    │    Backend      │
│  with OTel SDK  │───►│  (Processing)   │───►│   (Storage)     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## アプリケーションへのOpenTelemetry実装

### 1. 必要なパッケージの追加
```go
import (
    "context"
    "go.opentelemetry.io/otel"
    "go.opentelemetry.io/otel/attribute"
    "go.opentelemetry.io/otel/exporters/otlp/otlptrace"
    "go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc"
    "go.opentelemetry.io/otel/sdk/resource"
    sdktrace "go.opentelemetry.io/otel/sdk/trace"
    semconv "go.opentelemetry.io/otel/semconv/v1.21.0"
    "go.opentelemetry.io/otel/trace"
    "go.opentelemetry.io/contrib/instrumentation/net/http/otelhttp"
)
```

### 2. トレーサープロバイダーの初期化
```go
func initTracer() (*sdktrace.TracerProvider, error) {
    ctx := context.Background()

    // Collectorへの接続設定
    exporter, err := otlptrace.New(ctx,
        otlptracegrpc.NewClient(
            otlptracegrpc.WithInsecure(),
            otlptracegrpc.WithEndpoint("otel-collector:4317"),
        ),
    )
    if err != nil {
        return nil, err
    }

    // リソース属性の設定
    res, err := resource.New(ctx,
        resource.WithAttributes(
            semconv.ServiceName("workshop-app"),
            semconv.ServiceVersion("1.0.0"),
            attribute.String("environment", "production"),
        ),
    )
    if err != nil {
        return nil, err
    }

    // TracerProviderの設定
    tp := sdktrace.NewTracerProvider(
        sdktrace.WithBatcher(exporter),
        sdktrace.WithResource(res),
        sdktrace.WithSampler(sdktrace.AlwaysSample()),
    )

    otel.SetTracerProvider(tp)
    return tp, nil
}
```

### 3. メイン関数の修正
```go
func main() {
    tp, err := initTracer()
    if err != nil {
        log.Fatal(err)
    }
    defer func() {
        if err := tp.Shutdown(context.Background()); err != nil {
            log.Printf("Error shutting down tracer provider: %v", err)
        }
    }()

    // ハンドラーのラップ
    http.Handle("/", otelhttp.NewHandler(http.HandlerFunc(handler), "root"))
    http.Handle("/health", otelhttp.NewHandler(http.HandlerFunc(healthHandler), "health"))
    http.Handle("/time", otelhttp.NewHandler(http.HandlerFunc(timeHandler), "time"))
    http.Handle("/env", otelhttp.NewHandler(http.HandlerFunc(envHandler), "env"))

    http.ListenAndServe(":8080", nil)
}
```

### 4. ハンドラー関数の修正
```go
func handler(w http.ResponseWriter, r *http.Request) {
    ctx := r.Context()
    span := trace.SpanFromContext(ctx)
    defer span.End()

    span.SetAttributes(attribute.String("handler", "root"))

    msg := os.Getenv("APP_MESSAGE")
    if msg == "" {
        msg = "Welcome to Workshop 2025!"
        span.SetAttributes(attribute.String("message_source", "default"))
    } else {
        span.SetAttributes(attribute.String("message_source", "env"))
    }

    fmt.Fprintf(w, msg)
}

func timeHandler(w http.ResponseWriter, r *http.Request) {
    ctx := r.Context()
    span := trace.SpanFromContext(ctx)
    defer span.End()

    currentTime := time.Now().Format(time.RFC3339)
    span.SetAttributes(attribute.String("time", currentTime))

    fmt.Fprintf(w, "Current Server Time: %s", currentTime)
}

func envHandler(w http.ResponseWriter, r *http.Request) {
    ctx := r.Context()
    span := trace.SpanFromContext(ctx)
    defer span.End()

    envVars := make(map[string]string)
    for _, e := range os.Environ() {
        pair := strings.SplitN(e, "=", 2)
        envVars[pair[0]] = pair[1]
    }

    span.SetAttributes(attribute.Int("env_vars_count", len(envVars)))

    w.Header().Set("Content-Type", "application/json")
    json.NewEncoder(w).Encode(envVars)
}

func healthHandler(w http.ResponseWriter, r *http.Request) {
    ctx := r.Context()
    span := trace.SpanFromContext(ctx)
    defer span.End()

    w.WriteHeader(http.StatusOK)
    currentTime := time.Now().Format(time.RFC3339)
    span.SetAttributes(attribute.String("health_check_time", currentTime))

    fmt.Fprintf(w, "Healthy: %s", currentTime)
}
```

## OpenTelemetry Collectorの設定

### Collectorのデプロイ
```yaml
apiVersion: opentelemetry.io/v1alpha1
kind: OpenTelemetryCollector
metadata:
  name: otel-collector
spec:
  mode: deployment
  config: |
    receivers:
      otlp:
        protocols:
          grpc:
            endpoint: ":4317"
          http:
            endpoint: ":4318"

    processors:
      batch:
        timeout: 1s
        send_batch_size: 1024
      memory_limiter:
        check_interval: 1s
        limit_mib: 1500
        spike_limit_mib: 512
      resourcedetection:
        detectors: [env, system]
        timeout: 2s
      attributes:
        actions:
          - key: environment
            value: production
            action: insert
      tail_sampling:
        policies:
          - name: error-sampling
            type: status_code
            sampling_percentage: 100
            status_code: ERROR
          - name: default-policy
            type: probabilistic
            sampling_percentage: 10

    exporters:
      jaeger:
        endpoint: jaeger-collector:14250
        tls:
          insecure: true
      logging:
        loglevel: debug
        sampling_initial: 5
        sampling_thereafter: 200

    service:
      pipelines:
        traces:
          receivers: [otlp]
          processors: [memory_limiter, batch, resourcedetection, attributes, tail_sampling]
          exporters: [jaeger, logging]
      telemetry:
        logs:
          level: debug
        metrics:
          address: ":8888"
```

### Jaegerのデプロイ
```yaml
apiVersion: jaegertracing.io/v1
kind: Jaeger
metadata:
  name: jaeger
spec:
  strategy: production
  storage:
    type: elasticsearch
    options:
      es:
        server-urls: http://elasticsearch:9200
  ingress:
    enabled: true
    hosts:
      - jaeger.example.com
```

## 課題

### 1.トレーシングの実装
- Goのアプリケーションの関数にトレースを実装してください。

### 2. 表示情報の追加
- トレースに以下の情報を追加してください。

## 高度な設定

### カスタムサンプリングの実装
```go
type CustomSampler struct {
    sdktrace.Sampler
    sampleRate float64
}

func (cs *CustomSampler) ShouldSample(p sdktrace.SamplingParameters) sdktrace.SamplingResult {
    if strings.Contains(p.Name, "health") {
        return sdktrace.SamplingResult{Decision: sdktrace.Drop}
    }
    
    if rand.Float64() < cs.sampleRate {
        return sdktrace.SamplingResult{Decision: sdktrace.RecordAndSample}
    }
    
    return sdktrace.SamplingResult{Decision: sdktrace.Drop}
}

// 使用例
tp := sdktrace.NewTracerProvider(
    sdktrace.WithSampler(&CustomSampler{sampleRate: 0.1}),
    // 他の設定...
)
```

### バッチ処理の最適化
```go
// バッチエクスポーターの設定
exporter, err := otlptrace.New(ctx,
    otlptracegrpc.NewClient(
        otlptracegrpc.WithEndpoint("otel-collector:4317"),
        otlptracegrpc.WithInsecure(),
    ),
)

// バッチスパンプロセッサーの設定
bsp := sdktrace.NewBatchSpanProcessor(
    exporter,
    sdktrace.WithBatchTimeout(5*time.Second),
    sdktrace.WithMaxQueueSize(2048),
    sdktrace.WithMaxExportBatchSize(512),
)

tp := sdktrace.NewTracerProvider(
    sdktrace.WithSpanProcessor(bsp),
    // 他の設定...
)
```

### コンテキスト伝播の設定
```go
import (
    "go.opentelemetry.io/otel/propagation"
)

// プロパゲーターの設定
otel.SetTextMapPropagator(propagation.NewCompositeTextMapPropagator(
    propagation.TraceContext{},
    propagation.Baggage{},
))

// HTTPクライアントでの使用例
client := &http.Client{
    Transport: otelhttp.NewTransport(
        http.DefaultTransport,
        otelhttp.WithPropagators(
            propagation.NewCompositeTextMapPropagator(
                propagation.TraceContext{},
                propagation.Baggage{},
            ),
        ),
    ),
}
```

## トラブルシューティング

### よくある問題と解決方法

#### 1. トレースデータが表示されない
症状：
- Jaegerにトレースが表示されない
- Collectorのログにエラーがない

解決方法：
1. サンプリング設定の確認
2. Collectorとの接続確認
3. エクスポーターの設定確認

```go
// デバッグ用のログエクスポーターを追加
logExporter := stdouttrace.New(
    stdouttrace.WithPrettyPrint(),
)

tp := sdktrace.NewTracerProvider(
    sdktrace.WithBatcher(otlpExporter),
    sdktrace.WithBatcher(logExporter),
    // 他の設定...
)
```

#### 2. パフォーマンスの問題
症状：
- アプリケーションの応答が遅い
- メモリ使用量が高い

解決方法：
1. バッチ設定の調整
2. サンプリングレートの調整
3. メモリ制限の設定

```yaml
processors:
  batch:
    timeout: 1s
    send_batch_size: 512
  memory_limiter:
    limit_mib: 1000
    spike_limit_mib: 200
```

#### 3. コンテキスト伝播の問題
症状：
- マイクロサービス間でトレースが途切れる
- トレースIDが継承されない

解決方法：
1. プロパゲーターの設定確認
2. ヘッダーの確認
3. ミドルウェアの設定確認

## ベストプラクティス

### 1. スパン管理
- 適切なスパン名の設定
- 重要な属性の追加
- エラーハンドリングの実装

### 2. パフォーマンス最適化
- 適切なサンプリングレートの設定
- バッチ処理の最適化
- リソース使用量の監視

### 3. セキュリティ
- 機密情報の除外
- 認証・認可の設定
- TLS/SSL通信の設定

### 4. 監視とメンテナンス
- ログレベルの適切な設定
- メトリクスの監視
- 定期的な設定の見直し

## 参考資料

### 公式ドキュメント
- [OpenTelemetry Documentation](https://opentelemetry.io/docs/)
- [OpenTelemetry Go Documentation](https://pkg.go.dev/go.opentelemetry.io/otel)
- [OpenTelemetry Collector Documentation](https://opentelemetry.io/docs/collector/)
- [Jaeger Documentation](https://www.jaegertracing.io/docs/)

### チュートリアルとガイド
- [OpenTelemetry Go Getting Started](https://opentelemetry.io/docs/languages/go/getting-started/)
- [Distributed Tracing with OpenTelemetry](https://www.cncf.io/blog/2020/12/02/getting-started-with-opentelemetry/)
- [OpenTelemetry Best Practices](https://opentelemetry.io/docs/concepts/sdk-configuration/)
- [OpenTelemetry Collector Configuration](https://opentelemetry.io/docs/collector/configuration/)
- [Sampling in OpenTelemetry](https://opentelemetry.io/docs/concepts/sampling/)

### コミュニティリソース
- [OpenTelemetry GitHub](https://github.com/open-telemetry)
- [OpenTelemetry Community](https://opentelemetry.io/community/)
- [CNCF OpenTelemetry Project](https://www.cncf.io/projects/opentelemetry/)
- [OpenTelemetry Discussions](https://github.com/open-telemetry/community/discussions)

### 有用なツール
- [OpenTelemetry Playground](https://playground.opentelemetry.io/)
- [Jaeger UI](https://www.jaegertracing.io/docs/1.41/frontend-ui/)
- [OpenTelemetry Registry](https://opentelemetry.io/registry/)
- [OpenTelemetry Demo Applications](https://github.com/open-telemetry/opentelemetry-demo)

## アプリケーション開発のベストプラクティス

### トレース情報の効果的な収集

#### 1. 適切なスパン粒度の設定
```go
func complexOperation(ctx context.Context) error {
    tr := otel.Tracer("complex-operation")
    ctx, span := tr.Start(ctx, "complex-operation")
    defer span.End()

    // サブ操作のトレース
    if err := subOperation1(ctx); err != nil {
        span.SetStatus(codes.Error, "sub-operation-1 failed")
        span.RecordError(err)
        return err
    }

    if err := subOperation2(ctx); err != nil {
        span.SetStatus(codes.Error, "sub-operation-2 failed")
        span.RecordError(err)
        return err
    }

    return nil
}
```

#### 2. 有用な属性の追加
```go
func processRequest(ctx context.Context, req *http.Request) {
    span := trace.SpanFromContext(ctx)
    
    // リクエスト情報の記録
    span.SetAttributes(
        attribute.String("user.id", req.Header.Get("X-User-ID")),
        attribute.String("request.method", req.Method),
        attribute.String("request.path", req.URL.Path),
        attribute.String("request.remote_addr", req.RemoteAddr),
    )

    // カスタムビジネスロジック属性
    span.SetAttributes(
        attribute.String("business.transaction_id", generateTransactionID()),
        attribute.String("business.customer_type", determineCustomerType(req)),
    )
}
```

#### 3. エラーハンドリングの統合
```go
func handleDatabaseOperation(ctx context.Context) error {
    span := trace.SpanFromContext(ctx)
    
    result, err := db.Query(ctx, "SELECT * FROM users")
    if err != nil {
        // エラー情報をスパンに記録
        span.SetStatus(codes.Error, fmt.Sprintf("database query failed: %v", err))
        span.RecordError(err, trace.WithAttributes(
            attribute.String("error.type", "database_error"),
            attribute.String("query.type", "select"),
        ))
        return err
    }

    span.SetAttributes(attribute.Int("result.count", len(result)))
    return nil
}
```

## セキュリティとプライバシーの考慮事項

### 機密情報の保護
1. 個人情報や機密データをトレース属性から除外
2. 必要に応じてデータのマスキング
3. アクセス制御の実装

```go
func sanitizeUserData(data map[string]string) map[string]string {
    sensitiveFields := []string{"password", "credit_card", "ssn"}
    sanitized := make(map[string]string)
    
    for k, v := range data {
        if contains(sensitiveFields, k) {
            sanitized[k] = "***REDACTED***"
        } else {
            sanitized[k] = v
        }
    }
    
    return sanitized
}
```

### セキュアな通信の設定
```go
// TLSを使用したCollectorへの接続
exporter, err := otlptrace.New(ctx,
    otlptracegrpc.NewClient(
        otlptracegrpc.WithEndpoint("otel-collector:4317"),
        otlptracegrpc.WithTLSCredentials(credentials.NewClientTLSFromCert(nil, "")),
    ),
)
```

## パフォーマンスチューニング

### 1. サンプリング戦略の最適化
```go
// 動的サンプリングの実装
type DynamicSampler struct {
    baseRate    float64
    currentLoad int64
}

func (s *DynamicSampler) ShouldSample(p sdktrace.SamplingParameters) sdktrace.SamplingResult {
    load := atomic.LoadInt64(&s.currentLoad)
    rate := s.baseRate
    
    if load > threshold {
        rate = s.baseRate * 0.5
    }
    
    if rand.Float64() < rate {
        return sdktrace.SamplingResult{Decision: sdktrace.RecordAndSample}
    }
    
    return sdktrace.SamplingResult{Decision: sdktrace.Drop}
}
```

### 2. バッチ処理の最適化
```go
// 最適化されたバッチ設定
bsp := sdktrace.NewBatchSpanProcessor(
    exporter,
    sdktrace.WithBatchTimeout(2*time.Second),
    sdktrace.WithMaxQueueSize(4096),
    sdktrace.WithMaxExportBatchSize(512),
    sdktrace.WithExportTimeout(30*time.Second),
)
```

## 終わりに

OpenTelemetryを使用した分散トレーシングの実装は、マイクロサービスアーキテクチャにおいて重要な観測可能性を提供します。本ガイドで紹介した実装方法とベストプラクティスを参考に、自身のアプリケーションに適した形で導入を検討してください。

定期的に以下の点を見直すことをお勧めします：

1. トレース情報の有用性
2. パフォーマンスへの影響
3. サンプリング戦略の適切性
4. セキュリティ設定の妥当性

これらの要素を継続的に評価・改善することで、より効果的な分散トレーシングの実装が可能となります。
