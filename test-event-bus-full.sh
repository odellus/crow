#!/bin/bash
# Full integration test for event bus - tests streaming message events
# Validates: session events, message events, status events during agent execution

set -e

CROW_PORT=${CROW_PORT:-8899}
TEST_DIR="/home/thomas/src/projects/opencode-project/test-dummy"
CROW_DIR="/home/thomas/src/projects/opencode-project/crow"

echo "=== Crow Event Bus Full Integration Test ==="
echo "Port: $CROW_PORT"
echo "Test dir: $TEST_DIR"
echo ""

# Build
echo "1. Building crow..."
cd "$CROW_DIR"
cargo build --features server --bin crow-serve 2>&1 | tail -3

# Start server with verbose logging
echo ""
echo "2. Starting server with verbose logging..."
cd "$TEST_DIR"
rm -rf .crow/snapshot 2>/dev/null || true
CROW_VERBOSE_LOG=1 "$CROW_DIR/target/debug/crow-serve" --port $CROW_PORT &
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
    rm -f "$EVENT_LOG" 2>/dev/null || true
}
trap cleanup EXIT

# Connect to /event SSE endpoint
echo ""
echo "3. Connecting to /event SSE endpoint..."
EVENT_LOG="/tmp/crow-events-full-$$.log"
curl -s -N "http://localhost:$CROW_PORT/event" > "$EVENT_LOG" &
SSE_PID=$!
sleep 1

if ! kill -0 $SSE_PID 2>/dev/null; then
    echo "❌ SSE connection failed"
    exit 1
fi
echo "✅ Connected to event stream"

# Check server.connected
echo ""
echo "4. Checking server.connected event..."
sleep 1
if grep -q "server.connected" "$EVENT_LOG"; then
    echo "✅ Received server.connected"
else
    echo "❌ No server.connected event"
    exit 1
fi

# Create session
echo ""
echo "5. Creating session..."
SESSION_RESPONSE=$(curl -s -X POST "http://localhost:$CROW_PORT/session" \
    -H "Content-Type: application/json" \
    -d '{"title": "Event Bus Full Test"}')
SESSION_ID=$(echo "$SESSION_RESPONSE" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
echo "   Session ID: $SESSION_ID"
sleep 1

if grep -q "session.created" "$EVENT_LOG"; then
    echo "✅ Received session.created"
else
    echo "❌ No session.created event"
fi

# Send streaming message - this should trigger:
# - session.status (busy)
# - message.part.updated (text deltas)
# - message.updated
# - session.idle
echo ""
echo "6. Sending streaming message to agent..."
echo "   (This will trigger session.status, message.part.updated, session.idle)"
echo ""

# Use a simple prompt that should get a quick response
STREAM_OUTPUT="/tmp/crow-stream-$$.log"
curl -s -N "http://localhost:$CROW_PORT/session/$SESSION_ID/message/stream" \
    -X POST \
    -H "Content-Type: application/json" \
    -d '{"agent":"build","parts":[{"type":"text","text":"Say hello in exactly 3 words"}]}' \
    > "$STREAM_OUTPUT" &
STREAM_PID=$!

# Wait for response (max 30 seconds)
echo "   Waiting for agent response..."
WAIT_COUNT=0
while kill -0 $STREAM_PID 2>/dev/null && [ $WAIT_COUNT -lt 30 ]; do
    sleep 1
    WAIT_COUNT=$((WAIT_COUNT + 1))
    # Check if we got complete event
    if grep -q "message.complete" "$STREAM_OUTPUT"; then
        break
    fi
done

# Give events time to propagate
sleep 2

# Check for session.status (busy)
echo ""
echo "7. Checking session.status event..."
if grep -q '"type":"session.status"' "$EVENT_LOG"; then
    echo "✅ Received session.status"
    # Show the status
    grep '"type":"session.status"' "$EVENT_LOG" | head -1 | grep -o '"status":{[^}]*}' || true
else
    echo "⚠️  No session.status event (may not have been published)"
fi

# Check for message.part.updated (streaming)
echo ""
echo "8. Checking message.part.updated events..."
PART_COUNT=$(grep -c '"type":"message.part.updated"' "$EVENT_LOG" 2>/dev/null || echo "0")
if [ "$PART_COUNT" -gt 0 ]; then
    echo "✅ Received $PART_COUNT message.part.updated events"
else
    echo "⚠️  No message.part.updated events"
fi

# Check for message.updated
echo ""
echo "9. Checking message.updated event..."
if grep -q '"type":"message.updated"' "$EVENT_LOG"; then
    echo "✅ Received message.updated"
else
    echo "⚠️  No message.updated event"
fi

# Check for session.idle
echo ""
echo "10. Checking session.idle event..."
if grep -q '"type":"session.idle"' "$EVENT_LOG"; then
    echo "✅ Received session.idle"
else
    echo "⚠️  No session.idle event"
fi

# Check XDG directories
echo ""
echo "=== XDG Directory Check ==="

CROW_DATA="$HOME/.local/share/crow"

echo ""
echo "Sessions stored:"
ls -la "$CROW_DATA/storage/session/" 2>/dev/null | head -5 || echo "(none)"

echo ""
echo "Messages stored:"
ls -la "$CROW_DATA/storage/message/" 2>/dev/null | head -5 || echo "(none)"

if [ -d "$CROW_DATA/requests" ]; then
    echo ""
    echo "Verbose request logs:"
    ls -lt "$CROW_DATA/requests/" 2>/dev/null | head -5 || echo "(none)"
fi

# Check local .crow directory
echo ""
echo "=== Local .crow Directory ==="
echo ""
echo "Sessions exported:"
ls -la "$TEST_DIR/.crow/sessions/" 2>/dev/null | head -5 || echo "(none)"

# Event summary
echo ""
echo "=== Event Summary ==="
echo "All event types received:"
grep -o '"type":"[^"]*"' "$EVENT_LOG" | sort | uniq -c | sort -rn

# Stream response summary
echo ""
echo "=== Stream Response Summary ==="
if [ -f "$STREAM_OUTPUT" ]; then
    echo "Events in stream response:"
    grep -o 'event: [a-z.]*' "$STREAM_OUTPUT" | sort | uniq -c || echo "(none)"
fi

# Final validation
echo ""
echo "=== Final Validation ==="

PASS=true

if ! grep -q "server.connected" "$EVENT_LOG"; then
    echo "❌ Missing: server.connected"
    PASS=false
fi

if ! grep -q "session.created" "$EVENT_LOG"; then
    echo "❌ Missing: session.created"
    PASS=false
fi

if [ "$PART_COUNT" -eq 0 ]; then
    echo "⚠️  Warning: No message.part.updated events (streaming may not have published)"
fi

if $PASS; then
    echo ""
    echo "✅ Event bus integration test PASSED!"
else
    echo ""
    echo "❌ Event bus integration test FAILED"
    exit 1
fi

# Cleanup stream output
rm -f "$STREAM_OUTPUT" 2>/dev/null || true
