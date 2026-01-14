#!/bin/bash
# Run ACP development job with proper virtual environment prompts

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}ACP Development Job${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Check if we're in the right directory
if [ ! -f "ACP_PROMPTS.md" ]; then
    echo "Error: ACP_PROMPTS.md not found"
    echo "Please run this script from the crow/ directory"
    exit 1
fi

# Default values
PLAN_FILE="${PLAN_FILE:-ACP_PROMPTS.md}"
WORKSPACE_DIR="$(pwd)"
QUALITY_THRESHOLD="${QUALITY_THRESHOLD:-90}"
MAX_ITERATIONS="${MAX_ITERATIONS:-5}"

# Show config
echo -e "${YELLOW}Configuration:${NC}"
echo "  Plan file: $PLAN_FILE"
echo "  Workspace: $WORKSPACE_DIR"
echo "  Quality threshold: $QUALITY_THRESHOLD%"
echo "  Max iterations: $MAX_ITERATIONS"
echo ""

# Read prompts
PLANNING_PROMPT="$(cat prompts/acp_planning_prompt.txt)"
IMPLEMENTATION_PROMPT="$(cat prompts/acp_implementation_prompt.txt)"
CRITIC_PROMPT="$(cat prompts/acp_critic_prompt.txt)"

# Change to parent directory (where universal_refinement.py lives)
cd "$(dirname "$WORKSPACE_DIR")"

# Run with custom prompts
python crow/universal_refinement.py \
  --plan-file "crow/$PLAN_FILE" \
  --workspace-dir "$WORKSPACE_DIR" \
  --quality-threshold "$QUALITY_THRESHOLD" \
  --max-iterations "$MAX_ITERATIONS" \
  --planning-prompt "$PLANNING_PROMPT" \
  --implementation-prompt "$IMPLEMENTATION_PROMPT" \
  --critic-prompt "$CRITIC_PROMPT"

echo ""
echo -e "${GREEN}Job complete!${NC}"
echo "Check critique_report.md for results"
