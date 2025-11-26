#!/usr/bin/env bash
# TEST: Multi-Tool Workflow
# WHAT: Agent uses multiple tools in sequence
# AGENT CHECK: Correct tool sequence, synthesizes results

set -euo pipefail
CROW_CLI="${CROW_CLI:-crow-cli}"
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"

SESSION=$("$CROW_CLI" new "Workflow Test" 2>&1 | grep -o 'ses_[a-zA-Z0-9]*' | head -1)
echo "Session: $SESSION"
echo "Test dir: $TEST_DIR"

echo ""
echo "=== Test: Write Then Read ==="
"$CROW_CLI" chat --session "$SESSION" "Create config.json with {\"name\": \"crow\"}, then read it and tell me the name"

echo ""
echo "=== File Check ==="
echo "config.json contents:"
cat config.json 2>/dev/null || echo "(not found)"

echo ""
echo "=== AGENT: Verify ==="
echo "1. config.json created with valid JSON"
echo "2. Agent used Write tool then Read tool"
echo "3. Agent correctly reports name is 'crow'"
echo "4. Agent synthesized info (didn't just dump file)"

rm -rf "$TEST_DIR"
