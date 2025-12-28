#!/bin/bash

# バックグラウンドで実行
echo "タスク1を開始..."
sleep 3 &
pid1=$!

echo "タスク2を開始..."
sleep 2 &
pid2=$!

echo "タスク3を開始..."
sleep 4 &
pid3=$!

# すべてのタスクの完了を待つ
echo "すべてのタスクが完了するのを待っています..."
wait $pid1
echo "タスク1が完了しました"

wait $pid2
echo "タスク2が完了しました"

wait $pid3
echo "タスク3が完了しました"

echo "すべて完了！"
