# Rust builder stage
FROM rust:1.83-slim-bullseye as builder

# Create a new empty shell project
RUN USER=root cargo new --bin new_year
WORKDIR /new_year

# Copy manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Build only the dependencies to cache them
RUN cargo build --release
RUN rm src/*.rs

# Copy the source code
COPY ./src ./src

# Build for release
RUN rm ./target/release/deps/new_year*
RUN cargo build --release

# Final stage
FROM debian:bullseye-slim

# Runtime dependencies for the colored crate
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libssl1.1 \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /new_year/target/release/new_year /usr/local/bin/

# Set the entrypoint
ENTRYPOINT ["new_year"]
