#!/bin/bash

# デフォルト値
CONFIG_FILE=""
RELEASE_MODE=false

# 引数の解析
while [[ $# -gt 0 ]]; do
  case $1 in
    --config)
      CONFIG_FILE="$2"
      shift 2
      ;;
    --release)
      RELEASE_MODE=true
      shift
      ;;
    *)
      echo "Unknown option: $1"
      echo "Usage: $0 --config <config_file> [--release]"
      exit 1
      ;;
  esac
done

# 設定ファイルが指定されていない場合はエラー
if [ -z "$CONFIG_FILE" ]; then
  echo "Error: Config file not specified"
  echo "Usage: $0 --config <config_file> [--release]"
  exit 1
fi

# 設定ファイルの存在確認
if [ ! -f "$CONFIG_FILE" ]; then
  echo "Error: Config file '$CONFIG_FILE' not found"
  exit 1
fi

# ファイル拡張子の取得
FILE_EXT="${CONFIG_FILE##*.}"

# 一時ファイルの作成
TEMP_CONFIG=$(mktemp)
cp "$CONFIG_FILE" "$TEMP_CONFIG"

# リリースモードの設定
if [ "$RELEASE_MODE" = true ]; then
  echo "Setting release mode to true in config file..."
  
  if [ "$FILE_EXT" = "toml" ]; then
    # TOMLファイルの場合
    sed -i 's/release = false/release = true/g' "$TEMP_CONFIG"
  elif [ "$FILE_EXT" = "yaml" ] || [ "$FILE_EXT" = "yml" ]; then
    # YAMLファイルの場合
    sed -i 's/release: false/release: true/g' "$TEMP_CONFIG"
  else
    echo "Unsupported file format: $FILE_EXT"
    rm "$TEMP_CONFIG"
    exit 1
  fi
fi

# サンプルプログラムをビルド
echo "Building example programs..."
BUILD_CMD="cargo build"
if [ "$RELEASE_MODE" = true ]; then
  BUILD_CMD="$BUILD_CMD --release"
fi

(cd examples/monotonic_check && $BUILD_CMD)
(cd examples/even_counter && $BUILD_CMD)
(cd examples/max_finder && $BUILD_CMD)

# テストを実行
echo "Running tests..."
if [ "$RELEASE_MODE" = true ]; then
  cargo run --release -- --config "$TEMP_CONFIG"
else
  cargo run -- --config "$TEMP_CONFIG"
fi

# 一時ファイルの削除
rm "$TEMP_CONFIG" 