#!/bin/bash

check_server() {
    local server=$1
    local start_time=$(date +%s)

    if ping -c 1 -W 2 "$server" > /dev/null 2>&1; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        echo "$server: OK (${duration}秒)"
    else
        echo "$server: 到達不可"
    fi
}

# 複数のサーバを同時にチェック
servers=("google.com" "github.com" "stackoverflow.com" "rust-lang.org")

for server in "${servers[@]}"; do
    check_server "$server" &
done

# すべての完了を待つ
wait

echo "すべてのチェックが完了しました"
