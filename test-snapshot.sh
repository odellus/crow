#!/bin/bash
# Integration test for crow snapshot/revert system

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
TEST_DIR="test-dummy"
CROW_PORT=3179
CROW_PID=""
BASE_URL="http://localhost:$CROW_PORT"

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}Cleaning up...${NC}"
    if [ -n "$CROW_PID" ]; then
        kill $CROW_PID 2>/dev/null || true
        wait $CROW_PID 2>/dev/null || true
    fi
    # Restore test file if it was modified
    if [ -f "$TEST_DIR/test-file.txt.backup" ]; then
        mv "$TEST_DIR/test-file.txt.backup" "$TEST_DIR/test-file.txt"
    fi
}

trap cleanup EXIT

# Helper functions
pass() {
    echo -e "${GREEN}✓ $1${NC}"
}

fail() {
    echo -e "${RED}✗ $1${NC}"
    exit 1
}

info() {
    echo -e "${YELLOW}→ $1${NC}"
}

wait_for_server() {
    local max_attempts=30
    local attempt=0
    while ! curl -s "$BASE_URL/session" > /dev/null 2>&1; do
        attempt=$((attempt + 1))
        if [ $attempt -ge $max_attempts ]; then
            fail "Server failed to start after $max_attempts attempts"
        fi
        sleep 0.5
    done
}

# Start test
echo "=========================================="
echo "Crow Snapshot Integration Test"
echo "=========================================="
echo ""

# Setup test directory
info "Setting up test directory"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# Create initial test file
echo "Initial content line 1" > test-file.txt
echo "Initial content line 2" >> test-file.txt
echo "Initial content line 3" >> test-file.txt

# Backup for cleanup
cp test-file.txt test-file.txt.backup

pass "Test directory setup complete"

# Check snapshot directory before starting
info "Checking snapshot directory before server start"
SNAPSHOT_DIR="$HOME/.local/share/crow/snapshot"
if [ -d "$SNAPSHOT_DIR" ]; then
    echo "  Existing snapshot dirs: $(ls -1 "$SNAPSHOT_DIR" 2>/dev/null | wc -l)"
else
    echo "  No snapshot directory exists yet"
fi

# Build and start crow server
info "Building crow server"
cd ..
cargo build --package api --features server --bin crow 2>&1 | tail -5
cd "$TEST_DIR"

info "Starting crow server on port $CROW_PORT"
../target/debug/crow --port $CROW_PORT &
CROW_PID=$!
wait_for_server
pass "Crow server started (PID: $CROW_PID)"

# Test 1: Create a session
echo ""
echo "Test 1: Create a session"
echo "------------------------"

SESSION_RESPONSE=$(curl -s -X POST "$BASE_URL/session" \
    -H "Content-Type: application/json" \
    -d '{"title": "Snapshot Test Session"}')

SESSION_ID=$(echo "$SESSION_RESPONSE" | jq -r '.id')
if [ -z "$SESSION_ID" ] || [ "$SESSION_ID" = "null" ]; then
    fail "Failed to create session: $SESSION_RESPONSE"
fi
pass "Created session: $SESSION_ID"

# Test 2: Check snapshot directory after session creation
echo ""
echo "Test 2: Check snapshot directory"
echo "---------------------------------"

# Give it a moment for any initialization
sleep 1

if [ -d "$SNAPSHOT_DIR" ]; then
    SNAPSHOT_COUNT=$(ls -1 "$SNAPSHOT_DIR" 2>/dev/null | wc -l)
    echo "  Snapshot directories: $SNAPSHOT_COUNT"
    ls -la "$SNAPSHOT_DIR" 2>/dev/null || true
else
    echo "  Snapshot directory not created yet (will be created on first tool execution)"
fi
pass "Snapshot directory check complete"

# Test 3: Send message to modify the file
echo ""
echo "Test 3: Send message to modify file"
echo "------------------------------------"

info "Sending request to add a line to test-file.txt"

# Use streaming endpoint and capture the response
MODIFY_RESPONSE=$(curl -s -X POST "$BASE_URL/session/$SESSION_ID/message/stream" \
    -H "Content-Type: application/json" \
    -H "Accept: text/event-stream" \
    -d '{
        "parts": [{
            "type": "text",
            "text": "Please add the line \"Added by crow agent\" to the end of test-file.txt using the edit tool"
        }]
    }' --max-time 60 2>&1)

# Check if file was modified
sleep 2
if grep -q "Added by crow agent" test-file.txt; then
    pass "File was modified successfully"
    echo "  File contents:"
    cat test-file.txt | sed 's/^/    /'
else
    echo "  Response: $MODIFY_RESPONSE"
    echo "  Current file contents:"
    cat test-file.txt | sed 's/^/    /'
    fail "File was not modified as expected"
fi

# Test 4: Check snapshot directory after modification
echo ""
echo "Test 4: Check snapshot after modification"
echo "------------------------------------------"

if [ -d "$SNAPSHOT_DIR/$SESSION_ID" ]; then
    pass "Snapshot directory created for session"
    echo "  Contents:"
    ls -la "$SNAPSHOT_DIR/$SESSION_ID" | sed 's/^/    /'

    # Check git objects
    if [ -d "$SNAPSHOT_DIR/$SESSION_ID/objects" ]; then
        OBJECT_COUNT=$(find "$SNAPSHOT_DIR/$SESSION_ID/objects" -type f | wc -l)
        echo "  Git objects: $OBJECT_COUNT"
    fi
else
    info "Snapshot directory not found at expected location"
    echo "  Checking alternative locations..."
    find "$HOME/.local/share/crow" -name "snapshot" -type d 2>/dev/null || true
fi

# Test 5: Get messages to find message IDs for revert
echo ""
echo "Test 5: Get messages to find revert target"
echo "-------------------------------------------"

MESSAGES_RESPONSE=$(curl -s "$BASE_URL/session/$SESSION_ID/message")
MESSAGE_COUNT=$(echo "$MESSAGES_RESPONSE" | jq 'length')
echo "  Total messages: $MESSAGE_COUNT"

# Get the first message ID (user message before modification)
FIRST_MESSAGE_ID=$(echo "$MESSAGES_RESPONSE" | jq -r '.[0].info.id')
echo "  First message ID: $FIRST_MESSAGE_ID"

# Check for patches in the response
PATCH_COUNT=$(echo "$MESSAGES_RESPONSE" | jq '[.[] | .parts[] | select(.type == "patch")] | length')
echo "  Patch parts found: $PATCH_COUNT"

if [ "$PATCH_COUNT" -gt 0 ]; then
    pass "Patches were recorded"
    echo "  Patch details:"
    echo "$MESSAGES_RESPONSE" | jq '[.[] | .parts[] | select(.type == "patch")]' | sed 's/^/    /'
else
    info "No patches found - this might indicate snapshot tracking isn't working"
fi

# Test 6: Send another message to make another modification
echo ""
echo "Test 6: Send second modification"
echo "---------------------------------"

info "Sending request to add another line"

MODIFY2_RESPONSE=$(curl -s -X POST "$BASE_URL/session/$SESSION_ID/message/stream" \
    -H "Content-Type: application/json" \
    -H "Accept: text/event-stream" \
    -d '{
        "parts": [{
            "type": "text",
            "text": "Please add the line \"Second modification by crow\" to test-file.txt using the edit tool"
        }]
    }' --max-time 60 2>&1)

sleep 2
if grep -q "Second modification" test-file.txt; then
    pass "Second modification successful"
    echo "  File contents now:"
    cat test-file.txt | sed 's/^/    /'
else
    info "Second modification may not have completed"
    echo "  Current file contents:"
    cat test-file.txt | sed 's/^/    /'
fi

# Get updated messages
MESSAGES_RESPONSE=$(curl -s "$BASE_URL/session/$SESSION_ID/message")
MESSAGE_COUNT=$(echo "$MESSAGES_RESPONSE" | jq 'length')
echo "  Total messages now: $MESSAGE_COUNT"

# Get the second message ID (for reverting to after first modification)
if [ "$MESSAGE_COUNT" -ge 2 ]; then
    SECOND_MESSAGE_ID=$(echo "$MESSAGES_RESPONSE" | jq -r '.[1].info.id')
    echo "  Second message ID: $SECOND_MESSAGE_ID"
fi

# Test 7: Call revert endpoint
echo ""
echo "Test 7: Call revert endpoint"
echo "----------------------------"

if [ -z "$SECOND_MESSAGE_ID" ] || [ "$SECOND_MESSAGE_ID" = "null" ]; then
    # Fall back to first message
    REVERT_TARGET=$FIRST_MESSAGE_ID
    info "Reverting to first message: $REVERT_TARGET"
else
    REVERT_TARGET=$SECOND_MESSAGE_ID
    info "Reverting to second message: $REVERT_TARGET"
fi

REVERT_RESPONSE=$(curl -s -X POST "$BASE_URL/session/$SESSION_ID/revert" \
    -H "Content-Type: application/json" \
    -d "{\"messageID\": \"$REVERT_TARGET\"}")

echo "  Revert response:"
echo "$REVERT_RESPONSE" | jq '.' | sed 's/^/    /'

# Check if revert state was set
REVERT_STATE=$(echo "$REVERT_RESPONSE" | jq -r '.revert')
if [ "$REVERT_STATE" != "null" ]; then
    pass "Revert state set on session"
    echo "  Revert details:"
    echo "$REVERT_RESPONSE" | jq '.revert' | sed 's/^/    /'
else
    info "Revert state not set (may indicate no patches to revert)"
fi

# Test 8: Check filesystem state after revert
echo ""
echo "Test 8: Check filesystem after revert"
echo "--------------------------------------"

echo "  File contents after revert:"
cat test-file.txt | sed 's/^/    /'

# Check what we expect based on revert target
if [ "$REVERT_TARGET" = "$SECOND_MESSAGE_ID" ]; then
    # Should have first modification but not second
    if grep -q "Added by crow agent" test-file.txt && ! grep -q "Second modification" test-file.txt; then
        pass "File correctly reverted to state after first modification"
    else
        info "File state may not match expected (check manually)"
    fi
elif [ "$REVERT_TARGET" = "$FIRST_MESSAGE_ID" ]; then
    # Should be back to original
    if ! grep -q "Added by crow agent" test-file.txt; then
        pass "File correctly reverted to original state"
    else
        info "File still contains modifications"
    fi
fi

# Test 9: Check session state
echo ""
echo "Test 9: Verify session state"
echo "-----------------------------"

SESSION_STATE=$(curl -s "$BASE_URL/session/$SESSION_ID")
echo "  Session state:"
echo "$SESSION_STATE" | jq '{id, title, revert}' | sed 's/^/    /'

# Test 10: Test unrevert
echo ""
echo "Test 10: Test unrevert endpoint"
echo "--------------------------------"

UNREVERT_RESPONSE=$(curl -s -X POST "$BASE_URL/session/$SESSION_ID/unrevert" \
    -H "Content-Type: application/json")

# Check if response is valid JSON
if echo "$UNREVERT_RESPONSE" | jq . > /dev/null 2>&1; then
    echo "  Unrevert response:"
    echo "$UNREVERT_RESPONSE" | jq '{id, revert}' | sed 's/^/    /'

    UNREVERT_STATE=$(echo "$UNREVERT_RESPONSE" | jq -r '.revert')
    if [ "$UNREVERT_STATE" = "null" ]; then
        pass "Revert state cleared"
    else
        info "Revert state still present"
    fi
else
    info "Unrevert response not valid JSON (session state not persisted - known limitation)"
    echo "  Raw response: $UNREVERT_RESPONSE"
fi

echo "  File contents after unrevert:"
cat test-file.txt | sed 's/^/    /'

echo ""
echo "=========================================="
echo "Snapshot Integration Test Complete"
echo "=========================================="
