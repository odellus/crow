#!/usr/bin/env bash
# TEST: Dual-Agent Task (Executor + Arbiter)
# WHAT: Agent spawns a dual-agent pair for verified task completion
# AGENT CHECK: Executor implements, Arbiter verifies, task_complete called, both sessions logged

set -euo pipefail
CROW_CLI="${CROW_CLI:-crow-cli}"
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"

SESSION=$("$CROW_CLI" new "Dual Agent Test" 2>&1 | grep -o 'ses_[a-zA-Z0-9]*' | head -1)
echo "Session: $SESSION"
echo "Test dir: $TEST_DIR"

echo ""
echo "=== Test: Dual-Agent Verified Task ==="
echo "Asking agent to spawn dual-agent for a task requiring verification..."
echo ""
"$CROW_CLI" chat --session "$SESSION" "Use the Task tool with subagent_type 'dual' to: Create a simple Python function in calculator.py that adds two numbers, then verify it works by running a test. The arbiter should verify the function exists and runs correctly before calling task_complete."

echo ""
echo "=== File Check ==="
echo "Checking if calculator.py was created..."
if [ -f calculator.py ]; then
    echo "calculator.py exists:"
    cat calculator.py
else
    echo "WARNING: calculator.py not found"
fi

echo ""
echo "=== Dual-Agent Session Check ==="
echo "Sessions stored in: ~/.local/share/crow/storage/session/{projectID}/"
echo ""
echo "Looking for executor and arbiter sessions..."
"$CROW_CLI" sessions 2>&1 | grep -E "(Executor|Arbiter|dual)" | head -10
echo ""
echo "All recent sessions:"
"$CROW_CLI" sessions 2>&1 | head -10
echo ""
echo "Raw session files (created in last 5 min):"
find ~/.local/share/crow/storage/session -name "*.json" -mmin -5 2>/dev/null | head -10

echo ""
echo "=== AGENT: Verify ==="
echo "1. Task tool was invoked with subagent_type 'dual'"
echo "2. TWO sibling sessions created: Executor and Arbiter"
echo "3. Executor created the calculator.py file"
echo "4. Arbiter verified (ran tests, checked file exists)"
echo "5. Arbiter called task_complete to signal completion"
echo "6. Output shows '✅ Task completed and verified'"
echo ""
echo "7. DUAL-AGENT OBSERVABILITY CHECK:"
echo "   Find executor session ID from sessions list, then:"
echo "   - crow-cli session info <executor-session-id>"
echo "   - crow-cli session history <executor-session-id>"
echo "   Should show: write/edit tool calls for creating calculator.py"
echo ""
echo "   Find arbiter session ID from sessions list, then:"
echo "   - crow-cli session info <arbiter-session-id>"
echo "   - crow-cli session history <arbiter-session-id>"
echo "   Should show: read/bash tool calls for verification, task_complete call"
echo ""
echo "8. Check session metadata for dual-agent linking:"
echo "   Session JSON should have metadata.dual_agent with:"
echo "   - role: 'executor' or 'arbiter'"
echo "   - pair_id: same ID for both"
echo "   - sibling_id: points to the other session"
echo ""
echo "RED FLAGS:"
echo "- Agent used single subagent instead of dual"
echo "- Only executor session exists (arbiter missing)"
echo "- Arbiter didn't call task_complete"
echo "- Sessions not linked (missing sibling_id metadata)"
echo "- File not created but task marked complete"
echo "- Executor/Arbiter sessions not appearing in sessions list"

echo ""
echo "=== Cleanup ==="
echo "Test directory: $TEST_DIR"
echo "Run 'rm -rf $TEST_DIR' to clean up"
