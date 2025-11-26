#!/usr/bin/env bash
# TEST: Session Management
# WHAT: Creates a session and verifies it persists
# AGENT CHECK: Session ID returned, appears in list

set -euo pipefail
CROW_CLI="${CROW_CLI:-crow-cli}"
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"

echo "=== Creating Session ==="
OUTPUT=$("$CROW_CLI" new "E2E Session Test" 2>&1)
echo "$OUTPUT"

SESSION=$(echo "$OUTPUT" | grep -o 'ses_[a-zA-Z0-9]*' | head -1)
echo ""
echo "=== Session ID ==="
echo "$SESSION"

echo ""
echo "=== Session List ==="
"$CROW_CLI" sessions 2>&1 | head -10

echo ""
echo "=== AGENT: Verify ==="
echo "1. Session ID was returned: $SESSION"
echo "2. Session appears in list above"
echo "3. Working directory is: $TEST_DIR"

rm -rf "$TEST_DIR"
