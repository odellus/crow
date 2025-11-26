#!/usr/bin/env bash
# TEST: Grep Tool
# WHAT: Agent searches for patterns in files
# AGENT CHECK: Correct files found, used Grep tool not bash

set -euo pipefail
CROW_CLI="${CROW_CLI:-crow-cli}"
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"

# Setup test files
echo "ERROR: something failed" > log1.txt
echo "INFO: all systems go" > log2.txt
echo "ERROR: another failure" > log3.txt
echo "DEBUG: testing" > log4.txt

SESSION=$("$CROW_CLI" new "Grep Test" 2>&1 | grep -o 'ses_[a-zA-Z0-9]*' | head -1)
echo "Session: $SESSION"
echo "Test dir: $TEST_DIR"

echo ""
echo "=== Test Files ==="
head -1 *.txt

echo ""
echo "=== Test: Grep for ERROR ==="
"$CROW_CLI" chat --session "$SESSION" "Use grep to find files containing ERROR"

echo ""
echo "=== AGENT: Verify ==="
echo "1. Found log1.txt and log3.txt (both have ERROR)"
echo "2. Did NOT find log2.txt or log4.txt"
echo "3. Used Grep tool (not bash grep)"
echo "4. Tool called once"

rm -rf "$TEST_DIR"
