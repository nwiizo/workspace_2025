# 信頼性の高いコンテナイメージの利用

# 目的

コンテナイメージのセキュリティとサプライチェーンの完全性を確保するため、信頼性の高いコンテナイメージを選択・利用する方法を提示します。

## 課題定義

コンテナイメージの利用において以下の課題が存在します。

- **攻撃対象領域の増大**: 標準的なコンテナイメージには不要なパッケージやツールが含まれており、脆弱性の原因となる可能性がある
- **サプライチェーンの脆弱性**: イメージの構成要素や出所が不明確な場合が多く、改ざんリスクが存在する
- **イメージの肥大化**: 不要なコンポーネントによりイメージサイズが増大し、効率性が低下する
- **脆弱性管理の複雑さ**: 多数のパッケージやライブラリを含むイメージは脆弱性の特定と修正が困難になる
- **透明性の欠如**: イメージの内容を完全に把握することが難しく、セキュリティ監査やコンプライアンス対応が複雑になる

## 解決方法

上記の課題に対して、以下の方法で信頼性の高いコンテナイメージを利用します。

1. **公式または信頼性の高い組織が提供するイメージの利用**:
    - [Chainguard Images](https://www.chainguard.dev/chainguard-images): セキュリティを重視した最小限のコンテナイメージ
    - [Docker Trusted Content](https://hub.docker.com/search?q=&type=image&image_filter=official): Docker社が検証したイメージ
2. **ディストリビューションを含まないコンテナイメージの利用**:
    - [Distroless](https://github.com/GoogleContainerTools/distroless): 必要最小限のコンポーネントのみを含むイメージ

これらのアプローチにより、攻撃対象領域の削減、脆弱性の最小化、イメージサイズの最適化、サプライチェーンの透明性向上が達成できます。

## 性能指針

信頼性の高いコンテナイメージを利用することで、以下の性能向上が期待できます。

- **イメージサイズの縮小**: 最小限のコンポーネントのみを含むことで、通常のコンテナイメージと比較して50〜90%のサイズ削減が可能
- **起動時間の短縮**: 小さいイメージは素早く転送・展開でき、コンテナ起動時間が短縮される
- **リソース効率の向上**: 不要なプロセスやサービスが含まれないため、メモリ使用量が削減される
- **セキュリティパッチの効率化**: 必要なコンポーネントのみを更新すればよいため、パッチ適用プロセスが簡素化される

特定の数値は環境やワークロードによって異なるため定量的な指標は示さないが、最小限のイメージを使用することで全体的な効率が向上します。

# 導入するプログラム

## Chainguard Images

[Chainguard社](https://www.chainguard.dev/about-us)が提供するdistrolessを中心としたセキュアなコンテナイメージ群。[Wolfi OS](https://edu.chainguard.dev/open-source/wolfi/overview/)というLinux undistroをベースにしており、SBOM（Software Bill of Materials）の提供、最小限のパッケージ、検証可能な署名などの特徴を持ちます。

- 詳細情報: [Chainguard Images Overview](https://edu.chainguard.dev/chainguard/chainguard-images/overview/)
- イメージリファレンス: [Chainguard Images Reference](https://edu.chainguard.dev/chainguard/chainguard-images/reference/)
- GitHub: [Chainguard Images Repository](https://github.com/chainguard-images/images)

## Docker Trusted Content

Docker社が提供する公式イメージプログラム。セキュリティスキャン、脆弱性修正、定期的な更新が保証されている信頼性の高いコンテナイメージです。

- 公式イメージ一覧: [Docker Hub Official Images](https://hub.docker.com/search?q=&type=image&image_filter=official)
- ドキュメント: [Docker Official Images Documentation](https://docs.docker.com/docker-hub/official_images/)

## Distroless

[Google](https://github.com/GoogleContainerTools)が提供するDistrolessイメージは、アプリケーションとそのランタイム依存関係のみを含み、パッケージマネージャやシェルなどの不要なコンポーネントを排除したイメージです。

- GitHub: [GoogleContainerTools/distroless](https://github.com/GoogleContainerTools/distroless)
- イメージリポジトリ: [gcr.io/distroless](https://console.cloud.google.com/gcr/images/distroless)
- ドキュメント: [Distroless Container Images](https://github.com/GoogleContainerTools/distroless/blob/main/README.md)

## 選定理由

以下の理由から上記のイメージを選定しました：

- **セキュリティ強化**: 最小限のコンポーネントにより攻撃対象領域が削減され、既知の脆弱性を含むパッケージが少ない
- **透明性**: SBOMの提供により、イメージに含まれるコンポーネントの完全な把握が可能
- **検証可能性**: 署名メカニズムによりイメージの完全性が検証可能
- **効率性**: 必要最小限のコンポーネントのみを含むため、リソース効率が高い
- **信頼性**: 公式または信頼性の高い組織によるサポートと定期的な更新があり、継続的なセキュリティ対応が期待できる

## 課題

以下の課題も存在します。

- **学習コスト**: 従来の完全なLinuxディストリビューションと比較して、操作方法やデバッグ方法が異なる
- **デバッグの難しさ**: シェルなどの標準ツールが含まれていないため、コンテナ内でのデバッグが困難
- **一部ツールとの互換性**: 特定のアプリケーションやツールがベースとなるシステムライブラリに依存している場合、互換性の問題が発生する可能性がある
- **エンタープライズサポートの制限**: 一部のイメージでは、エンタープライズサポートが必要な場合がある（例：[Chainguard Imagesの特定バージョンタグの利用](https://www.chainguard.dev/software-license-agreement)）

## 検討したツール

- **[Alpine Linux](https://alpinelinux.org/)**: 軽量なイメージとして広く利用されているが、musl libcの使用による互換性の問題や、一部のセキュリティ機能の欠如から、より包括的なソリューションを選択した
- **カスタムベースイメージ**: 独自のセキュアなベースイメージの構築も検討したが、継続的なメンテナンスとセキュリティ対応のコストが高く、専門知識も必要となるため採用しなかった
- **[UBI (Universal Base Image)](https://www.redhat.com/en/blog/introducing-red-hat-universal-base-image)**: Red Hatが提供するイメージで、商用サポートがあるが、イメージサイズが比較的大きく、ライセンスの制約がある

# 全体構成

信頼性の高いコンテナイメージを利用するための全体構成は以下の通り：

1. **ベースイメージの選択**:
    - 公式イメージ（[Docker Trusted Content](https://hub.docker.com/search?q=&type=image&image_filter=official)）
    - 最小限のイメージ（[Distroless](https://github.com/GoogleContainerTools/distroless), [Chainguard Images](https://www.chainguard.dev/chainguard-images)）
2. **イメージビルドプロセス**:
    - [マルチステージビルド](https://docs.docker.com/build/building/multi-stage/)
    - 依存関係の最小化
    - 不要なファイルの除外
3. **イメージ検証**:
    - 署名検証（[Cosign](https://github.com/sigstore/cosign), [Notary](https://github.com/notaryproject/notary)）
    - SBOM確認（[Syft](https://github.com/anchore/syft), [SPDX](https://spdx.dev/)）
    - 脆弱性スキャン（[Trivy](https://github.com/aquasecurity/trivy), [Grype](https://github.com/anchore/grype)）
4. **デプロイメントポリシー**:
    - [イメージポリシー適用](https://kubernetes.io/docs/reference/access-authn-authz/admission-controllers/#imagepolicywebhook)
    - イミュータブルタグの使用
    - CI/CDパイプラインとの統合

## 用語定義

- **Distroless**: 従来のLinuxディストリビューションに含まれる不要なコンポーネント（シェル、パッケージマネージャなど）を除いたコンテナイメージ
- **SBOM (Software Bill of Materials)**: ソフトウェアの構成要素一覧。コンテナイメージに含まれるすべてのコンポーネントとそのバージョンを記録
- **[Wolfi OS](https://edu.chainguard.dev/open-source/wolfi/overview/)**: Chainguard社が開発したコンテナ・クラウドネイティブ用途向けのLinux undistro
- **[melange](https://github.com/chainguard-dev/melange)**: apkパッケージを宣言的にビルドするツール
- **[apko](https://github.com/chainguard-dev/apko)**: DockerfileなしでapkパッケージからOCIイメージを直接ビルドするツール
- **[Cosign](https://github.com/sigstore/cosign)**: コンテナイメージに対する署名と検証を行うツール
- **マルチステージビルド**: ビルド環境と実行環境を分離するDockerfileの記述方法
- **UBI (Universal Base Image)**: Red Hatが提供するベースイメージ

## 命名規約

特になし。各イメージプロバイダの命名規則に従うことを推奨します。

# 可用性

コンテナイメージ自体には高可用性の概念は直接適用されないが、以下の点に注意することで可用性を確保できる：

- **複数のレジストリミラーの利用**: イメージを複数のレジストリにミラーリングし、単一障害点を排除する
- **イメージのキャッシング**: [ノードローカルまたはプロキシキャッシュ](https://kubernetes.io/docs/concepts/containers/images/#using-a-private-registry)を使用して、レジストリの可用性に依存しない構成を構築する
- **イメージのバージョン固定**: 特定の[ダイジェスト](https://github.com/opencontainers/image-spec/blob/main/descriptor.md#digests)を指定することで、イメージの変更による問題を防止する
- **フォールバックメカニズム**: プライマリイメージが利用できない場合に備えて、フォールバックイメージを設定する

[Chainguard Images](https://www.chainguard.dev/chainguard-images)の場合、エンタープライズサポートに加入することで、SLAに基づくサポートを受けることが可能です。

# 拡張性

信頼性の高いコンテナイメージの利用は、以下の点で良好なスケーラビリティを提供します。

- **イメージサイズの最適化**: 小さいイメージサイズにより、多数のコンテナを効率的にデプロイ可能
- **カスタマイズの柔軟性**: [Chainguard Images](https://www.chainguard.dev/chainguard-images)や[Distroless](https://github.com/GoogleContainerTools/distroless)は、必要に応じて特定の用途向けにカスタマイズ可能
- **CI/CDとの統合**: 自動化されたビルドパイプラインと統合することで、多数のイメージを効率的に管理可能
- **イメージレイヤの共有**: 共通のベースレイヤを持つことで、ストレージとネットワーク転送の効率が向上する

スケールに関する制限は主にイメージレジストリの性能に依存するが、多くの場合、イメージ自体のスケーラビリティは問題にならない。

# セキュリティ

信頼性の高いコンテナイメージを利用することで、以下のセキュリティ上の利点が得られます：

## 攻撃対象領域の縮小

- 不要なパッケージやツールを含まないため、悪用可能なコンポーネントが少ない
- シェルやパッケージマネージャが含まれていないため、侵入後の攻撃者の活動が制限される
- 最小限のコンポーネントのみを含むため、脆弱性の総数が減少する

## サプライチェーンセキュリティ

- [署名検証](https://github.com/sigstore/cosign)により、イメージの出所と完全性を確認可能
- [SBOM](https://www.cisa.gov/sbom)により、含まれるコンポーネントを透明化
- 信頼できるソースからのイメージ提供により、サプライチェーン攻撃のリスクを低減

## 脆弱性管理

- 最小限のコンポーネントのみを含むため、パッチ適用が容易
- 自動更新メカニズムにより、最新のセキュリティパッチを適用
- 脆弱性スキャンの結果が単純になり、優先順位付けが容易になる

## コンプライアンス

- イメージの内容が明確であるため、コンプライアンス要件への適合が容易
- 監査証跡が提供され、セキュリティ検証プロセスが簡素化
- ライセンスコンプライアンスの確認が容易になる

# 詳細

## Chainguard Images

[Chainguard Images](https://www.chainguard.dev/chainguard-images)は、Chainguard社が提供するdistrolessを中心としたセキュアなコンテナイメージ群です。[Wolfi OS](https://edu.chainguard.dev/open-source/wolfi/overview/)というLinux undistroをベースとしており、以下の特徴を持ちます。

### 主な特徴

- [SBOMをすべてのパッケージに標準で提供](https://edu.chainguard.dev/chainguard/chainguard-images/overview/)
- 必要最小限のパッケージのみを使用
- apkパッケージ形式を採用
- [melange](https://github.com/chainguard-dev/melange)/[apko](https://github.com/chainguard-dev/apko)という宣言的なツールでビルド
- [Sigstore](https://edu.chainguard.dev/open-source/sigstore/cosign/an-introduction-to-cosign/)による検証可能な署名
- 自動化されたビルドによるセキュリティパッチの適用

### Wolfi OSの特徴

[Wolfi OS](https://edu.chainguard.dev/open-source/wolfi/overview/)はChainguard社によって開発されたコンテナ・クラウドネイティブ用途向けのLinux undistroであり、以下の特徴を持ちます。

- [apkパッケージマネージャー](https://edu.chainguard.dev/open-source/wolfi/apk-package-manager/)を使用
- glibc/muslの両方をサポート
- [melange](https://github.com/chainguard-dev/melange)/[apko](https://github.com/chainguard-dev/apko)ツールでパッケージ管理とイメージビルド
- コンテナ環境に最適化された設計

### 提供されているイメージ

[Chainguard Images](https://edu.chainguard.dev/chainguard/chainguard-images/reference/)では以下のような様々なカテゴリのイメージが提供されています：

- 開発者向け: [Go](https://edu.chainguard.dev/chainguard/chainguard-images/reference/go/), [Ruby](https://edu.chainguard.dev/chainguard/chainguard-images/reference/ruby/), [PHP](https://edu.chainguard.dev/chainguard/chainguard-images/reference/php/) など
- ミドルウェア: [etcd](https://edu.chainguard.dev/chainguard/chainguard-images/reference/etcd/), [MariaDB](https://edu.chainguard.dev/chainguard/chainguard-images/reference/mariadb/) など
- CLI: [aws-cli](https://edu.chainguard.dev/chainguard/chainguard-images/reference/aws-cli/), [kubectl](https://edu.chainguard.dev/chainguard/chainguard-images/reference/kubectl/) など
- Kubernetes Operator: [aws-ebs-csi-driver](https://edu.chainguard.dev/chainguard/chainguard-images/reference/aws-ebs-csi-driver/), [calico-kube-controllers](https://edu.chainguard.dev/chainguard/chainguard-images/reference/calico-kube-controllers/) など

GoogleのDistrolessイメージが提供する種類と比較して、より多くの種類のコンテナイメージが提供されています。

### サポート

[Chainguard Images](https://www.chainguard.dev/chainguard-images)には以下のようなサポートオプションがあります：

- フリーティア: latestタグのみ利用可能
- [エンタープライズサポート](https://www.chainguard.dev/software-license-agreement): メジャー/マイナーバージョンタグの利用、SLA、カスタマーサポートが含まれる

### イメージビルドの再現

[Chainguard Images](https://www.chainguard.dev/chainguard-images)の特徴として、イメージビルドの再現性があります。以下のコマンドでイメージビルドを再現できます：

```bash
# イメージ構成を取得
IMAGE_NAME=cgr.dev/chainguard/wolfi-base
cosign verify-attestation \
  --type https://apko.dev/image-configuration \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity https://github.com/chainguard-images/images/.github/workflows/release.yaml@refs/heads/main \
  "${IMAGE_NAME}" | jq -r .payload | base64 -d | jq .predicate > latest.apko.json

# 構成からイメージをビルド
apko build latest.apko.json wolfi-base:local wolfi-base.tar
```

詳細は[こちらのドキュメント](https://www.chainguard.dev/unchained/reproducing-chainguards-reproducible-image-builds)を参照してください。

## Docker Trusted Content

[Docker社が提供する公式イメージプログラム](https://hub.docker.com/search?q=&type=image&image_filter=official)で、セキュリティとベストプラクティスに準拠したイメージを提供します。

### 主な特徴

- Docker社による検証と認証
- 定期的なセキュリティスキャンと更新
- 透明性の高いビルドプロセス
- 最新のセキュリティパッチの適用
- 広範なドキュメント提供

### 提供されているイメージ

[Docker Trusted Content](https://hub.docker.com/search?q=&type=image&image_filter=official)では以下のような様々なカテゴリのイメージが提供されています：

- 言語ランタイム: [Node.js](https://hub.docker.com/_/node), [Python](https://hub.docker.com/_/python), [Java](https://hub.docker.com/_/openjdk) など
- データベース: [MySQL](https://hub.docker.com/_/mysql), [PostgreSQL](https://hub.docker.com/_/postgres), [MongoDB](https://hub.docker.com/_/mongo) など
- ウェブサーバー: [Nginx](https://hub.docker.com/_/nginx), [Apache](https://hub.docker.com/_/httpd) など
- その他の一般的なアプリケーションとサービス

### セキュリティのメリット

- 信頼できるソースからのイメージ提供
- 脆弱性スキャンの結果が公開
- セキュリティアップデートのタイムリーな適用
- ベストプラクティスに準拠したイメージ構成

## Distroless

Googleによる[Distroless](https://github.com/GoogleContainerTools/distroless)イメージは、アプリケーションとそのランタイム依存関係のみを含み、パッケージマネージャーやシェルなどの不要なコンポーネントを排除したイメージです。

### 主な特徴

- 最小限のコンポーネントのみを含む
- シェル、パッケージマネージャー、一般的なLinuxツールを含まない
- イメージサイズの大幅な削減
- 攻撃対象領域の縮小
- セキュリティポスチャーの向上

### 提供されているイメージ

[Distroless](https://github.com/GoogleContainerTools/distroless)では以下のような様々なカテゴリのイメージが提供されています：

- 言語ベースのイメージ: [Java](https://github.com/GoogleContainerTools/distroless/blob/main/java/README.md), [Python](https://github.com/GoogleContainerTools/distroless/blob/main/python3/README.md), [Node.js](https://github.com/GoogleContainerTools/distroless/blob/main/nodejs/README.md), [Go](https://github.com/GoogleContainerTools/distroless/blob/main/base/README.md) など
- ベースイメージ: [static](https://github.com/GoogleContainerTools/distroless/blob/main/base/README.md), [cc](https://github.com/GoogleContainerTools/distroless/blob/main/cc/README.md) など
- デバッグ用イメージ: 通常のイメージ + シェルとデバッグツール

### 使用上の注意点

- シェルが含まれていないため、従来のデバッグ手法が使用できない
- 最小限のツールセットのため、トラブルシューティングに制約がある
- アプリケーションの依存関係を明示的に管理する必要がある

# 導入手順

## Chainguard Images の利用

### 基本的な使用方法

1. イメージをプルする:

```bash
docker pull cgr.dev/chainguard/<イメージ名>:latest
```

2. Dockerfileで使用する:

```dockerfile
FROM cgr.dev/chainguard/go:latest AS builder
WORKDIR /app
COPY . .
RUN go build -o myapp .

FROM cgr.dev/chainguard/static:latest
COPY --from=builder /app/myapp /app/myapp
CMD ["/app/myapp"]
```

### 署名の検証

イメージの署名を検証するには:

```bash
cosign verify \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity https://github.com/chainguard-images/images/.github/workflows/release.yaml@refs/heads/main \
  cgr.dev/chainguard/<イメージ名>:latest
```

### イメージビルドの再現

イメージのビルドを再現するには:

```bash
IMAGE_NAME=cgr.dev/chainguard/<イメージ名>
cosign verify-attestation \
  --type https://apko.dev/image-configuration \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity https://github.com/chainguard-images/images/.github/workflows/release.yaml@refs/heads/main \
  "${IMAGE_NAME}" | jq -r .payload | base64 -d | jq .predicate > latest.apko.json
apko build latest.apko.json <イメージ名>:local <イメージ名>.tar
```

## Docker Trusted Content の利用

1. Docker Hubから公式イメージを検索:

```bash
docker search --filter is-official=true <イメージ名>
```

2. イメージをプルする:

```bash
docker pull <イメージ名>:tag
```

3. Dockerfileで使用する:

```dockerfile
FROM python:3.9-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY . .
CMD ["python", "app.py"]
```

## Distroless の利用

1. イメージをプルする:

```bash
docker pull gcr.io/distroless/static:latest
```

2. マルチステージビルドで使用する:

```dockerfile
FROM golang:1.19 AS builder
WORKDIR /app
COPY . .
RUN CGO_ENABLED=0 go build -o app .

FROM gcr.io/distroless/static:latest
COPY --from=builder /app/app /
CMD ["/app"]
```

詳細は[Distrolessの公式ドキュメント](https://github.com/GoogleContainerTools/distroless/blob/main/README.md)を参照してください。

# 運用手順

## 利用方法

### CI/CDパイプラインへの統合

以下のステップをCI/CDパイプラインに組み込むことを推奨します。

1. ベースイメージの選択:

```yaml
# GitLab CI/CD の例
build:
  image: cgr.dev/chainguard/go:latest
  script:
    - go build -o myapp .
```

2. イメージビルド:

```yaml
# GitHub Actions の例
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build image
        run: |
          docker build -t myapp:${{ github.sha }} .
```

3. イメージスキャン:

```yaml
scan:
  script:
    - trivy image myapp:${{ github.sha }}
```

4. デプロイ:

```yaml
deploy:
  script:
    - kubectl set image deployment/myapp myapp=myapp:${{ github.sha }}
```

### セキュアなイメージの検証

コンテナイメージを使用する前に、以下の検証を行うことを推奨します。

1. イメージの署名検証:

```bash
cosign verify <イメージ名>@<ダイジェスト>
```

2. SBOMの確認:

```bash
syft <イメージ名>:<タグ>
```

3. 脆弱性スキャン:

```bash
trivy image <イメージ名>:<タグ>
```

### Kubernetes環境での使用

Kubernetes環境では以下の設定を推奨します。

1. イメージプルポリシー:

```yaml
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: app
    image: cgr.dev/chainguard/nginx:latest
    imagePullPolicy: Always
```

2. ダイジェストを使用したデプロイ:

```yaml
apiVersion: apps/v1
kind: Deployment
spec:
  template:
    spec:
      containers:
      - name: app
        image: cgr.dev/chainguard/nginx@sha256:a1b2c3d4...
```

## 維持管理手順

### 定期的な更新

1. 最新のセキュリティパッチを含むイメージへの更新:

```bash
docker pull <イメージ名>:latest
```

2. バージョンを固定する場合は、定期的に更新:

```dockerfile
# 古いバージョン
FROM cgr.dev/chainguard/go:1.19
# 新しいバージョン
FROM cgr.dev/chainguard/go:1.20
```

### 監査とモニタリング

1. デプロイされたイメージの監査:

```bash
# イメージのダイジェストを取得
docker inspect --format='{{index .RepoDigests 0}}' <コンテナID>
```

2. イメージスキャンの定期実行:

```bash
# 週次でのスキャン実行例
0 0 * * 0 trivy image --format json --output trivy-results.json <イメージ名>:<タグ>
```

3. SBOMの保存と管理:

```bash
# SBOMを生成して保存
syft <イメージ名>:<タグ> -o spdx-json > sbom.json
```

### バージョン管理とポリシー適用

1. イメージタグではなくダイジェストを使用:

```yaml
# Kubernetes Deploymentの例
spec:
  containers:
  - name: app
    image: cgr.dev/chainguard/go@sha256:a1b2c3d4e5f6...
```

2. イメージポリシーの設定:

```yaml
# Kubernetes ImagePolicyWebhook の例
apiVersion: admissionregistration.k8s.io/v1
kind: ValidatingWebhookConfiguration
webhooks:
- name: imagepolicy.k8s.io
  rules:
  - apiGroups: [""]
    apiVersions: ["v1"]
    operations: ["CREATE", "UPDATE"]
    resources: ["pods"]
    scope: "Namespaced"
```

3. [OPA/Gatekeeper](https://github.com/open-policy-agent/gatekeeper) によるポリシー適用:

```yaml
apiVersion: constraints.gatekeeper.sh/v1beta1
kind: K8sTrustedImages
metadata:
  name: trusted-images
spec:
  match:
    kinds:
    - apiGroups: [""]
      kinds: ["Pod"]
  parameters:
    repositories:
    - "cgr.dev/chainguard/*"
    - "gcr.io/distroless/*"
```

これらの手順を実施することで、信頼性の高いコンテナイメージを安全かつ効率的に運用することができます。
