#!/bin/bash
# Integration tests for crow interrupt/abort functionality
# Run from crow directory: ./test-integration.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PORT=7171
CROW_PID=""
TEST_DIR="/home/thomas/src/projects/opencode-project/test-dummy"

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}Cleaning up...${NC}"
    if [ -n "$CROW_PID" ]; then
        kill $CROW_PID 2>/dev/null || true
        wait $CROW_PID 2>/dev/null || true
    fi
}
trap cleanup EXIT

# Build crow
echo -e "${YELLOW}Building crow...${NC}"
cargo build --release --bin crow-serve --features server

# Start crow server
echo -e "${YELLOW}Starting crow server on port $PORT...${NC}"
./target/release/crow-serve --port $PORT &
CROW_PID=$!
sleep 2

# Check if server is running
if ! kill -0 $CROW_PID 2>/dev/null; then
    echo -e "${RED}FAIL: Server failed to start${NC}"
    exit 1
fi

echo -e "${GREEN}Server started with PID $CROW_PID${NC}"

# Helper function for tests
run_test() {
    local name="$1"
    echo -e "\n${YELLOW}TEST: $name${NC}"
}

pass() {
    echo -e "${GREEN}PASS: $1${NC}"
}

fail() {
    echo -e "${RED}FAIL: $1${NC}"
    exit 1
}

# =============================================================================
# TEST 1: Basic bash command works
# =============================================================================
run_test "Basic bash command execution"

SESSION_ID=$(curl -s http://localhost:$PORT/session -X POST \
  -H "Content-Type: application/json" \
  -d "{\"path\":\"$TEST_DIR\"}" | jq -r '.id')

if [ "$SESSION_ID" == "null" ] || [ -z "$SESSION_ID" ]; then
    fail "Could not create session"
fi

RESPONSE=$(curl -s "http://localhost:$PORT/session/$SESSION_ID/message" -X POST \
  -H "Content-Type: application/json" \
  -d '{"parts":[{"type":"text","text":"Run: echo hello-from-test"}]}')

if echo "$RESPONSE" | grep -q "hello-from-test"; then
    pass "Bash command executed and returned output"
else
    echo "Response: $RESPONSE"
    fail "Expected 'hello-from-test' in response"
fi

# =============================================================================
# TEST 2: Long-running bash command completes normally
# =============================================================================
run_test "Long-running bash command completes normally"

SESSION_ID=$(curl -s http://localhost:$PORT/session -X POST \
  -H "Content-Type: application/json" \
  -d "{\"path\":\"$TEST_DIR\"}" | jq -r '.id')

START_TIME=$(date +%s)

RESPONSE=$(curl -s "http://localhost:$PORT/session/$SESSION_ID/message" -X POST \
  -H "Content-Type: application/json" \
  -d '{"parts":[{"type":"text","text":"Run this command: sleep 2 && echo completed-after-sleep"}]}')

END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))

if echo "$RESPONSE" | grep -q "completed-after-sleep"; then
    if [ $ELAPSED -ge 2 ]; then
        pass "Long-running command completed in ${ELAPSED}s"
    else
        fail "Command completed too quickly (${ELAPSED}s), sleep may not have run"
    fi
else
    echo "Response: $RESPONSE"
    fail "Expected 'completed-after-sleep' in response"
fi

# =============================================================================
# TEST 3: Abort endpoint stops streaming request
# =============================================================================
run_test "Abort endpoint interrupts streaming request"

SESSION_ID=$(curl -s http://localhost:$PORT/session -X POST \
  -H "Content-Type: application/json" \
  -d "{\"path\":\"$TEST_DIR\"}" | jq -r '.id')

# Start a streaming request in background that would take a while
# Ask it to do something that takes time (multiple bash commands)
curl -s "http://localhost:$PORT/session/$SESSION_ID/message/stream" -X POST \
  -H "Content-Type: application/json" \
  -d '{"agent":"build","parts":[{"type":"text","text":"Run these commands one by one: sleep 5; echo first; sleep 5; echo second; sleep 5; echo third"}]}' > /tmp/stream_output.txt &
CURL_PID=$!

# Wait a moment for the request to start
sleep 1

# Abort the session
ABORT_RESPONSE=$(curl -s "http://localhost:$PORT/session/$SESSION_ID/abort" -X POST)

if echo "$ABORT_RESPONSE" | grep -q '"aborted":true'; then
    pass "Abort endpoint returned success"
else
    echo "Abort response: $ABORT_RESPONSE"
    fail "Abort endpoint did not return success"
fi

# Wait for curl to finish (it should finish quickly after abort)
sleep 2
if kill -0 $CURL_PID 2>/dev/null; then
    kill $CURL_PID 2>/dev/null || true
    fail "Streaming request did not terminate after abort"
fi

pass "Streaming request terminated after abort"

# =============================================================================
# TEST 4: Abort kills running bash process
# =============================================================================
run_test "Abort kills running bash process"

SESSION_ID=$(curl -s http://localhost:$PORT/session -X POST \
  -H "Content-Type: application/json" \
  -d "{\"path\":\"$TEST_DIR\"}" | jq -r '.id')

# Start a long-running bash command in background
curl -s "http://localhost:$PORT/session/$SESSION_ID/message/stream" -X POST \
  -H "Content-Type: application/json" \
  -d '{"agent":"build","parts":[{"type":"text","text":"Run: sleep 30"}]}' > /tmp/bash_output.txt &
CURL_PID=$!

# Wait for the command to start
sleep 2

# Check that sleep process exists
if pgrep -f "sleep 30" > /dev/null; then
    echo "  Sleep process is running"
else
    fail "Sleep process not found - bash may not have started"
fi

# Abort the session
curl -s "http://localhost:$PORT/session/$SESSION_ID/abort" -X POST > /dev/null

# Wait for process to be killed
sleep 1

# Check that sleep process is gone
if pgrep -f "sleep 30" > /dev/null; then
    fail "Sleep process still running after abort"
else
    pass "Sleep process was killed after abort"
fi

# Clean up curl
kill $CURL_PID 2>/dev/null || true

# =============================================================================
# TEST 5: Multiple sessions don't interfere
# =============================================================================
run_test "Multiple sessions work independently"

SESSION_1=$(curl -s http://localhost:$PORT/session -X POST \
  -H "Content-Type: application/json" \
  -d "{\"path\":\"$TEST_DIR\"}" | jq -r '.id')

SESSION_2=$(curl -s http://localhost:$PORT/session -X POST \
  -H "Content-Type: application/json" \
  -d "{\"path\":\"$TEST_DIR\"}" | jq -r '.id')

# Start long request on session 1
curl -s "http://localhost:$PORT/session/$SESSION_1/message/stream" -X POST \
  -H "Content-Type: application/json" \
  -d '{"agent":"build","parts":[{"type":"text","text":"Run: sleep 10 && echo session1done"}]}' > /tmp/session1.txt &
CURL_PID_1=$!

sleep 1

# Send quick request on session 2 - use simpler prompt that won't trigger doom loop
RESPONSE_2=$(curl -s "http://localhost:$PORT/session/$SESSION_2/message" -X POST \
  -H "Content-Type: application/json" \
  -d '{"parts":[{"type":"text","text":"What is 2+2? Just answer with the number."}]}')

if echo "$RESPONSE_2" | grep -q "4"; then
    pass "Session 2 completed while session 1 was running"
else
    echo "Response 2: $RESPONSE_2"
    fail "Session 2 did not complete properly"
fi

# Abort session 1
curl -s "http://localhost:$PORT/session/$SESSION_1/abort" -X POST > /dev/null
kill $CURL_PID_1 2>/dev/null || true

# =============================================================================
# TEST 6: Streaming returns proper SSE events
# =============================================================================
run_test "Streaming returns proper SSE events"

SESSION_ID=$(curl -s http://localhost:$PORT/session -X POST \
  -H "Content-Type: application/json" \
  -d "{\"path\":\"$TEST_DIR\"}" | jq -r '.id')

# Get streaming response
STREAM_OUTPUT=$(curl -s "http://localhost:$PORT/session/$SESSION_ID/message/stream" -X POST \
  -H "Content-Type: application/json" \
  -d '{"agent":"build","parts":[{"type":"text","text":"Say exactly: hello world"}]}' \
  --max-time 30)

# Check for expected SSE events
if echo "$STREAM_OUTPUT" | grep -q "event: message.start"; then
    pass "Found message.start event"
else
    fail "Missing message.start event"
fi

if echo "$STREAM_OUTPUT" | grep -q "event: message.complete"; then
    pass "Found message.complete event"
else
    echo "Stream output:"
    echo "$STREAM_OUTPUT"
    fail "Missing message.complete event"
fi

# =============================================================================
# SUMMARY
# =============================================================================
echo -e "\n${GREEN}========================================${NC}"
echo -e "${GREEN}All integration tests passed!${NC}"
echo -e "${GREEN}========================================${NC}"
