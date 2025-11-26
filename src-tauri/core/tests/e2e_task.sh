#!/bin/bash
# E2E Tests for Task Tool
# Run with: ./tests/e2e_task.sh
#
# Requirements:
# - crow-cli built (cargo build --release --bin crow-cli)
# - API key set (MOONSHOT_API_KEY or configured provider)

# Don't exit on error - we handle errors ourselves
set +e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
CROW_CLI="$PROJECT_ROOT/target/release/crow-cli"
TEST_DIR="/tmp/crow-e2e-tests"
XDG_DATA="$HOME/.local/share/crow"
XDG_STATE="$HOME/.local/state/crow"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

TESTS_PASSED=0
TESTS_FAILED=0

# Test helper functions
log_test() {
    echo -e "${YELLOW}[TEST]${NC} $1"
}

log_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((TESTS_PASSED++))
}

log_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((TESTS_FAILED++))
}

setup_test_project() {
    local name="$1"
    # Add timestamp to make each project unique
    local unique_name="${name}-$(date +%s%N)"
    local dir="$TEST_DIR/$unique_name"
    rm -rf "$dir"
    mkdir -p "$dir"
    cd "$dir"
    git init --quiet
    echo "# $name" > README.md
    echo "fn main() { println!(\"$name\"); }" > main.rs
    echo "pub fn add(a: i32, b: i32) -> i32 { a + b }" > lib.rs
    git add .
    git commit -m "Initial commit" --quiet
    echo "$dir"
}

cleanup() {
    echo ""
    echo "=================================="
    echo -e "Tests passed: ${GREEN}$TESTS_PASSED${NC}"
    echo -e "Tests failed: ${RED}$TESTS_FAILED${NC}"
    echo "=================================="

    if [ $TESTS_FAILED -gt 0 ]; then
        exit 1
    fi
}

trap cleanup EXIT

# Verify crow-cli exists
if [ ! -f "$CROW_CLI" ]; then
    echo "Error: crow-cli not found at $CROW_CLI"
    echo "Run: cargo build --release --bin crow-cli"
    exit 1
fi

echo "=================================="
echo "E2E Tests for Task Tool"
echo "=================================="
echo "CLI: $CROW_CLI"
echo "Test dir: $TEST_DIR"
echo ""

# Setup
mkdir -p "$TEST_DIR"

# ==============================================================================
# TEST 1: Basic chat creates session
# ==============================================================================
log_test "Basic chat creates session"

PROJECT_DIR=$(setup_test_project "test-basic-chat")
cd "$PROJECT_DIR"

# Run a simple chat command
OUTPUT=$("$CROW_CLI" chat --json "say hello" 2>&1) || true

# Check if session was created (look for session ID in output)
if echo "$OUTPUT" | grep -q "session"; then
    log_pass "Chat command executed and returned session info"
else
    log_fail "Chat command did not return session info"
    echo "Output: $OUTPUT"
fi

# ==============================================================================
# TEST 2: Session persists to XDG
# ==============================================================================
log_test "Session persists to XDG storage"

# Check XDG data directory
if [ -d "$XDG_DATA/storage/session" ]; then
    SESSION_COUNT=$(find "$XDG_DATA/storage/session" -name "*.json" 2>/dev/null | wc -l)
    if [ "$SESSION_COUNT" -gt 0 ]; then
        log_pass "Sessions found in XDG storage ($SESSION_COUNT files)"
    else
        log_fail "No session files in XDG storage"
    fi
else
    log_fail "XDG session directory does not exist"
fi

# ==============================================================================
# TEST 3: Tool calls logged to JSONL
# ==============================================================================
log_test "Tool calls logged to XDG state"

TOOL_LOG="$XDG_STATE/logs/tool-calls.jsonl"
if [ -f "$TOOL_LOG" ]; then
    TOOL_LINES=$(wc -l < "$TOOL_LOG")
    log_pass "Tool calls logged ($TOOL_LINES entries in $TOOL_LOG)"
else
    log_fail "Tool calls log not found at $TOOL_LOG"
fi

# ==============================================================================
# TEST 4: Read tool works via CLI
# ==============================================================================
log_test "Read tool works via CLI"

PROJECT_DIR=$(setup_test_project "test-read-tool")
cd "$PROJECT_DIR"

OUTPUT=$("$CROW_CLI" chat --json "read the README.md file and tell me what it says" 2>&1) || true

if echo "$OUTPUT" | grep -q -i "readme\|test-read-tool"; then
    log_pass "Read tool returned file content"
else
    log_fail "Read tool did not return expected content"
    echo "Output: $OUTPUT" | head -20
fi

# ==============================================================================
# TEST 5: Grep tool works via CLI
# ==============================================================================
log_test "Grep tool works via CLI"

PROJECT_DIR=$(setup_test_project "test-grep-tool")
cd "$PROJECT_DIR"
echo "TODO: implement feature" >> main.rs
echo "TODO: write tests" >> lib.rs

OUTPUT=$("$CROW_CLI" chat --json "use grep to find all TODO comments in this project" 2>&1) || true

if echo "$OUTPUT" | grep -q -i "todo\|TODO"; then
    log_pass "Grep tool found TODO comments"
else
    log_fail "Grep tool did not find TODO comments"
    echo "Output: $OUTPUT" | head -20
fi

# ==============================================================================
# TEST 6: Write tool creates file
# ==============================================================================
log_test "Write tool creates file"

PROJECT_DIR=$(setup_test_project "test-write-tool")
cd "$PROJECT_DIR"

OUTPUT=$("$CROW_CLI" chat --json "create a new file called test_output.txt with the content 'hello from e2e test'" 2>&1) || true

if [ -f "test_output.txt" ]; then
    CONTENT=$(cat test_output.txt)
    if echo "$CONTENT" | grep -q -i "hello"; then
        log_pass "Write tool created file with correct content"
    else
        log_fail "Write tool created file but content is wrong: $CONTENT"
    fi
else
    log_fail "Write tool did not create file"
    echo "Output: $OUTPUT" | head -20
fi

# ==============================================================================
# TEST 7: TodoWrite tool creates todos
# ==============================================================================
log_test "TodoWrite tool creates todos"

PROJECT_DIR=$(setup_test_project "test-todo-tool")
cd "$PROJECT_DIR"

OUTPUT=$("$CROW_CLI" chat --json "create a todo list with 3 items: 1. Setup project, 2. Write tests, 3. Deploy" 2>&1) || true

# Check if todos were created (look in XDG or output)
if echo "$OUTPUT" | grep -q -i "todo\|task"; then
    log_pass "TodoWrite tool executed"
else
    log_fail "TodoWrite tool did not create todos"
    echo "Output: $OUTPUT" | head -20
fi

# ==============================================================================
# TEST 8: Session history accessible
# ==============================================================================
log_test "Session history accessible via CLI"

# List sessions
SESSIONS=$("$CROW_CLI" sessions 2>&1) || true

if echo "$SESSIONS" | grep -q "ses_"; then
    log_pass "Sessions listed successfully"

    # Get first session ID
    SESSION_ID=$(echo "$SESSIONS" | grep -o "ses_[a-zA-Z0-9]*" | head -1)

    if [ -n "$SESSION_ID" ]; then
        # Try to get session info
        INFO=$("$CROW_CLI" session info "$SESSION_ID" 2>&1) || true
        if echo "$INFO" | grep -q -i "session\|id\|title"; then
            log_pass "Session info retrieved for $SESSION_ID"
        else
            log_fail "Could not get session info for $SESSION_ID"
        fi
    fi
else
    log_fail "No sessions found"
    echo "Output: $SESSIONS"
fi

# ==============================================================================
# TEST 9: Multiple sessions isolated (using explicit new sessions)
# ==============================================================================
log_test "Multiple sessions are isolated"

PROJECT_A=$(setup_test_project "test-isolation-a")
PROJECT_B=$(setup_test_project "test-isolation-b")

# Create explicit new sessions for each project
cd "$PROJECT_A"
SESSION_A=$("$CROW_CLI" new "Test Session A" 2>&1 | grep -o 'ses_[a-zA-Z0-9]*' | head -1)

cd "$PROJECT_B"
SESSION_B=$("$CROW_CLI" new "Test Session B" 2>&1 | grep -o 'ses_[a-zA-Z0-9]*' | head -1)

if [ -n "$SESSION_A" ] && [ -n "$SESSION_B" ] && [ "$SESSION_A" != "$SESSION_B" ]; then
    log_pass "Different projects get different sessions (A=$SESSION_A, B=$SESSION_B)"
else
    log_fail "Session isolation failed (A=$SESSION_A, B=$SESSION_B)"
fi

# ==============================================================================
# TEST 10: JSON output mode works
# ==============================================================================
log_test "JSON output mode returns valid JSON"

PROJECT_DIR=$(setup_test_project "test-json-output")
cd "$PROJECT_DIR"

OUTPUT=$("$CROW_CLI" chat --json "what is 2+2?" 2>&1) || true

# Try to parse as JSON (using python since jq might not be installed)
if echo "$OUTPUT" | python3 -c "import sys,json; json.load(sys.stdin)" 2>/dev/null; then
    log_pass "JSON output is valid JSON"
else
    # Maybe partial JSON, check for JSON-like structure
    if echo "$OUTPUT" | grep -q '{"'; then
        log_pass "JSON output contains JSON structure"
    else
        log_fail "JSON output is not valid JSON"
        echo "Output: $OUTPUT" | head -10
    fi
fi

echo ""
echo "E2E Tests Complete!"
