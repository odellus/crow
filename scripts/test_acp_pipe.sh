#!/bin/bash
set -e

echo "🚀 Testing Crow ACP Server with proper pipe..."

cd /home/thomas/src/projects/orchestrator-project/crow

# Start server with actual pipe
echo "📤 Starting server..."

# Use process substitution to create a real pipe
(. .venv/bin/activate && crow-acp) < <(cat << 'EOF'
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{"fs":{"readTextFile":true,"writeTextFile":true},"terminal":true},"clientInfo":{"name":"test","version":"1.0"}}}
{"jsonrpc":"2.0","id":2,"method":"session/new","params":{"cwd":"/tmp","mcpServers":[]}}
EOF
) &

SERVER_PID=$!

echo "⏳ Server PID: $SERVER_PID"
sleep 5

# Check if still running
if kill -0 $SERVER_PID 2>/dev/null; then
    echo "✅ Server is running"
    kill $SERVER_PID 2>/dev/null || true
else
    wait $SERVER_PID
    EXIT_CODE=$?
    echo "⚠️  Server exited with code: $EXIT_CODE"
fi

echo "\n🎉 Test complete!"
