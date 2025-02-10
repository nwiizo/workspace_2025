#!/bin/bash
set -e

PORT=8080
HOST="test-app.localdev.me"
CURL_OPTS="-H Host:${HOST}"

# port-forwardを開始（バックグラウンドで実行）
echo "Starting port forward..."
kubectl port-forward -n ingress-nginx service/ingress-nginx-controller ${PORT}:80 &
FORWARD_PID=$!

# プロセスが終了時にport-forwardを停止
trap "kill $FORWARD_PID" EXIT

# port-forwardが準備できるまで少し待機
sleep 5

echo "Running security tests against localhost:${PORT}"

# 接続テスト
echo -e "\nTesting connectivity..."
if ! curl -s ${CURL_OPTS} "http://localhost:${PORT}/" > /dev/null; then
    echo "Error: Cannot connect to the application"
    echo "Please ensure:"
    echo "1. Kind cluster is running"
    echo "2. Ingress controller is deployed"
    echo "3. Port ${PORT} is available"
    kill $FORWARD_PID
    exit 1
fi

# 各種攻撃パターンのテスト
echo -e "\n1. Testing normal access..."
curl -s ${CURL_OPTS} "http://localhost:${PORT}/" | head -n 5

echo -e "\n2. Testing XSS attack..."
curl -s ${CURL_OPTS} -X POST -d "payload=<script>alert(1)</script>" "http://localhost:${PORT}/"

echo -e "\n3. Testing SQL injection..."
curl -s ${CURL_OPTS} "http://localhost:${PORT}/?id=1'+OR+'1'='1"

echo -e "\n4. Testing directory traversal..."
curl -s ${CURL_OPTS} "http://localhost:${PORT}/../../../etc/passwd"

echo -e "\n5. Testing command injection..."
curl -s ${CURL_OPTS} "http://localhost:${PORT}/?cmd=cat%20/etc/passwd"

# ログの確認
PODNAME=$(kubectl -n ingress-nginx get pod -l app.kubernetes.io/component=controller -o=jsonpath='{.items[0].metadata.name}')
echo -e "\nChecking ModSecurity logs..."
kubectl -n ingress-nginx logs $PODNAME | grep ModSecurity: | tail -n 10

echo -e "\nTests completed. Stopping port forward..."
