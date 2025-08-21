#!/bin/bash
set -ex

echo "Creating test application namespace..."
kubectl create namespace test-app || true

echo "Applying test application manifests..."
kubectl apply -f test-app/deployment.yaml -n test-app
kubectl apply -f test-app/service.yaml -n test-app
kubectl apply -f test-app/ingress.yaml -n test-app

echo "Waiting for deployment to be ready..."
kubectl wait --for=condition=available deployment/test-app -n test-app --timeout=60s

echo "Waiting for Ingress controller to be ready..."
kubectl -n ingress-nginx wait --for=condition=ready pod \
  --selector=app.kubernetes.io/component=controller \
  --timeout=90s

# port-forwardを開始（バックグラウンドで実行）
echo "Starting port forward..."
kubectl port-forward -n ingress-nginx service/ingress-nginx-controller 8080:80 &
FORWARD_PID=$!

# プロセスが終了時にport-forwardを停止
trap "kill $FORWARD_PID" EXIT

# port-forwardが準備できるまで少し待機
sleep 5

echo "Service available at: http://localhost:8080"
echo "You can test the application using: curl -H 'Host: test-app.localdev.me' http://localhost:8080"

# 簡単な動作確認
curl -H "Host: test-app.localdev.me" http://localhost:8080
