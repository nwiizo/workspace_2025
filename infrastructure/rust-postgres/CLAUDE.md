# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust technical verification project for the "Just Use Postgres!" book. It demonstrates PostgreSQL advanced features (full-text search, JSONB, TimescaleDB, PostGIS, pgvector, message queues) using Rust with `tokio-postgres` and `deadpool-postgres`.

## Build and Run Commands

```bash
# Start PostgreSQL (TimescaleDB + PostGIS + pgvector)
docker compose up -d

# Build
cargo build --release

# Run all demos
cargo run --release -- all

# Run individual demos
cargo run --release -- generate-series      # Chapter 1
cargo run --release -- show-schemas         # Chapter 2
cargo run --release -- transaction          # Chapter 2
cargo run --release -- cart-demo            # Chapter 2
cargo run --release -- view-demo            # Chapter 2
cargo run --release -- cte-demo             # Chapter 3
cargo run --release -- window-functions     # Chapter 3
cargo run --release -- recursive-query      # Chapter 3
cargo run --release -- index-demo           # Chapter 4
cargo run --release -- jsonb-demo           # Chapter 5
cargo run --release -- json-advanced        # Chapter 5
cargo run --release -- full-text-search "search query"  # Chapter 6
cargo run --release -- full-text-advanced   # Chapter 6
cargo run --release -- extensions           # Chapter 7
cargo run --release -- pgvector             # Chapter 8
cargo run --release -- timescaledb          # Chapter 9
cargo run --release -- postgis              # Chapter 10
cargo run --release -- message-queue        # Chapter 11

# Format and lint
cargo fmt
cargo clippy -- -D warnings
```

## Architecture

```
src/
├── main.rs           # CLI entry point using clap, creates deadpool connection pool
└── examples/         # Each module demonstrates a PostgreSQL feature
    ├── mod.rs        # Module exports
    ├── generate_series.rs   # Mock data generation
    ├── schemas.rs           # Multi-schema design
    ├── transaction.rs       # Transaction handling
    ├── cart.rs              # PL/pgSQL functions
    ├── view.rs              # Database views
    ├── cte.rs               # Common Table Expressions
    ├── window_functions.rs  # Window functions
    ├── recursive.rs         # Recursive CTEs
    ├── indexes.rs           # Index strategies
    ├── jsonb.rs             # JSONB operations
    ├── json_advanced.rs     # JSON path queries
    ├── full_text_search.rs  # tsvector/tsquery
    ├── full_text_advanced.rs # Weighted search, highlighting
    ├── extensions.rs        # pgcrypto, pg_trgm, citext
    ├── pgvector.rs          # Vector similarity search
    ├── timescaledb.rs       # Hypertables, time_bucket
    ├── postgis.rs           # Geometry/geography types
    └── message_queue.rs     # FOR UPDATE SKIP LOCKED
```

## Database Setup

- Docker image: `timescale/timescaledb-ha:pg17` (includes TimescaleDB, PostGIS, pgvector)
- Connection: `localhost:5432`, user `postgres`, password `password`, database `coffee_chain`
- Init scripts in `init/` are auto-executed on first container start

## Database Schema Structure

```
coffee_chain (database)
├── products (schema)    # Product catalog and reviews
├── customers (schema)   # Customer accounts
├── sales (schema)       # Orders, order_items, views
├── streaming (schema)   # Songs and plays (for recursive queries)
├── watch (schema)       # TimescaleDB hypertables
├── florida (schema)     # PostGIS data (cities, POIs)
└── mq (schema)          # Message queue tables
```

## Rust + PostgreSQL Patterns

### Type Mapping Issues

```rust
// pg_typeof() requires ::TEXT cast
let row = client.query_one("SELECT pg_typeof(data)::TEXT AS type FROM table", &[]).await?;

// ENUM types should be retrieved as String
let status: String = row.get("status");

// tsvector cannot be directly deserialized - cast to TEXT or use ts_rank()
```

### TEMP TABLE Naming

When creating temp tables, use unique names to avoid conflicts across demos:
```rust
client.execute("DROP TABLE IF EXISTS pg_temp.movies_vector", &[]).await?;
client.execute("CREATE TEMP TABLE movies_vector ...", &[]).await?;
```

### PL/pgSQL Variable Naming

Use prefixes to avoid collision with column names:
```sql
CREATE FUNCTION test(p_order_id UUID) ...  -- Use p_ prefix for parameters
DECLARE v_price NUMERIC;                    -- Use v_ prefix for variables
```
