#!/bin/bash
# Integration test for edit tool - compares Crow and OpenCode behavior

set -e

TEST_DIR="/tmp/crow-edit-test-$$"
OPENCODE_PORT=4096
CROW_PORT=4097

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

cleanup() {
    echo "Cleaning up..."
    pkill -f "opencode serve -p $OPENCODE_PORT" 2>/dev/null || true
    pkill -f "crow-serve --port $CROW_PORT" 2>/dev/null || true
    rm -rf "$TEST_DIR"
}

trap cleanup EXIT

echo "=== Edit Tool Integration Test ==="
echo ""

# Create test directory and file
mkdir -p "$TEST_DIR"
cat > "$TEST_DIR/test.rs" << 'EOF'
fn main() {
    println!("Hello, World!");
}

pub fn hello_world() -> String {
    "Hello, World!".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        assert_eq!(hello_world(), "Hello, World!");
    }
}
EOF

# Create .crow config
mkdir -p "$TEST_DIR/.crow"
cat > "$TEST_DIR/.crow/config.jsonc" << 'EOF'
{
  "model": "moonshotai/kimi-k2-thinking"
}
EOF

echo "Test file created at $TEST_DIR/test.rs"
echo ""

# Start OpenCode server
echo "Starting OpenCode server on port $OPENCODE_PORT..."
cd "$TEST_DIR"
opencode serve -p $OPENCODE_PORT &
OPENCODE_PID=$!
sleep 3

# Start Crow server
echo "Starting Crow server on port $CROW_PORT..."
CROW_BIN="/home/thomas/src/projects/opencode-project/crow/target/release/crow-serve"
if [ ! -f "$CROW_BIN" ]; then
    echo -e "${RED}Error: Crow binary not found at $CROW_BIN${NC}"
    exit 1
fi
"$CROW_BIN" --port $CROW_PORT &
CROW_PID=$!
sleep 2

# Function to test edit tool directly via API
test_edit_tool() {
    local server_name=$1
    local port=$2
    local test_name=$3
    local old_string=$4
    local new_string=$5
    local replace_all=$6

    echo "Testing $server_name: $test_name"

    # Reset test file
    cat > "$TEST_DIR/test.rs" << 'EOF'
fn main() {
    println!("Hello, World!");
}

pub fn hello_world() -> String {
    "Hello, World!".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        assert_eq!(hello_world(), "Hello, World!");
    }
}
EOF

    # Create session
    if [ "$server_name" = "OpenCode" ]; then
        SESSION_ID=$(curl -s "http://localhost:$port/session" -X POST \
            -H "Content-Type: application/json" \
            -d '{}' | jq -r '.id')
    else
        SESSION_ID=$(curl -s "http://localhost:$port/session" -X POST \
            -H "Content-Type: application/json" \
            -d "{\"path\":\"$TEST_DIR\"}" | jq -r '.id')
    fi

    if [ -z "$SESSION_ID" ] || [ "$SESSION_ID" = "null" ]; then
        echo -e "${RED}  Failed to create session${NC}"
        return 1
    fi

    # Build the message asking to edit
    local replace_all_text=""
    if [ "$replace_all" = "true" ]; then
        replace_all_text=" with replaceAll=true"
    fi

    local message="Use the edit tool to change \"$old_string\" to \"$new_string\" in $TEST_DIR/test.rs$replace_all_text. Only use the edit tool, nothing else."

    # Send message
    local response=$(curl -s --max-time 120 "http://localhost:$port/session/$SESSION_ID/message" -X POST \
        -H "Content-Type: application/json" \
        -d "{\"parts\":[{\"type\":\"text\",\"text\":\"$message\"}]}")

    # Check if edit was successful by examining the file
    local file_content=$(cat "$TEST_DIR/test.rs")

    if echo "$file_content" | grep -q "$new_string"; then
        echo -e "${GREEN}  PASS: File contains expected content${NC}"

        # Count occurrences
        local count=$(echo "$file_content" | grep -c "$new_string" || echo 0)
        if [ "$replace_all" = "true" ]; then
            if [ "$count" -eq 3 ]; then
                echo -e "${GREEN}  PASS: All 3 occurrences replaced${NC}"
            else
                echo -e "${YELLOW}  WARN: Expected 3 replacements, found $count${NC}"
            fi
        else
            if [ "$count" -eq 1 ]; then
                echo -e "${GREEN}  PASS: Exactly 1 occurrence replaced${NC}"
            else
                echo -e "${YELLOW}  WARN: Expected 1 replacement, found $count${NC}"
            fi
        fi
        return 0
    else
        echo -e "${RED}  FAIL: File does not contain expected content${NC}"
        echo "  Response: $(echo "$response" | jq -c '.parts[] | select(.tool == "edit") | .state')"
        return 1
    fi
}

echo ""
echo "=== Test 1: Simple single replacement ==="
test_edit_tool "OpenCode" $OPENCODE_PORT "single replacement" 'println!("Hello, World!");' 'println!("Hello, Universe!");' "false"
echo ""
test_edit_tool "Crow" $CROW_PORT "single replacement" 'println!("Hello, World!");' 'println!("Hello, Universe!");' "false"

echo ""
echo "=== Test 2: Replace all occurrences ==="
test_edit_tool "OpenCode" $OPENCODE_PORT "replace all" "Hello, World!" "Hello, Universe!" "true"
echo ""
test_edit_tool "Crow" $CROW_PORT "replace all" "Hello, World!" "Hello, Universe!" "true"

echo ""
echo "=== Test 3: Fuzzy matching with whitespace ==="
test_edit_tool "OpenCode" $OPENCODE_PORT "fuzzy whitespace" '    "Hello, World!".to_string()' '    "Hello, Galaxy!".to_string()' "false"
echo ""
test_edit_tool "Crow" $CROW_PORT "fuzzy whitespace" '    "Hello, World!".to_string()' '    "Hello, Galaxy!".to_string()' "false"

echo ""
echo "=== All tests completed ==="
