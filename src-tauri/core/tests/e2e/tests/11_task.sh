#!/usr/bin/env bash
# TEST: Task Tool (Subagent Spawning)
# WHAT: Agent spawns a subagent to do deep research
# AGENT CHECK: Subagent executes, does research, parent synthesizes, subagent session logged

set -euo pipefail
CROW_CLI="${CROW_CLI:-crow-cli}"
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"

SESSION=$("$CROW_CLI" new "Task Research Test" 2>&1 | grep -o 'ses_[a-zA-Z0-9]*' | head -1)
echo "Session: $SESSION"
echo "Test dir: $TEST_DIR"

echo ""
echo "=== Test: Deep Research Task ==="
echo "Asking agent to spawn subagent for research that requires web search..."
"$CROW_CLI" chat --session "$SESSION" "Use the Task tool to research: What are the latest developments in Rust async runtimes in 2024-2025? Compare tokio, async-std, and smol. I need a subagent to do deep research on this."

echo ""
echo "=== Subagent Session Check ==="
echo "Sessions stored in: ~/.local/share/crow/storage/session/{projectID}/"
echo "Looking for subagent sessions (should have 'Subagent:' in title)..."
"$CROW_CLI" sessions 2>&1 | grep -i "subagent" | head -5
echo ""
echo "Raw session files:"
find ~/.local/share/crow/storage/session -name "*.json" -mmin -5 2>/dev/null | head -5

echo ""
echo "=== AGENT: Verify ==="
echo "1. Task tool was invoked (look for 'task [general]' in output)"
echo "2. Subagent used web search to find current info (websearch tool calls)"
echo "3. Parent received and synthesized research results"
echo "4. Response compares the three runtimes with 2024-2025 info"
echo ""
echo "5. SUBAGENT OBSERVABILITY CHECK:"
echo "   - Subagent session appears in 'crow-cli sessions' with 'Subagent:' prefix"
echo "   - Run: crow-cli session history <subagent-session-id>"
echo "   - Should show all tool calls (websearch, webfetch, etc.)"
echo "   - Run: crow-cli session info <subagent-session-id>"
echo "   - Should show tool call count, token usage"
echo ""
echo "RED FLAGS:"
echo "- Agent answered from memory instead of spawning Task"
echo "- Task spawned but no web search performed"
echo "- Results are outdated (pre-2024 info only)"
echo "- Subagent timeout or failure"
echo "- Subagent session NOT appearing in sessions list (observability broken)"

rm -rf "$TEST_DIR"
