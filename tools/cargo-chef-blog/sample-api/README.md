# Sample API - cargo-chef Docker Build Demo

This is a demonstration project for `cargo-chef`, a tool that optimizes Rust Docker builds by caching dependencies separately from application code.

## Project Overview

A simple REST API built with:
- **actix-web** - Web framework
- **tokio** - Async runtime
- **serde** - Serialization
- **env_logger** - Logging

## API Endpoints

- `GET /health` - Health check endpoint
- `GET /users` - Get all users
- `POST /users` - Create a new user

## Docker Build Strategies

This project includes two Dockerfile configurations to demonstrate cargo-chef optimization:

### 1. Standard Build (Dockerfile)

Standard 3-stage build with cargo-chef:
- **Planner stage**: Generates recipe.json
- **Builder stage**: Caches dependencies, builds application
- **Runtime stage**: Minimal production image

```bash
# Build
docker build -t sample-api:standard .

# Run
docker run -p 8080:8080 sample-api:standard
```

### 2. Optimized Build (Dockerfile.optimized)

Advanced build with multiple caching layers:
- **cargo-chef**: Coarse-grained dependency caching
- **sccache**: Fine-grained compilation artifact caching
- **BuildKit cache mounts**: Registry and git cache

```bash
# Build with BuildKit
DOCKER_BUILDKIT=1 docker build -f Dockerfile.optimized -t sample-api:optimized .

# Run
docker run -p 8080:8080 sample-api:optimized
```

## Testing with Docker Compose

```bash
# Build and run standard version
docker compose up api-standard

# Build and run optimized version
docker compose up api-optimized

# Access endpoints
curl http://localhost:8080/health
curl http://localhost:8080/users
```

## Verifying Cache Behavior

### First Build (Cold Cache)
```bash
time DOCKER_BUILDKIT=1 docker build -t sample-api:test .
```

### Second Build (Warm Cache - No Changes)
```bash
time DOCKER_BUILDKIT=1 docker build -t sample-api:test .
# Should be extremely fast due to layer caching
```

### Third Build (Source Code Change Only)
```bash
# Modify src/main.rs
echo "// Test comment" >> src/main.rs

time DOCKER_BUILDKIT=1 docker build -t sample-api:test .
# Dependencies should be cached, only app compilation runs
```

### Fourth Build (Dependency Change)
```bash
# Add a dependency to Cargo.toml
echo 'chrono = "0.4"' >> Cargo.toml

time DOCKER_BUILDKIT=1 docker build -t sample-api:test .
# Full rebuild required
```

## Expected Performance

Based on blog.md findings:

- **Cold build**: 10-20 minutes (depends on machine)
- **Warm build (no changes)**: <1 second
- **Source-only changes**: 2-5 minutes (depends on code size)
- **Dependency changes**: 10-20 minutes (full rebuild)

## Key Takeaways

1. **cargo-chef separates dependency compilation from source compilation**
   - Dependencies are cached in a separate Docker layer
   - Source changes don't invalidate dependency cache

2. **BuildKit cache mounts prevent re-downloading**
   - Cargo registry cache persists between builds
   - Git dependencies cache persists

3. **sccache provides fine-grained caching**
   - Individual compilation artifacts are cached
   - Partial dependency changes only rebuild affected crates

## Project Structure

```
sample-api/
├── Dockerfile              # Standard cargo-chef build
├── Dockerfile.optimized    # Optimized with sccache
├── docker-compose.yml      # Easy testing setup
├── .dockerignore          # Minimize build context
├── Cargo.toml             # Project manifest
├── src/
│   └── main.rs           # API server implementation
└── README.md
```

## Clean Up

```bash
# Stop containers
docker compose down

# Remove images
docker rmi sample-api:standard sample-api:optimized

# Prune build cache (use with caution)
docker builder prune
```

## References

- [cargo-chef GitHub](https://github.com/LukeMathWalker/cargo-chef)
- [Blog post](../blog.md) - Detailed technical analysis
