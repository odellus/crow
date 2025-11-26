#!/usr/bin/env bash
# E2E tests for crow-cli - Real agent execution tests
# Run with: bash test_crow_e2e.sh

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

PASSED=0
FAILED=0

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CROW_CLI="$SCRIPT_DIR/../../../target/debug/crow-cli"

if [[ ! -f "$CROW_CLI" ]]; then
    echo "Building crow-cli..."
    cd "$SCRIPT_DIR/../../" && cargo build --bin crow-cli
fi

TEST_DIR=$(mktemp -d -t crow-e2e-XXXXXX)
cd "$TEST_DIR"
echo -e "${CYAN}Test dir: $TEST_DIR${NC}"

cleanup() { rm -rf "$TEST_DIR"; }
trap cleanup EXIT

pass() { echo -e "${GREEN}[PASS]${NC} $1"; ((PASSED++)) || true; }
fail() { echo -e "${RED}[FAIL]${NC} $1"; ((FAILED++)) || true; }

new_session() {
    "$CROW_CLI" new "$1" 2>&1 | grep -o 'ses_[a-zA-Z0-9]*' | head -1
}

# ============================================================
echo -e "\n${CYAN}=== Session Management ===${NC}"
# ============================================================
SESSION=$(new_session "E2E Test")
if [[ -n "$SESSION" ]]; then
    pass "Create session: $SESSION"
else
    fail "Create session"
    exit 1
fi

# ============================================================
echo -e "\n${CYAN}=== Bash Tool ===${NC}"
# ============================================================
BASH_S=$(new_session "Bash")
output=$("$CROW_CLI" chat --session "$BASH_S" "Run: echo CROW_E2E_SUCCESS" 2>&1)
if echo "$output" | grep -q "CROW_E2E_SUCCESS"; then
    pass "Bash: echo command"
else
    fail "Bash: echo command"
fi

# ============================================================
echo -e "\n${CYAN}=== Write Tool ===${NC}"
# ============================================================
WRITE_S=$(new_session "Write")
"$CROW_CLI" chat --session "$WRITE_S" "Create file hello.txt with content: Written by Crow" 2>&1 > /dev/null
sleep 1
if [[ -f "hello.txt" ]] && grep -q "Crow" hello.txt 2>/dev/null; then
    pass "Write: create file"
    echo "  Content: $(cat hello.txt)"
else
    fail "Write: create file"
    ls -la
fi

# ============================================================
echo -e "\n${CYAN}=== Read Tool ===${NC}"
# ============================================================
READ_S=$(new_session "Read")
echo "This is test content for reading" > read_test.txt
output=$("$CROW_CLI" chat --session "$READ_S" "Read read_test.txt and tell me what it says" 2>&1)
if echo "$output" | grep -qi "test content\|reading"; then
    pass "Read: file contents"
else
    fail "Read: file contents"
fi

# ============================================================
echo -e "\n${CYAN}=== Edit Tool ===${NC}"
# ============================================================
EDIT_S=$(new_session "Edit")
echo "Hello World" > edit_test.txt
"$CROW_CLI" chat --session "$EDIT_S" "Edit edit_test.txt: replace World with Crow" 2>&1 > /dev/null
sleep 1
if grep -q "Crow" edit_test.txt 2>/dev/null; then
    pass "Edit: replacement"
    echo "  Content: $(cat edit_test.txt)"
else
    fail "Edit: replacement"
    echo "  Content: $(cat edit_test.txt 2>/dev/null || echo 'file missing')"
fi

# ============================================================
echo -e "\n${CYAN}=== Grep Tool ===${NC}"
# ============================================================
GREP_S=$(new_session "Grep")
echo "ERROR: something failed" > error_log.txt
echo "INFO: all good" > info_log.txt
output=$("$CROW_CLI" chat --session "$GREP_S" "Use grep to find files with ERROR" 2>&1)
if echo "$output" | grep -qi "error_log\|ERROR\|found"; then
    pass "Grep: pattern search"
else
    fail "Grep: pattern search"
fi

# ============================================================
echo -e "\n${CYAN}=== Glob Tool ===${NC}"
# ============================================================
GLOB_S=$(new_session "Glob")
mkdir -p src
touch src/app.js src/main.ts src/utils.js
output=$("$CROW_CLI" chat --session "$GLOB_S" "Use glob to find .js files" 2>&1)
if echo "$output" | grep -qi "app.js\|utils.js\|js"; then
    pass "Glob: find by extension"
else
    fail "Glob: find by extension"
fi

# ============================================================
echo -e "\n${CYAN}=== List Tool ===${NC}"
# ============================================================
LIST_S=$(new_session "List")
output=$("$CROW_CLI" chat --session "$LIST_S" "List files in current directory" 2>&1)
if echo "$output" | grep -qi "src\|txt\|hello"; then
    pass "List: directory contents"
else
    fail "List: directory contents"
fi

# ============================================================
echo ""
echo "============================================"
echo -e "${CYAN}E2E Tests Complete${NC}"
echo -e "  ${GREEN}Passed: $PASSED${NC}"
echo -e "  ${RED}Failed: $FAILED${NC}"
echo "============================================"

[[ $FAILED -eq 0 ]]
