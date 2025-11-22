#!/bin/bash
# Integration test for bash tool - compares Crow and OpenCode behavior

set -e

TEST_DIR="/tmp/crow-bash-test-$$"
OPENCODE_PORT=4098
CROW_PORT=4099

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

cleanup() {
    echo "Cleaning up..."
    pkill -f "opencode serve -p $OPENCODE_PORT" 2>/dev/null || true
    pkill -f "crow-serve --port $CROW_PORT" 2>/dev/null || true
    rm -rf "$TEST_DIR"
}

trap cleanup EXIT

echo "=== Bash Tool Integration Test ==="
echo ""

# Create test directory
mkdir -p "$TEST_DIR"
mkdir -p "$TEST_DIR/.crow"
cat > "$TEST_DIR/.crow/config.jsonc" << 'EOF'
{
  "model": "moonshotai/kimi-k2-thinking"
}
EOF

echo "Test directory: $TEST_DIR"
echo ""

# Start OpenCode server
echo "Starting OpenCode server on port $OPENCODE_PORT..."
cd "$TEST_DIR"
OPENCODE_VERBOSE_LOG=1 opencode serve -p $OPENCODE_PORT &
sleep 3

# Start Crow server
echo "Starting Crow server on port $CROW_PORT..."
CROW_BIN="/home/thomas/src/projects/opencode-project/crow/target/release/crow-serve"
if [ ! -f "$CROW_BIN" ]; then
    echo -e "${RED}Error: Crow binary not found at $CROW_BIN${NC}"
    exit 1
fi
CROW_VERBOSE_LOG=1 "$CROW_BIN" --port $CROW_PORT &
sleep 2

# Verify servers are running
echo "Verifying servers..."
curl -s http://localhost:$OPENCODE_PORT/session > /dev/null || { echo -e "${RED}OpenCode not running${NC}"; exit 1; }
curl -s http://localhost:$CROW_PORT/session > /dev/null || { echo -e "${RED}Crow not running${NC}"; exit 1; }
echo -e "${GREEN}Both servers running${NC}"
echo ""

# Create sessions
echo "Creating sessions..."
OC_SID=$(curl -s "http://localhost:$OPENCODE_PORT/session" -X POST \
    -H "Content-Type: application/json" \
    -d '{}' | jq -r '.id')
CROW_SID=$(curl -s "http://localhost:$CROW_PORT/session" -X POST \
    -H "Content-Type: application/json" \
    -d "{\"path\":\"$TEST_DIR\"}" | jq -r '.id')

echo "OpenCode session: $OC_SID"
echo "Crow session: $CROW_SID"
echo ""

# Test function
test_bash() {
    local test_name=$1
    local command=$2

    echo "=== Test: $test_name ==="
    echo "Command: $command"
    echo ""

    # Send to OpenCode
    echo "Sending to OpenCode..."
    OC_RESPONSE=$(curl -s --max-time 60 "http://localhost:$OPENCODE_PORT/session/$OC_SID/message" -X POST \
        -H "Content-Type: application/json" \
        -d "{\"parts\":[{\"type\":\"text\",\"text\":\"Run this bash command: $command\"}]}")

    # Send to Crow
    echo "Sending to Crow..."
    CROW_RESPONSE=$(curl -s --max-time 60 "http://localhost:$CROW_PORT/session/$CROW_SID/message" -X POST \
        -H "Content-Type: application/json" \
        -d "{\"parts\":[{\"type\":\"text\",\"text\":\"Run this bash command: $command\"}]}")

    # Extract bash tool results
    OC_BASH=$(echo "$OC_RESPONSE" | jq -r '.parts[] | select(.tool == "bash") | .state.output // .state.metadata.output // "N/A"' 2>/dev/null | head -1)
    CROW_BASH=$(echo "$CROW_RESPONSE" | jq -r '.parts[] | select(.tool == "bash") | .state.output // .state.metadata.output // "N/A"' 2>/dev/null | head -1)

    echo "OpenCode output: ${OC_BASH:0:100}..."
    echo "Crow output: ${CROW_BASH:0:100}..."

    if [ "$OC_BASH" != "N/A" ] && [ "$CROW_BASH" != "N/A" ]; then
        # Compare outputs (trim whitespace for comparison)
        OC_TRIMMED=$(echo "$OC_BASH" | tr -d '[:space:]')
        CROW_TRIMMED=$(echo "$CROW_BASH" | tr -d '[:space:]')

        if [ "$OC_TRIMMED" = "$CROW_TRIMMED" ]; then
            echo -e "${GREEN}PASS: Outputs match${NC}"
        else
            echo -e "${YELLOW}WARN: Outputs differ (may be expected)${NC}"
        fi
    else
        echo -e "${YELLOW}WARN: Could not extract bash output${NC}"
    fi
    echo ""
}

# Run tests
test_bash "echo" "echo hello world"
test_bash "pwd" "pwd"
test_bash "date" "date +%Y-%m-%d"
test_bash "arithmetic" "echo \$((2 + 2))"
test_bash "multiline" "echo line1 && echo line2"

echo ""
echo "=== Checking verbose logs ==="
echo ""

# Check verbose logs
OC_LOG_DIR="$HOME/.local/share/opencode/verbose"
CROW_LOG_DIR="$HOME/.local/share/crow/requests"

if [ -d "$OC_LOG_DIR" ]; then
    OC_LATEST=$(ls -t "$OC_LOG_DIR" 2>/dev/null | head -1)
    if [ -n "$OC_LATEST" ]; then
        echo "OpenCode latest log: $OC_LOG_DIR/$OC_LATEST"
    fi
fi

if [ -d "$CROW_LOG_DIR" ]; then
    CROW_LATEST=$(ls -t "$CROW_LOG_DIR" 2>/dev/null | head -1)
    if [ -n "$CROW_LATEST" ]; then
        echo "Crow latest log: $CROW_LOG_DIR/$CROW_LATEST"
    fi
fi

echo ""
echo "=== Integration test completed ==="
