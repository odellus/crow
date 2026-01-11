# Crow Iterative Refinement

Automated iterative refinement workflow for implementing the Crow Agent plan.

## What It Does

This script implements a **planning → implementation → critique** loop:

1. **Planning Agent**: Reads `CROW_AGENT_PLAN.md` and creates a structured task list
2. **Implementation Agent**: Executes tasks with full tool access (file_editor, terminal, etc.)
3. **Critic Agent**: Tests the work with **playwright + zai-vision** and scores quality
4. **Loop**: Repeats until quality threshold is met or max iterations reached

## The Critic Agent's Superpowers

The critic agent has access to:
- **playwright-mcp**: Browser automation for E2E testing
- **zai-vision-server**: Visual analysis, screenshot testing, OCR
- **file_editor**: Code inspection
- **terminal**: Run tests
- **task_tracker**: Review progress

This means the critic can **actually test** the implementation, not just read code!

## Usage

```bash
# Set your ZAI credentials (same as crow/main.py)
export ZAI_API_KEY=your_api_key
export ZAI_BASE_URL=https://your-base-url  # optional

# Set quality threshold (default: 90)
export QUALITY_THRESHOLD=90

# Set max iterations (default: 5)
export MAX_ITERATIONS=5

# Run the script
cd crow
python crow_iterative_refinement.py
```

## How It Works

### Phase 1: Planning
The planning agent reads `CROW_AGENT_PLAN.md` and creates a task list using the `task_tracker` tool.

### Phase 2: Iterative Refinement
For each iteration:

1. **Implementation Agent**:
   - Views the task list
   - Executes tasks marked as "todo"
   - Updates task status (todo → in_progress → done)
   - Addresses issues from previous critique

2. **Critic Agent**:
   - Uses playwright to test any web/ACP functionality
   - Uses zai-vision to analyze screenshots and verify UI
   - Inspects code for quality
   - Scores the work (0-100)
   - Provides specific feedback

3. **Loop Decision**:
   - If score >= threshold: SUCCESS!
   - If score < threshold and iterations < max: Try again with critique feedback
   - If iterations >= max: Report final score

## Output

The script generates:
- **Workspace**: Temporary directory with all work
- **Critique Report**: Detailed evaluation with scores and issues
- **Final Summary**: Total iterations, final score, cost

## Example Output

```
================================================================================
CROW ITERATIVE REFINEMENT
================================================================================
Quality Threshold: 90.0%
Max Iterations: 5
Plan File: /path/to/CROW_AGENT_PLAN.md

================================================================================
PHASE 1: PLANNING
================================================================================
[Planning agent creates task list...]

Planning phase complete.

================================================================================
ITERATION 1
================================================================================

--- Sub-phase: Implementation ---
[Implementation agent executes tasks...]

Implementation phase complete.

--- Sub-phase: Critique ---
[Critic agent tests and scores...]

Critique phase complete.

Current Score: 75.0%

✗ Score below threshold (90.0%). Continuing refinement...

[... repeats until threshold met or max iterations ...]

================================================================================
ITERATIVE REFINEMENT COMPLETE
================================================================================
Total iterations: 3
Final score: 92.0%
Workspace: /tmp/xyz

Final critique report: /tmp/xyz/critique_report.md

Total Cost: $0.1234

✓ SUCCESS: Quality threshold met!
```

## Configuration

All configuration via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `ZAI_API_KEY` | *required* | Your ZAI API key |
| `ZAI_BASE_URL` | *none* | Custom base URL |
| `LLM_MODEL` | `anthropic/glm-4.7` | Model to use |
| `QUALITY_THRESHOLD` | `90.0` | Quality threshold (0-100) |
| `MAX_ITERATIONS` | `5` | Maximum refinement iterations |

## The Plan

The script implements the plan from `CROW_AGENT_PLAN.md`:

- **Phase 0**: Integrate OpenHands into crow_ide + E2E tests + crow-agent package
- **Phase 1**: ACP server, SDK integration, proper streaming
- **Phase 2**: Terminal, pause/cancel, hooks
- **Phase 3**: Agent generator, planning agent
- **Phase 4**: Testing, optimization, release

## Why This Approach?

Instead of manually executing 13+ tasks, this script:

1. **Automates the workflow**: Planning → implementation → critique → repeat
2. **Actually tests the work**: Critic agent uses playwright + zai-vision
3. **Ensures quality**: Won't stop until threshold is met
4. **Learns from feedback**: Each iteration addresses previous critique
5. **Tracks progress**: Task list shows what's done and what's left

This is the same pattern used in the OpenHands SDK's `31_iterative_refinement.py` example, but adapted for building the Crow Agent ACP implementation.

## License

MIT
