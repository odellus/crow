#!/bin/bash
set -e

echo "🚀 Testing Crow ACP Server with simple echo commands..."

# Create a test script that sends commands
cat > /tmp/acp_commands.jsonl << 'EOF'
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{"fs":{"readTextFile":true,"writeTextFile":true},"terminal":true},"clientInfo":{"name":"test","version":"1.0"}}}
{"jsonrpc":"2.0","id":2,"method":"session/new","params":{"cwd":"/tmp","mcpServers":[]}}
EOF

echo "📤 Starting server and sending commands..."
cd /home/thomas/src/projects/orchestrator-project/crow

# Start server in background and pipe commands
(. .venv/bin/activate && crow-acp) < /tmp/acp_commands.jsonl &
SERVER_PID=$!

echo "⏳ Waiting for responses..."
sleep 5

# Check if server is still running
if kill -0 $SERVER_PID 2>/dev/null; then
    echo "✅ Server is still running (PID: $SERVER_PID)"
    kill $SERVER_PID 2>/dev/null || true
    echo "🧹 Server stopped"
else
    echo "⚠️  Server exited early"
    wait $SERVER_PID
    EXIT_CODE=$?
    echo "📛 Exit code: $EXIT_CODE"
fi

echo "\n🎉 Test complete!"
