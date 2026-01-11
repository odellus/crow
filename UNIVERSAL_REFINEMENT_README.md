# Universal Iterative Refinement

A generalized planning-execution-critique workflow that works with ANY plan file.

## Overview

This script automates the iterative refinement process:
1. **Planning Agent**: Reads a PLAN.md file and creates structured tasks
2. **Implementation Agent**: Executes tasks from the plan
3. **Critic Agent**: Tests and scores each iteration (with playwright + zai-vision!)
4. **Documentation Agent**: Documents everything and cleans up when done

## Features

- ✅ **Works with any plan file** - Just point it at a PLAN.md
- ✅ **Configurable prompts** - Override defaults for any agent
- ✅ **CLI interface** - Full control via command-line args
- ✅ **Quality threshold** - Loop until quality target is met
- ✅ **Vision testing** - Uses playwright + zai-vision for visual testing
- ✅ **Auto-documentation** - Generates project summary and cleans up

## Usage

### Basic Usage

```bash
# Use default plan (CROW_AGENT_PLAN.md)
python universal_refinement.py

# Specify custom plan
python universal_refinement.py --plan-file GRAND_ORCHESTRATOR_VISION.md

# Custom workspace
python universal_refinement.py --plan-file MY_PLAN.md --workspace-dir /path/to/project
```

### Advanced Options

```bash
# Override quality threshold
python universal_refinement.py --quality-threshold 85

# Limit iterations
python universal_refinement.py --max-iterations 3

# Skip documentation phase
python universal_refinement.py --no-documentation

# Custom prompts
python universal_refinement.py --planning-prompt "Be extra thorough in planning."
python universal_refinement.py --critic-prompt "Focus on code quality over speed."
```

### All Options

```
--plan-file PATH              Plan file to execute (default: CROW_AGENT_PLAN.md)
--workspace-dir PATH          Workspace directory (default: current directory)
--quality-threshold FLOAT     Quality threshold % (default: 90.0)
--max-iterations INT          Max refinement iterations (default: 5)
--planning-prompt TEXT        Custom planning prompt
--implementation-prompt TEXT Custom implementation prompt
--critic-prompt TEXT          Custom critic prompt
--documentation-prompt TEXT   Custom documentation prompt
--no-documentation            Skip documentation phase
```

## Environment Variables

```bash
# Required
export ZAI_API_KEY=your_key

# Optional
export ZAI_BASE_URL=https://your-base-url
export LLM_MODEL=anthropic/claude-sonnet-4-5-20250929
```

## How It Works

### Phase 1: Planning
1. Reads the plan file
2. Creates structured task list using task_tracker
3. Breaks down plan into actionable tasks

### Phase 2: Iterative Refinement
Loop until quality threshold met:
1. **Implementation**: Execute tasks from plan
2. **Critique**: Test and score the work
3. **Refine**: Address issues from critique

### Phase 3: Documentation
1. Reviews all completed work
2. Creates PROJECT_SUMMARY.md
3. Consolidates PHASE_*.md reports
4. Cleans up temporary files

## Examples

### Example 1: Execute CROW_AGENT_PLAN.md

```bash
cd /home/thomas/src/projects/orchestrator-project
python crow/universal_refinement.py --plan-file CROW_AGENT_PLAN.md
```

### Example 2: Execute Memory Architecture Plan

```bash
python crow/universal_refinement.py --plan-file CROW_MEMORY_ARCHITECTURE.md
```

### Example 3: Custom Quality Threshold

```bash
python crow/universal_refinement.py \
  --plan-file MY_PLAN.md \
  --quality-threshold 80 \
  --max-iterations 3
```

### Example 4: Skip Documentation (Quick Test)

```bash
python crow/universal_refinement.py \
  --plan-file TEST_PLAN.md \
  --no-documentation
```

## Output Files

- **critique_report.md** - Latest critique with score and feedback
- **PROJECT_SUMMARY.md** - Final documentation (if not skipped)
- **PHASE_*.md** - Individual phase reports (consolidated into summary)

## Customization

### Override Prompts

Create a file with your custom prompt:

```bash
# custom_planning_prompt.txt
You are an EXPERT PLANNER. Focus on:
- Breaking down into smallest possible tasks
- Identifying dependencies early
- Estimating complexity
```

Use it:

```bash
python universal_refinement.py \
  --planning-prompt "$(cat custom_planning_prompt.txt)"
```

### Different Quality Thresholds for Different Plans

```bash
# High-stakes production code
python universal_refinement.py --plan-file PRODUCTION.md --quality-threshold 95

# Quick prototype
python universal_refinement.py --plan-file PROTOTYPE.md --quality-threshold 75
```

## Tips

1. **Start with default prompts** - They're well-tested
2. **Use --no-documentation for testing** - Faster iterations
3. **Check critique_report.md** - See what needs improvement
4. **Adjust quality threshold** - Balance between speed and quality
5. **Different plans, different settings** - Tune for each use case

## Related Files

- `crow_iterative_refinement.py` - Original specialized version
- `CROW_AGENT_PLAN.md` - Crow Agent implementation plan
- `GRAND_ORCHESTRATOR_VISION.md` - Overall vision document
- `CROW_MEMORY_ARCHITECTURE.md` - Memory system design
