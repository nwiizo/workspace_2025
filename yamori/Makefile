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
	./run_tests_debug.sh "$(CONFIG)"

# テスト実行（リリースビルド）
test-release:
	@if [ -z "$(CONFIG)" ]; then \
		echo "Error: CONFIG variable not set"; \
		echo "Usage: make test-release CONFIG=<config_file>"; \
		exit 1; \
	fi
	./run_tests_release.sh "$(CONFIG)"

# テスト実行（デフォルトはデバッグビルド）
test:
	@if [ -z "$(CONFIG)" ]; then \
		echo "Error: CONFIG variable not set"; \
		echo "Usage: make test CONFIG=<config_file>"; \
		exit 1; \
	fi
	./run_tests_debug.sh "$(CONFIG)"

# クリーン
clean:
	cargo clean
	(cd examples/monotonic_check && cargo clean)
	(cd examples/even_counter && cargo clean)
	(cd examples/max_finder && cargo clean)

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