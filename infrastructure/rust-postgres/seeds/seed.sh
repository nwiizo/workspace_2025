#!/usr/bin/env bash
#
# seed.sh - データベースシードスクリプト
#
# 使用方法:
#   ./seeds/seed.sh              # 全データ投入（デフォルト）
#   ./seeds/seed.sh --schema     # スキーマのみ作成
#   ./seeds/seed.sh --products   # 商品データのみ
#   ./seeds/seed.sh --stores     # 店舗データのみ
#   ./seeds/seed.sh --metrics    # メトリクスデータのみ
#   ./seeds/seed.sh --reset      # データベースをリセットして全投入
#
# 環境変数:
#   PGHOST     - PostgreSQL ホスト (default: localhost)
#   PGPORT     - PostgreSQL ポート (default: 5432)
#   PGUSER     - PostgreSQL ユーザー (default: postgres)
#   PGPASSWORD - PostgreSQL パスワード (default: password)
#   PGDATABASE - PostgreSQL データベース名 (default: coffee_chain)

set -euo pipefail

# =============================================================================
# 設定
# =============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# データベース接続設定
export PGHOST="${PGHOST:-localhost}"
export PGPORT="${PGPORT:-5432}"
export PGUSER="${PGUSER:-postgres}"
export PGPASSWORD="${PGPASSWORD:-password}"
export PGDATABASE="${PGDATABASE:-coffee_chain}"

# 色付き出力
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# =============================================================================
# ユーティリティ関数
# =============================================================================

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# 実行時間を計測して表示
time_it() {
    local name="$1"
    shift
    local start_time=$(date +%s.%N)

    "$@"

    local end_time=$(date +%s.%N)
    local elapsed=$(echo "$end_time - $start_time" | bc)
    success "$name completed in ${elapsed}s"
}

# SQLファイルを実行
run_sql() {
    local file="$1"
    local name="$2"

    if [[ ! -f "$file" ]]; then
        error "SQL file not found: $file"
    fi

    info "Running $name..."
    psql -v ON_ERROR_STOP=1 -f "$file" 2>&1 | grep -E "^(NOTICE|INSERT|CREATE|DROP|SELECT|Total)" || true
}

# データベース接続確認
check_connection() {
    info "Checking database connection..."
    if ! psql -c "SELECT 1" > /dev/null 2>&1; then
        error "Cannot connect to PostgreSQL. Please check your connection settings:
  PGHOST=$PGHOST
  PGPORT=$PGPORT
  PGUSER=$PGUSER
  PGDATABASE=$PGDATABASE"
    fi
    success "Database connection OK"
}

# データベース作成（存在しない場合）
ensure_database() {
    info "Ensuring database exists..."
    psql -d postgres -c "CREATE DATABASE $PGDATABASE" 2>/dev/null || true
}

# =============================================================================
# シード実行関数
# =============================================================================

run_schema() {
    time_it "Schema creation" run_sql "$SCRIPT_DIR/01_schema.sql" "schema"
}

run_products() {
    time_it "Products seed" run_sql "$SCRIPT_DIR/02_products.sql" "products"
}

run_stores() {
    time_it "Stores seed" run_sql "$SCRIPT_DIR/03_stores.sql" "stores"
}

run_metrics() {
    info "Running metrics seed (this may take a few minutes)..."
    time_it "Metrics seed" run_sql "$SCRIPT_DIR/04_metrics.sql" "metrics"
}

run_all() {
    echo ""
    echo "========================================"
    echo "  Coffee Chain Database Seeder"
    echo "========================================"
    echo ""
    echo "Database: $PGDATABASE@$PGHOST:$PGPORT"
    echo ""

    check_connection

    local total_start=$(date +%s.%N)

    run_schema
    run_products
    run_stores
    run_metrics

    local total_end=$(date +%s.%N)
    local total_elapsed=$(echo "$total_end - $total_start" | bc)

    echo ""
    echo "========================================"
    success "All seeds completed in ${total_elapsed}s"
    echo "========================================"

    # 統計情報表示
    echo ""
    info "Database Statistics:"
    psql -c "
    SELECT 'Products' as table_name, COUNT(*) as row_count FROM products.catalog
    UNION ALL
    SELECT 'Stores', COUNT(*) FROM stores.locations
    UNION ALL
    SELECT 'Sensors', COUNT(*) FROM metrics.sensors
    UNION ALL
    SELECT 'Measurements', COUNT(*) FROM metrics.measurements
    UNION ALL
    SELECT 'User Activities', COUNT(*) FROM metrics.user_activities
    ORDER BY row_count DESC;
    "
}

reset_database() {
    warn "This will DROP all data in the database. Are you sure? (y/N)"
    read -r response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        info "Resetting database..."
        psql -c "
        DROP SCHEMA IF EXISTS products CASCADE;
        DROP SCHEMA IF EXISTS stores CASCADE;
        DROP SCHEMA IF EXISTS metrics CASCADE;
        "
        success "Database reset complete"
        run_all
    else
        info "Reset cancelled"
    fi
}

# =============================================================================
# ヘルプ表示
# =============================================================================

show_help() {
    cat << EOF
Coffee Chain Database Seeder

Usage: $0 [OPTION]

Options:
  (no option)    Run all seeds (schema + products + stores + metrics)
  --schema       Create schema only
  --products     Seed products data only
  --stores       Seed stores data only
  --metrics      Seed metrics data only (large dataset, takes time)
  --reset        Reset database and run all seeds
  --help, -h     Show this help message

Environment Variables:
  PGHOST         PostgreSQL host (default: localhost)
  PGPORT         PostgreSQL port (default: 5432)
  PGUSER         PostgreSQL user (default: postgres)
  PGPASSWORD     PostgreSQL password (default: password)
  PGDATABASE     PostgreSQL database (default: coffee_chain)

Examples:
  # Run all seeds with default settings
  $0

  # Run with custom database settings
  PGHOST=db.example.com PGPASSWORD=secret $0

  # Create schema only
  $0 --schema

  # Reset and reseed
  $0 --reset

EOF
}

# =============================================================================
# メイン
# =============================================================================

main() {
    case "${1:-}" in
        --help|-h)
            show_help
            ;;
        --schema)
            check_connection
            run_schema
            ;;
        --products)
            check_connection
            run_products
            ;;
        --stores)
            check_connection
            run_stores
            ;;
        --metrics)
            check_connection
            run_metrics
            ;;
        --reset)
            reset_database
            ;;
        "")
            run_all
            ;;
        *)
            error "Unknown option: $1\nUse --help for usage information."
            ;;
    esac
}

main "$@"
