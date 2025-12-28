## 基本的な考え方：2025年のメンタルモデル

### マルチステージビルドは前提条件

2025年においてマルチステージビルドを使わないDockerfileは、もはや考えられません。

- **メンテナンス性向上**：最終的な成果物以外ではサイズを意識したトリッキーな記述が不要
- **ビルド速度アップ**：並列化、キャッシュマウント、tmpfsなど最適化オプションが豊富
- **セキュリティ向上**：シークレット管理の仕組みが標準化

[https://docs.docker.com/build/building/multi-stage/:embed:cite]

### COPYは最小限に、--mountを活用

`COPY`が登場するのは以下の2つの場合のみです。

1. マルチステージビルドで別ステージから成果物を持ってくる場合
2. 最終ステージでアプリケーションバイナリをコピーする場合

ソースコードのビルド時には`--mount=type=bind`を使用します。

[https://docs.docker.com/build/guide/mounts/:embed:cite]

### 必ず記述すべきディレクティブ

```dockerfile
# syntax=docker/dockerfile:1
```

この1行を**必ず先頭に記述**します。最新のDockerfile構文が自動的に利用され、新機能が使えるようになります。

[https://docs.docker.com/build/buildkit/dockerfile-frontend/:embed:cite]

