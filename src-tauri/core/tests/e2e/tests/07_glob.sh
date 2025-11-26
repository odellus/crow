#!/usr/bin/env bash
# TEST: Glob Tool
# WHAT: Agent finds files by pattern
# AGENT CHECK: Correct files found, reasonable tool call count

set -euo pipefail
CROW_CLI="${CROW_CLI:-crow-cli}"
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"

# Setup test structure
mkdir -p src/components src/utils
touch src/app.js src/index.ts
touch src/components/Button.tsx src/components/Input.tsx
touch src/utils/helpers.ts

SESSION=$("$CROW_CLI" new "Glob Test" 2>&1 | grep -o 'ses_[a-zA-Z0-9]*' | head -1)
echo "Session: $SESSION"
echo "Test dir: $TEST_DIR"

echo ""
echo "=== Test Structure ==="
find . -type f | head -20

echo ""
echo "=== Test: Find .tsx Files ==="
"$CROW_CLI" chat --session "$SESSION" "Find all .tsx files"

echo ""
echo "=== AGENT: Verify ==="
echo "1. Found Button.tsx and Input.tsx"
echo "2. Did not include .ts or .js files"
echo "3. RED FLAG: Tool called many times = model confusion"
echo "   (This has been observed - watch the tool call count)"

rm -rf "$TEST_DIR"
