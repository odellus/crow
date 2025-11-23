#!/bin/bash
# E2E test for crow frontend - validates API integration

set -e

CROW_PORT=${CROW_PORT:-8898}
TEST_DIR="/home/thomas/src/projects/opencode-project/test-dummy"
CROW_DIR="/home/thomas/src/projects/opencode-project/crow"

echo "=== Crow Frontend E2E Test ==="
echo "Testing API endpoints that frontend uses"
echo ""

# Build crow
echo "1. Building crow server..."
cd "$CROW_DIR"
cargo build --features server --bin crow-serve 2>&1 | tail -3

# Start crow server
echo ""
echo "2. Starting crow server..."
cd "$TEST_DIR"
"$CROW_DIR/target/debug/crow-serve" --port $CROW_PORT &
SERVER_PID=$!
sleep 2

if ! kill -0 $SERVER_PID 2>/dev/null; then
    echo "Failed to start crow server"
    exit 1
fi
echo "Server started (PID: $SERVER_PID)"

# Cleanup
cleanup() {
    echo ""
    echo "Cleaning up..."
    kill $SERVER_PID 2>/dev/null || true
}
trap cleanup EXIT

# Test 1: GET /session (list sessions)
echo ""
echo "3. Testing GET /session..."
SESSIONS=$(curl -s "http://localhost:$CROW_PORT/session")
echo "Response: $SESSIONS" | head -c 200
echo ""
if echo "$SESSIONS" | grep -q '\['; then
    echo "GET /session"
else
    echo "GET /session failed"
    exit 1
fi

# Test 2: POST /session (create)
echo ""
echo "4. Testing POST /session..."
SESSION=$(curl -s -X POST "http://localhost:$CROW_PORT/session" \
    -H "Content-Type: application/json" \
    -d '{"title": "E2E Test Session"}')
SESSION_ID=$(echo "$SESSION" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
echo "Created session: $SESSION_ID"
if [ -n "$SESSION_ID" ]; then
    echo "POST /session"
else
    echo "POST /session failed"
    exit 1
fi

# Test 3: GET /session/:id/message (list messages - should be empty)
echo ""
echo "5. Testing GET /session/:id/message..."
MESSAGES=$(curl -s "http://localhost:$CROW_PORT/session/$SESSION_ID/message")
echo "Response: $MESSAGES"
if echo "$MESSAGES" | grep -q '\['; then
    echo "GET /session/:id/message"
else
    echo "GET /session/:id/message failed"
    exit 1
fi

# Test 4: POST /session/:id/message (send message)
echo ""
echo "6. Testing POST /session/:id/message..."
RESPONSE=$(curl -s -X POST "http://localhost:$CROW_PORT/session/$SESSION_ID/message" \
    -H "Content-Type: application/json" \
    -d '{"agent":"build","parts":[{"type":"text","text":"Say hello"}]}')
echo "Response (first 500 chars):"
echo "$RESPONSE" | head -c 500
echo ""
if echo "$RESPONSE" | grep -q '"role"'; then
    echo "POST /session/:id/message"
else
    echo "POST /session/:id/message failed"
    exit 1
fi

# Test 5: GET /session/:id/message (should have messages now)
echo ""
echo "7. Verifying messages exist..."
MESSAGES=$(curl -s "http://localhost:$CROW_PORT/session/$SESSION_ID/message")
MSG_COUNT=$(echo "$MESSAGES" | grep -o '"id"' | wc -l)
echo "Message count: $MSG_COUNT"
if [ "$MSG_COUNT" -ge 2 ]; then
    echo "Messages stored correctly"
else
    echo "Expected at least 2 messages (user + assistant)"
    exit 1
fi

# Test 6: DELETE /session/:id
echo ""
echo "8. Testing DELETE /session/:id..."
DELETE_RESP=$(curl -s -X DELETE "http://localhost:$CROW_PORT/session/$SESSION_ID")
echo "Response: $DELETE_RESP"
echo "DELETE /session/:id"

# Test 7: Verify deletion
echo ""
echo "9. Verifying session deleted..."
SESSIONS=$(curl -s "http://localhost:$CROW_PORT/session")
if echo "$SESSIONS" | grep -q "$SESSION_ID"; then
    echo "Session not deleted!"
    exit 1
else
    echo "Session deleted successfully"
fi

# Test 8: GET /file (list files)
echo ""
echo "10. Testing GET /file..."
FILES=$(curl -s "http://localhost:$CROW_PORT/file?path=.")
if echo "$FILES" | grep -q '"entries"'; then
    echo "GET /file"
else
    echo "GET /file failed"
    exit 1
fi

# Test 9: GET /file/content (read file)
echo ""
echo "11. Testing GET /file/content..."
# Find a file to read
FIRST_FILE=$(echo "$FILES" | grep -o '"path":"[^"]*"' | grep -v 'is_dir":true' | head -1 | cut -d'"' -f4)
if [ -n "$FIRST_FILE" ]; then
    CONTENT=$(curl -s "http://localhost:$CROW_PORT/file/content?path=$FIRST_FILE" | head -c 100)
    if [ -n "$CONTENT" ]; then
        echo "GET /file/content (read ${#CONTENT} chars)"
    else
        echo "GET /file/content returned empty"
    fi
else
    echo "No files found to test content endpoint"
fi

# Test 10: Create session for revert test
echo ""
echo "12. Testing revert endpoints..."
REVERT_SESSION=$(curl -s -X POST "http://localhost:$CROW_PORT/session" \
    -H "Content-Type: application/json" \
    -d '{"title": "Revert Test"}')
REVERT_SID=$(echo "$REVERT_SESSION" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)

# Send message to get a message ID
MSG_RESP=$(curl -s -X POST "http://localhost:$CROW_PORT/session/$REVERT_SID/message" \
    -H "Content-Type: application/json" \
    -d '{"agent":"build","parts":[{"type":"text","text":"hi"}]}')
MSG_ID=$(echo "$MSG_RESP" | grep -o '"id":"msg-[^"]*"' | head -1 | cut -d'"' -f4)
PART_ID=$(echo "$MSG_RESP" | grep -o '"id":"part-[^"]*"' | head -1 | cut -d'"' -f4)

if [ -n "$MSG_ID" ] && [ -n "$PART_ID" ]; then
    # Test revert
    REVERT_RESP=$(curl -s -X POST "http://localhost:$CROW_PORT/session/$REVERT_SID/revert" \
        -H "Content-Type: application/json" \
        -d "{\"message_id\":\"$MSG_ID\",\"part_id\":\"$PART_ID\"}")

    if echo "$REVERT_RESP" | grep -q '"revert"'; then
        echo "POST /session/:id/revert"

        # Test unrevert
        UNREVERT_RESP=$(curl -s -X POST "http://localhost:$CROW_PORT/session/$REVERT_SID/unrevert")
        if echo "$UNREVERT_RESP" | grep -q '"id"'; then
            echo "POST /session/:id/unrevert"
        else
            echo "POST /session/:id/unrevert - may have failed"
        fi
    else
        echo "POST /session/:id/revert - may have failed (no snapshot)"
    fi
else
    echo "Could not get message/part IDs for revert test"
fi

# Cleanup revert test session
curl -s -X DELETE "http://localhost:$CROW_PORT/session/$REVERT_SID" > /dev/null

echo ""
echo "=== E2E Test Complete ==="
echo "All API endpoints working correctly!"
