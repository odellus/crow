#!/usr/bin/env bash
# TEST: Edit Tool
# WHAT: Agent modifies existing file content
# AGENT CHECK: File changed on disk, diff shown in output

set -euo pipefail
CROW_CLI="${CROW_CLI:-crow-cli}"
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"

# Setup test file
echo "Hello World" > edit_me.txt

SESSION=$("$CROW_CLI" new "Edit Test" 2>&1 | grep -o 'ses_[a-zA-Z0-9]*' | head -1)
echo "Session: $SESSION"
echo "Test dir: $TEST_DIR"

echo ""
echo "=== Before Edit ==="
cat edit_me.txt

echo ""
echo "=== Test: Edit File ==="
"$CROW_CLI" chat --session "$SESSION" "Edit edit_me.txt: replace World with Crow"

echo ""
echo "=== After Edit ==="
cat edit_me.txt

echo ""
echo "=== AGENT: Verify ==="
echo "1. File now contains 'Hello Crow' (not 'Hello World')"
echo "2. Agent read file first, then edited"
echo "3. Diff was shown in output"
echo "4. RED FLAG: Edit tool called multiple times"
echo "5. RED FLAG: File unchanged = tool execution bug"

rm -rf "$TEST_DIR"
