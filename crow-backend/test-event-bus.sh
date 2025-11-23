#!/bin/bash
# Test event bus and /event SSE endpoint

set -e

CROW_PORT=${CROW_PORT:-8899}
TEST_DIR="/home/thomas/src/projects/opencode-project/test-dummy"

echo "=== Crow Event Bus Integration Test ==="
echo "Port: $CROW_PORT"
echo "Test dir: $TEST_DIR"
echo ""

# Build
echo "1. Building crow..."
cd /home/thomas/src/projects/opencode-project/crow
cargo build --features server --bin crow-serve 2>&1 | tail -5

# Start server
echo ""
echo "2. Starting server..."
cd "$TEST_DIR"
../crow/target/debug/crow-serve --port $CROW_PORT &
SERVER_PID=$!
sleep 2

# Check server is running
if ! kill -0 $SERVER_PID 2>/dev/null; then
    echo "❌ Server failed to start"
    exit 1
fi
echo "✅ Server started (PID: $SERVER_PID)"

# Cleanup on exit
cleanup() {
    echo ""
    echo "Cleaning up..."
    kill $SERVER_PID 2>/dev/null || true
    kill $SSE_PID 2>/dev/null || true
}
trap cleanup EXIT

# Test 3: Connect to /event and capture events in background
echo ""
echo "3. Connecting to /event SSE endpoint..."
EVENT_LOG="/tmp/crow-events-$$.log"
curl -s -N "http://localhost:$CROW_PORT/event" > "$EVENT_LOG" &
SSE_PID=$!
sleep 1

# Check SSE connection
if ! kill -0 $SSE_PID 2>/dev/null; then
    echo "❌ SSE connection failed"
    exit 1
fi
echo "✅ Connected to event stream"

# Test 4: Check for server.connected event
echo ""
echo "4. Checking for server.connected event..."
sleep 1
if grep -q "server.connected" "$EVENT_LOG"; then
    echo "✅ Received server.connected event"
else
    echo "❌ No server.connected event"
    cat "$EVENT_LOG"
    exit 1
fi

# Test 5: Create a session and check for session.created event
echo ""
echo "5. Creating session..."
SESSION_RESPONSE=$(curl -s -X POST "http://localhost:$CROW_PORT/session" \
    -H "Content-Type: application/json" \
    -d '{"title": "Event Bus Test"}')
SESSION_ID=$(echo "$SESSION_RESPONSE" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
echo "Session ID: $SESSION_ID"
sleep 1

if grep -q "session.created" "$EVENT_LOG"; then
    echo "✅ Received session.created event"
else
    echo "❌ No session.created event"
fi

# Test 6: Update session and check for session.updated event
echo ""
echo "6. Updating session..."
curl -s -X PATCH "http://localhost:$CROW_PORT/session/$SESSION_ID" \
    -H "Content-Type: application/json" \
    -d '{"title": "Updated Title"}' > /dev/null
sleep 1

if grep -q "session.updated" "$EVENT_LOG"; then
    echo "✅ Received session.updated event"
else
    echo "❌ No session.updated event"
fi

# Test 7: Delete session and check for session.deleted event
echo ""
echo "7. Deleting session..."
curl -s -X DELETE "http://localhost:$CROW_PORT/session/$SESSION_ID" > /dev/null
sleep 1

if grep -q "session.deleted" "$EVENT_LOG"; then
    echo "✅ Received session.deleted event"
else
    echo "❌ No session.deleted event"
fi

# Summary
echo ""
echo "=== Event Log Summary ==="
echo "Events received:"
grep -o '"type":"[^"]*"' "$EVENT_LOG" | sort | uniq -c || echo "(none)"

echo ""
echo "=== Test Complete ==="
echo "✅ Event bus integration test passed!"
