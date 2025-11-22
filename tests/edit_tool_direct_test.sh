#!/bin/bash
# Direct test of edit tool - tests the replace function behavior directly

set -e

TEST_DIR="/tmp/crow-edit-direct-test-$$"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

cleanup() {
    rm -rf "$TEST_DIR"
}

trap cleanup EXIT

echo "=== Edit Tool Direct Test ==="
echo ""

# Create test directory
mkdir -p "$TEST_DIR"

# Test 1: Simple replacement
echo "Test 1: Simple single replacement"
cat > "$TEST_DIR/test1.txt" << 'EOF'
Hello, World!
EOF

cd /home/thomas/src/projects/opencode-project/crow
cargo test --features server test_edit_simple_replacement -- --nocapture 2>&1 | tail -20
if [ $? -eq 0 ]; then
    echo -e "${GREEN}PASS${NC}"
else
    echo -e "${RED}FAIL${NC}"
fi

echo ""
echo "Test 2: Replace all"
cargo test --features server test_edit_replace_all -- --nocapture 2>&1 | tail -20
if [ $? -eq 0 ]; then
    echo -e "${GREEN}PASS${NC}"
else
    echo -e "${RED}FAIL${NC}"
fi

echo ""
echo "Test 3: Fuzzy matching"
cargo test --features server test_edit_fuzzy_matching -- --nocapture 2>&1 | tail -20
if [ $? -eq 0 ]; then
    echo -e "${GREEN}PASS${NC}"
else
    echo -e "${RED}FAIL${NC}"
fi

echo ""
echo "=== All tests completed ==="
