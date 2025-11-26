#!/usr/bin/env bash
# TEST: Bash Tool
# WHAT: Agent executes bash commands
# AGENT CHECK: Output contains expected text, tool called once

set -euo pipefail
CROW_CLI="${CROW_CLI:-crow-cli}"
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"

SESSION=$("$CROW_CLI" new "Bash Test" 2>&1 | grep -o 'ses_[a-zA-Z0-9]*' | head -1)
echo "Session: $SESSION"
echo "Test dir: $TEST_DIR"

echo ""
echo "=== Test: Simple Echo ==="
"$CROW_CLI" chat --session "$SESSION" "Run: echo BASH_TEST_12345"

echo ""
echo "=== Test: Pipe Command ==="
"$CROW_CLI" chat --session "$SESSION" "Run: echo hello | tr a-z A-Z"

echo ""
echo "=== AGENT: Verify ==="
echo "1. First test output contains: BASH_TEST_12345"
echo "2. Second test output contains: HELLO"
echo "3. Check tool call count - should be 1 per test"
echo "4. RED FLAG: Multiple tool calls for simple echo = model confusion"

rm -rf "$TEST_DIR"
