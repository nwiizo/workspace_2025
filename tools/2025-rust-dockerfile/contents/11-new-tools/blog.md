## 2025年の新ツール活用

### docker init - プロジェクトの素早い立ち上げ

```bash
# プロジェクトディレクトリで実行
docker init

# Rustを選択すると自動生成：
# - Dockerfile
# - compose.yaml
# - .dockerignore
# - README.Docker.md
```

[https://docs.docker.com/reference/cli/docker/init/:embed:cite]

### Docker Bake - 複雑なビルドの管理

**docker-bake.hcl:**

```hcl
group "default" {
  targets = ["app"]
}

variable "TAG" {
  default = "latest"
}

target "app" {
  context = "."
  dockerfile = "Dockerfile"
  tags = ["myapp:${TAG}"]
  platforms = ["linux/amd64", "linux/arm64"]
  cache-from = ["type=registry,ref=myapp:cache"]
  cache-to = ["type=registry,ref=myapp:cache,mode=max"]
}
```

```bash
# 実行
docker buildx bake

# 変数をオーバーライド
docker buildx bake --set TAG=v1.0.0
```

[https://docs.docker.com/build/bake/:embed:cite]

