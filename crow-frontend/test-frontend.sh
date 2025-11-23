#!/bin/bash
# Test crow frontend with crow server

set -e

CROW_PORT=${CROW_PORT:-8898}
FRONTEND_PORT=${FRONTEND_PORT:-5173}
TEST_DIR="/home/thomas/src/projects/opencode-project/test-dummy"
CROW_DIR="/home/thomas/src/projects/opencode-project/crow"
FRONTEND_DIR="/home/thomas/src/projects/opencode-project/crow-frontend"

echo "=== Crow Frontend Test ==="
echo "Crow server port: $CROW_PORT"
echo "Frontend port: $FRONTEND_PORT"
echo ""

# Build crow
echo "1. Building crow server..."
cd "$CROW_DIR"
cargo build --features server --bin crow-serve 2>&1 | tail -3

# Start crow server
echo ""
echo "2. Starting crow server..."
cd "$TEST_DIR"
CROW_VERBOSE_LOG=1 "$CROW_DIR/target/debug/crow-serve" --port $CROW_PORT &
SERVER_PID=$!
sleep 2

if ! kill -0 $SERVER_PID 2>/dev/null; then
    echo "Failed to start crow server"
    exit 1
fi
echo "Crow server started (PID: $SERVER_PID)"

# Cleanup on exit
cleanup() {
    echo ""
    echo "Cleaning up..."
    kill $SERVER_PID 2>/dev/null || true
    kill $FRONTEND_PID 2>/dev/null || true
}
trap cleanup EXIT

# Start frontend dev server
echo ""
echo "3. Starting frontend dev server..."
cd "$FRONTEND_DIR"
npm run dev -- --port $FRONTEND_PORT &
FRONTEND_PID=$!
sleep 3

if ! kill -0 $FRONTEND_PID 2>/dev/null; then
    echo "Failed to start frontend"
    exit 1
fi

echo ""
echo "=== Servers Running ==="
echo ""
echo "Crow API:    http://localhost:$CROW_PORT"
echo "Frontend:    http://localhost:$FRONTEND_PORT"
echo ""
echo "Open http://localhost:$FRONTEND_PORT in your browser"
echo ""
echo "Press Ctrl+C to stop both servers"

# Wait for either to exit
wait
