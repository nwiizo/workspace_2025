.PHONY: build test test-debug test-release clean

# デフォルトターゲット
all: build

# ビルド
build:
	cargo build

# リリースビルド
build-release:
	cargo build --release

# テスト実行（デバッグビルド）
test-debug:
	@if [ -z "$(CONFIG)" ]; then \
		echo "Error: CONFIG variable not set"; \
		echo "Usage: make test-debug CONFIG=<config_file>"; \
		exit 1; \
	fi
	./scripts/run_tests_debug.sh "$(CONFIG)"

# テスト実行（リリースビルド）
test-release:
	@if [ -z "$(CONFIG)" ]; then \
		echo "Error: CONFIG variable not set"; \
		echo "Usage: make test-release CONFIG=<config_file>"; \
		exit 1; \
	fi
	./scripts/run_tests_release.sh "$(CONFIG)"

# テスト実行（デフォルトはデバッグビルド）
test:
	@if [ -z "$(CONFIG)" ]; then \
		echo "Error: CONFIG variable not set"; \
		echo "Usage: make test CONFIG=<config_file>"; \
		exit 1; \
	fi
	./scripts/run_tests_debug.sh "$(CONFIG)"

# 特定のテストを実行
run-test:
	@if [ -z "$(TEST)" ]; then \
		echo "Error: TEST variable not set"; \
		echo "Usage: make run-test TEST=<test_name> [CONFIG=<config_file>]"; \
		exit 1; \
	fi
	@if [ -z "$(CONFIG)" ]; then \
		CONFIG="tests/configs/tests.toml"; \
	fi
	./scripts/run_tests.sh "$(CONFIG)" "$(TEST)"

# CLIモードでテストを実行
test-cli:
	@if [ -z "$(CONFIG)" ]; then \
		echo "Error: CONFIG variable not set"; \
		echo "Usage: make test-cli CONFIG=<config_file>"; \
		exit 1; \
	fi
	cargo run -- --cli --yamori-config "$(CONFIG)"

# TUIモードでテストを実行
test-tui:
	@if [ -z "$(CONFIG)" ]; then \
		echo "Error: CONFIG variable not set"; \
		echo "Usage: make test-tui CONFIG=<config_file>"; \
		exit 1; \
	fi
	cargo run -- --yamori-config "$(CONFIG)"

# クリーン
clean:
	cargo clean

# ヘルプ
help:
	@echo "Available targets:"
	@echo "  build         - Build the project (debug mode)"
	@echo "  build-release - Build the project (release mode)"
	@echo "  test          - Run tests (debug mode)"
	@echo "  test-debug    - Run tests (debug mode)"
	@echo "  test-release  - Run tests (release mode)"
	@echo "  test-cli      - Run tests in CLI mode"
	@echo "  test-tui      - Run tests in TUI mode"
	@echo "  run-test      - Run a specific test"
	@echo "  clean         - Clean build artifacts"
	@echo "  help          - Show this help"
	@echo ""
	@echo "Examples:"
	@echo "  make test CONFIG=tests/configs/tests.toml"
	@echo "  make test-cli CONFIG=tests/configs/tests.yaml"
	@echo "  make run-test TEST=\"Echo Test\" CONFIG=tests/configs/tests.toml"

# 全てのサンプルプログラムをビルド（デバッグ）
build-examples:
	(cd examples/monotonic_check && cargo build)
	(cd examples/even_counter && cargo build)
	(cd examples/max_finder && cargo build)

# 全てのサンプルプログラムをビルド（リリース）
build-examples-release:
	(cd examples/monotonic_check && cargo build --release)
	(cd examples/even_counter && cargo build --release)
	(cd examples/max_finder && cargo build --release)

# 例: TOML設定ファイルでテスト実行
test-toml:
	./run_tests.sh --config examples/examples_tests.toml

# 例: YAML設定ファイルでテスト実行
test-yaml:
	./run_tests.sh --config examples/examples_tests.yaml

# 例: TOML設定ファイルでリリースモードでテスト実行
test-toml-release:
	./run_tests.sh --config examples/examples_tests.toml --release

# 例: YAML設定ファイルでリリースモードでテスト実行
test-yaml-release:
	./run_tests.sh --config examples/examples_tests.yaml --release 