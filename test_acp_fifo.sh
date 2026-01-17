#!/bin/bash
set -e

echo "🚀 Testing Crow ACP Server with named pipe..."

cd /home/thomas/src/projects/orchestrator-project/crow

# Create a named pipe
PIPE="/tmp/acp_pipe_$$"
mkfifo "$PIPE"

# Write commands to pipe in background
cat > "$PIPE" << 'COMMANDS'
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{"fs":{"readTextFile":true,"writeTextFile":true},"terminal":true},"clientInfo":{"name":"test","version":"1.0"}}}
{"jsonrpc":"2.0","id":2,"method":"session/new","params":{"cwd":"/tmp","mcpServers":[]}}
COMMANDS

# Start server reading from pipe
(. .venv/bin/activate && crow-acp) < "$PIPE" &
SERVER_PID=$!

echo "⏳ Server PID: $SERVER_PID"
sleep 3

# Cleanup
rm -f "$PIPE"

if kill -0 $SERVER_PID 2>/dev/null; then
    echo "✅ Server is still running"
    kill $SERVER_PID 2>/dev/null || true
else
    wait $SERVER_PID
    EXIT_CODE=$?
    echo "⚠️  Server exited with code: $EXIT_CODE"
fi

echo "\n🎉 Test complete!"
