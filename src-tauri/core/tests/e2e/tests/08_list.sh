#!/usr/bin/env bash
# TEST: List Tool
# WHAT: Agent lists directory contents
# AGENT CHECK: Shows files and directories correctly

set -euo pipefail
CROW_CLI="${CROW_CLI:-crow-cli}"
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"

# Setup test structure
mkdir -p docs src
touch README.md config.json
touch src/main.rs src/lib.rs
touch docs/guide.md

SESSION=$("$CROW_CLI" new "List Test" 2>&1 | grep -o 'ses_[a-zA-Z0-9]*' | head -1)
echo "Session: $SESSION"
echo "Test dir: $TEST_DIR"

echo ""
echo "=== Actual Structure ==="
ls -la
echo ""
ls -la src/

echo ""
echo "=== Test: List Directory ==="
"$CROW_CLI" chat --session "$SESSION" "List the files in the current directory"

echo ""
echo "=== AGENT: Verify ==="
echo "1. Shows README.md, config.json"
echo "2. Shows src/ and docs/ directories"
echo "3. Agent understands structure"

rm -rf "$TEST_DIR"
