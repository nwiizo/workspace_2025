# Build stage
FROM rust:1.75-slim as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    nodejs \
    npm \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install wasm-pack and trunk
RUN cargo install trunk && \
    rustup target add wasm32-unknown-unknown

WORKDIR /app

# Copy project files
COPY . .

# Build the application
RUN trunk build --release

# Production stage
FROM nginx:alpine

# Copy the built assets from builder
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy nginx configuration if needed
# COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
