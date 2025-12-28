# Stage 1: Chef - Base image with cargo-chef installed
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

# Stage 2: Planner - Generate recipe.json
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Builder - Build dependencies and application
FROM chef AS builder

# Copy the recipe from planner stage
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this layer will be cached as long as dependencies don't change
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source code and build application
COPY . .
RUN cargo build --release --bin sample_api

# Stage 4: Runtime - Minimal production image
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from builder
COPY --from=builder /app/target/release/sample_api /usr/local/bin/sample_api

# Set environment variables
ENV RUST_LOG=info

# Expose port
EXPOSE 8080

# Run the application
ENTRYPOINT ["/usr/local/bin/sample_api"]
