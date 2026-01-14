#!/bin/bash
# Quick-start script for ACP development job

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}ACP Development Job for Crow${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Check if we're in the right directory
if [ ! -f "ACP_DEVELOPMENT_PLAN.md" ]; then
    echo -e "${RED}Error: ACP_DEVELOPMENT_PLAN.md not found${NC}"
    echo "Please run this script from the crow/ directory"
    exit 1
fi

# Check environment variables
echo -e "${YELLOW}Checking environment...${NC}"

if [ -z "$LLM_API_KEY" ]; then
    echo -e "${RED}Error: LLM_API_KEY not set${NC}"
    echo "Please set LLM_API_KEY environment variable"
    exit 1
fi

if [ -z "$ZAI_API_KEY" ]; then
    echo -e "${YELLOW}Warning: ZAI_API_KEY not set${NC}"
    echo "Vision testing features will be limited"
fi

echo -e "${GREEN}✓ Environment check passed${NC}"
echo ""

# Parse command line arguments
PLAN_FILE="ACP_DEVELOPMENT_PLAN.md"
WORKSPACE_DIR="$(pwd)"
QUALITY_THRESHOLD=90
MAX_ITERATIONS=5
NO_DOCUMENTATION=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --plan-file)
            PLAN_FILE="$2"
            shift 2
            ;;
        --workspace-dir)
            WORKSPACE_DIR="$2"
            shift 2
            ;;
        --quality-threshold)
            QUALITY_THRESHOLD="$2"
            shift 2
            ;;
        --max-iterations)
            MAX_ITERATIONS="$2"
            shift 2
            ;;
        --no-documentation)
            NO_DOCUMENTATION="--no-documentation"
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --plan-file PATH              Plan file to execute (default: ACP_DEVELOPMENT_PLAN.md)"
            echo "  --workspace-dir PATH          Workspace directory (default: current directory)"
            echo "  --quality-threshold FLOAT     Quality threshold % (default: 90)"
            echo "  --max-iterations INT          Max refinement iterations (default: 5)"
            echo "  --no-documentation            Skip documentation phase"
            echo "  --help                        Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Show configuration
echo -e "${YELLOW}Configuration:${NC}"
echo "  Plan file: $PLAN_FILE"
echo "  Workspace: $WORKSPACE_DIR"
echo "  Quality threshold: $QUALITY_THRESHOLD%"
echo "  Max iterations: $MAX_ITERATIONS"
echo "  Documentation: $([ -z "$NO_DOCUMENTATION" ] && echo "enabled" || echo "disabled")"
echo ""

# Ask for confirmation
echo -e "${YELLOW}Ready to start ACP development job?${NC}"
echo "Press Enter to continue or Ctrl+C to cancel..."
read

# Change to parent directory (where universal_refinement.py lives)
cd "$(dirname "$WORKSPACE_DIR")"

# Build command
CMD=".venv/bin/python crow/universal_refinement.py \
  --plan-file crow/$PLAN_FILE \
  --workspace-dir $WORKSPACE_DIR \
  --quality-threshold $QUALITY_THRESHOLD \
  --max-iterations $MAX_ITERATIONS"

if [ -n "$NO_DOCUMENTATION" ]; then
    CMD="$CMD $NO_DOCUMENTATION"
fi

# Run the job
echo ""
echo -e "${GREEN}Starting ACP development job...${NC}"
echo ""
eval $CMD

# Show results
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Job completed!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Check the following files:"
echo "  - critique_report.md (latest critique)"
echo "  - PROJECT_SUMMARY.md (final documentation)"
echo "  - PHASE_*.md (individual phase reports)"
