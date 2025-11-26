#!/usr/bin/env bash
# TEST: Error Handling
# WHAT: Agent handles errors gracefully
# AGENT CHECK: No hallucination, appropriate error reporting

set -euo pipefail
CROW_CLI="${CROW_CLI:-crow-cli}"
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"

SESSION=$("$CROW_CLI" new "Error Test" 2>&1 | grep -o 'ses_[a-zA-Z0-9]*' | head -1)
echo "Session: $SESSION"
echo "Test dir: $TEST_DIR"

echo ""
echo "=== Test: Read Nonexistent File ==="
"$CROW_CLI" chat --session "$SESSION" "Read the file does_not_exist_12345.txt"

echo ""
echo "=== AGENT: Verify ==="
echo "1. Agent reports file doesn't exist"
echo "2. Agent does NOT hallucinate file contents"
echo "3. Agent does NOT retry excessively"
echo "4. Error handled gracefully"

rm -rf "$TEST_DIR"
