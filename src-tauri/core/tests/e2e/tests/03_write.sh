#!/usr/bin/env bash
# TEST: Write Tool
# WHAT: Agent creates files with content
# AGENT CHECK: File exists on disk with correct content

set -euo pipefail
CROW_CLI="${CROW_CLI:-crow-cli}"
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"

SESSION=$("$CROW_CLI" new "Write Test" 2>&1 | grep -o 'ses_[a-zA-Z0-9]*' | head -1)
echo "Session: $SESSION"
echo "Test dir: $TEST_DIR"

echo ""
echo "=== Test: Create File ==="
"$CROW_CLI" chat --session "$SESSION" "Create a file named hello.txt with content: Written by Crow"

echo ""
echo "=== File System Check ==="
echo "Files in test dir:"
ls -la "$TEST_DIR"

echo ""
echo "File content:"
cat "$TEST_DIR/hello.txt" 2>/dev/null || echo "(file not found)"

echo ""
echo "=== Snapshot Check ==="
echo "Recent snapshots:"
ls -lt ~/.local/share/crow/snapshots/ 2>/dev/null | head -5

echo ""
echo "=== AGENT: Verify ==="
echo "1. hello.txt exists in test dir"
echo "2. Content is 'Written by Crow'"
echo "3. Agent used Write tool (not bash echo redirect)"
echo "4. New snapshot created for this project"

rm -rf "$TEST_DIR"
