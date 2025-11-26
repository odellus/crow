#!/usr/bin/env bash
# TEST: Read Tool
# WHAT: Agent reads file contents
# AGENT CHECK: Agent reports file content correctly

set -euo pipefail
CROW_CLI="${CROW_CLI:-crow-cli}"
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"

# Setup test file
echo "Line one: hello" > test_file.txt
echo "Line two: world" >> test_file.txt
echo "Line three: crow" >> test_file.txt

SESSION=$("$CROW_CLI" new "Read Test" 2>&1 | grep -o 'ses_[a-zA-Z0-9]*' | head -1)
echo "Session: $SESSION"
echo "Test dir: $TEST_DIR"

echo ""
echo "=== Test File Contents ==="
cat test_file.txt

echo ""
echo "=== Test: Read File ==="
"$CROW_CLI" chat --session "$SESSION" "Read test_file.txt and tell me what's on line two"

echo ""
echo "=== AGENT: Verify ==="
echo "1. Agent used Read tool (not bash cat)"
echo "2. Agent correctly reports line two contains 'world'"
echo "3. Agent understood the content, didn't just dump it"

rm -rf "$TEST_DIR"
