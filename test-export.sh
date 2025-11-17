#!/bin/bash
# Test script to verify session export functionality

set -e

API_URL="http://localhost:7070"

echo "=== Testing Crow Session Export ==="
echo ""

# Create a session
echo "1. Creating session..."
SESSION_ID=$(curl -s -X POST "$API_URL/session" \
  -H "Content-Type: application/json" \
  -d '{"directory": "/tmp/crow-test"}' | jq -r '.id')

echo "   Session ID: $SESSION_ID"
echo ""

# Send a message to the build agent
echo "2. Sending message to BUILD agent..."
curl -s -X POST "$API_URL/session/$SESSION_ID/message" \
  -H "Content-Type: application/json" \
  -d '{
    "agent": "build",
    "parts": [{"type": "text", "text": "List files in the current directory"}]
  }' > /dev/null

echo "   Message sent"
echo ""

# Wait a moment for export to complete
sleep 2

# Check if export file exists
EXPORT_FILE="/tmp/crow-test/.crow/sessions/$SESSION_ID.md"
echo "3. Checking export file..."
if [ -f "$EXPORT_FILE" ]; then
    echo "   ✅ Export file exists: $EXPORT_FILE"
    echo ""
    echo "4. Export file contents:"
    echo "   ================================"
    head -30 "$EXPORT_FILE" | sed 's/^/   /'
    echo "   ================================"
    echo ""
    echo "   Total lines: $(wc -l < "$EXPORT_FILE")"
else
    echo "   ❌ Export file not found: $EXPORT_FILE"
    exit 1
fi

echo ""
echo "=== Test completed successfully! ==="
