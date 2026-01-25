#!/home/thomas/src/projects/orchestrator-project/crow/.venv/bin/python
"""
Crow Iterative Refinement: ACP-First Agent Implementation

This script implements an iterative refinement workflow where:
1. A planning agent reads CROW_AGENT_PLAN.md and creates a structured task list
2. An implementation agent executes each task with proper tools (file_editor, terminal, etc.)
3. A critic agent (with playwright + zai-vision tools!) tests and scores each phase
4. The workflow loops until quality threshold is met or max iterations reached

The critic agent has access to:
- playwright-mcp: Browser automation for E2E testing
- zai-vision-server: Visual analysis and screenshot testing
- All standard tools: file_editor, terminal, task_tracker

Usage:
    export ZAI_API_KEY=your_key
    export ZAI_BASE_URL=https://your-base-url  # optional
    export QUALITY_THRESHOLD=90  # default: 90
    export MAX_ITERATIONS=5  # default: 5

    python crow_iterative_refinement.py
"""

import os
import re
import tempfile
from pathlib import Path
from typing import Any

from dotenv import load_dotenv

# Load environment variables
load_dotenv()

from openhands.sdk import LLM, Agent, Conversation
from openhands.sdk.context.condenser import LLMSummarizingCondenser
from openhands.sdk.tool import Tool
from openhands.tools.file_editor import FileEditorTool
from openhands.tools.task_tracker import TaskTrackerTool
from openhands.tools.terminal import TerminalTool

# Configuration
QUALITY_THRESHOLD = float(os.getenv("QUALITY_THRESHOLD", "90.0"))
MAX_ITERATIONS = int(os.getenv("MAX_ITERATIONS", "5"))
CROW_AGENT_PLAN = Path(__file__).parent.parent / "CROW_AGENT_PLAN.md"


def create_llm(usage_id: str) -> LLM:
    """Create LLM with ZAI configuration (same as crow/main.py)."""
    return LLM(
        model=os.getenv("LLM_MODEL", "anthropic/glm-4.7"),
        api_key=os.getenv("ZAI_API_KEY"),
        base_url=os.getenv("ZAI_BASE_URL", None),
        usage_id=usage_id,
    )


def create_mcp_config() -> dict[str, Any]:
    """Create MCP configuration with playwright and zai-vision."""
    return {
        "mcpServers": {
            "playwright-mcp": {
                "command": "npx",
                "args": ["@playwright/mcp@0.0.47"],
                "env": {},
            },
            "zai-vision-server": {
                "command": "npx",
                "args": ["-y", "@z_ai/mcp-server"],
                "env": {
                    "Z_AI_API_KEY": os.getenv("ZAI_API_KEY"),
                    "Z_AI_MODE": "ZAI",
                },
            },
        }
    }


def create_agent(with_mcp: bool = False, usage_id: str = "agent") -> Agent:
    """Create an agent with standard tools and optional MCP tools."""
    llm = create_llm(usage_id)

    tools = [
        Tool(name=TerminalTool.name),
        Tool(name=FileEditorTool.name),
        Tool(name=TaskTrackerTool.name),
    ]

    # Create condenser with same defaults as OpenHands-CLI
    condenser_llm = create_llm(f"{usage_id}_condenser")
    condenser = LLMSummarizingCondenser(
        llm=condenser_llm,
        max_size=240,
        keep_first=2,
    )

    if with_mcp:
        return Agent(
            llm=llm,
            tools=tools,
            mcp_config=create_mcp_config(),
            condenser=condenser,
        )
    else:
        return Agent(llm=llm, tools=tools, condenser=condenser)


def setup_workspace() -> Path:
    """Create workspace directory for the refinement workflow."""
    # Use orchestrator-project as workspace, not /tmp
    workspace_dir = Path("/home/thomas/src/projects/orchestrator-project")
    print(f"Workspace: {workspace_dir}")
    return workspace_dir


def get_planning_prompt(plan_file: Path) -> str:
    """Generate the prompt for the planning agent."""
    return f"""You are a PLANNING AGENT. Your job is to read the plan at {plan_file} and create a structured task list.

Read the plan file and create a task list using the task_tracker tool. The task list should:
1. Break down the plan into actionable tasks
2. Include proper phases (Phase 0, 1, 2, 3, 4)
3. Add notes for each task explaining what needs to be done
4. Mark all tasks as "todo" initially

Use the task_tracker tool with the "plan" command to create the task list.

After creating the task list, report back with a summary of the tasks created.
"""


def get_implementation_prompt(
    current_task: dict[str, Any], task_number: int, total_tasks: int
) -> str:
    """Generate the prompt for the implementation agent."""
    return f"""You are an IMPLEMENTATION AGENT. Your job is to execute tasks from the plan.

**Current Task ({task_number}/{total_tasks})**: {current_task["title"]}

**Notes**: {current_task.get("notes", "No additional notes")}

**Instructions**:
1. Use the appropriate tools for the task (file_editor, terminal, etc.)
2. Update the task status to "in_progress" when you start
3. Update the task status to "done" when you complete the task
4. If you encounter issues, update the task notes with what went wrong
5. Be thorough - test your work before marking the task as done

**Available Tools**:
- task_tracker: Update task status
- file_editor: View/create/edit files
- terminal: Run commands
- browser: (if needed) Use playwright for browser automation

Execute this task now. When complete, report back with what you did.
"""


def get_critic_prompt(
    phase: str,
    workspace_dir: Path,
    previous_critique: Path | None = None,
) -> str:
    """Generate the prompt for the critic agent."""

    base_prompt = f"""You are a CRITIC AGENT with advanced testing capabilities. Your job is to evaluate the quality of work for {phase}.

**Workspace**: {workspace_dir}

**Your Capabilities**:
- playwright-mcp: Browser automation for E2E testing
- zai-vision-server: Visual analysis, screenshot testing, OCR
- file_editor: Inspect code and configuration files
- terminal: Run tests and commands
- task_tracker: Review task completion status

**Evaluation Criteria** (100 points total):
1. Correctness (25 pts): Does the implementation match the requirements?
2. Code Quality (25 pts): Is the code clean, well-structured, following best practices?
3. Completeness (25 pts): Are all requirements met? Edge cases handled?
4. Testing & Documentation (25 pts): Is it tested? Documented? Ready to use?

**Testing Instructions**:
1. Use playwright-mcp to test any web/ACP functionality
2. Use zai-vision-server to analyze screenshots, verify UI, extract text
3. Run any test commands you find
4. Inspect the code for quality issues
5. Verify ACP protocol compliance if applicable

**Required Output Format** (save to {workspace_dir}/critique_report.md):

```markdown
# {phase} - Critique Report

## Summary
[Brief overall assessment]

## Evaluation

### Correctness: [score]/25
[Explanation with specific examples]

### Code Quality: [score]/25
[Explanation with specific examples]

### Completeness: [score]/25
[Explanation with specific examples]

### Testing & Documentation: [score]/25
[Explanation with specific examples]

## Overall Score: [total]/100

## Recommendation
[PASS if score >= {QUALITY_THRESHOLD}, NEEDS_IMPROVEMENT otherwise]

## Issues Found
1. [Specific issue with file/line reference]
2. [Specific issue with file/line reference]
...

## Priority Improvements
1. [Most critical improvement needed]
2. [Second priority]
3. [Third priority]

## Testing Performed
- [List tests you ran with playwright]
- [List visual checks with zai-vision]
- [List manual inspections]
```

**IMPORTANT**:
- Use your tools! Actually test the implementation, don't just read code.
- Take screenshots with playwright if there's UI/ACP functionality
- Use zai-vision to analyze those screenshots
- Be thorough and specific in your feedback

Begin your evaluation now.
"""

    if previous_critique and previous_critique.exists():
        base_prompt += f"""

**PREVIOUS CRITIQUE**: A previous iteration was evaluated. Please review:
{previous_critique}

Focus your evaluation on whether the issues from the previous critique were addressed.
"""

    return base_prompt


def parse_critique_score(critique_file: Path) -> float:
    """Parse the overall score from the critique report."""
    if not critique_file.exists():
        return 0.0

    content = critique_file.read_text()

    # Look for "Overall Score: X/100" pattern
    patterns = [
        r"## Overall Score:\s*(\d+(?:\.\d+)?)\s*/100",
        r"\*\*Overall Score\*\*:\s*(\d+(?:\.\d+)?)\s*/100",
        r"Overall Score:\s*(\d+(?:\.\d+)?)",
    ]

    for pattern in patterns:
        match = re.search(pattern, content, re.IGNORECASE)
        if match:
            return float(match.group(1))

    return 0.0


def run_iterative_refinement() -> None:
    """Run the iterative refinement workflow."""

    workspace_dir = setup_workspace()
    critique_file = workspace_dir / "critique_report.md"

    print("=" * 80)
    print("CROW ITERATIVE REFINEMENT")
    print("=" * 80)
    print(f"Quality Threshold: {QUALITY_THRESHOLD}%")
    print(f"Max Iterations: {MAX_ITERATIONS}")
    print(f"Plan File: {CROW_AGENT_PLAN}")
    print(f"LLM Model: {os.getenv('LLM_MODEL', 'anthropic/glm-4.7')}")
    print(f"ZAI Base URL: {os.getenv('ZAI_BASE_URL', 'default')}")
    print()

    # Phase 1: Planning
    print("=" * 80)
    print("PHASE 1: PLANNING")
    print("=" * 80)

    planning_agent = create_agent(with_mcp=False, usage_id="planning_agent")
    planning_conversation = Conversation(
        agent=planning_agent,
        workspace=str(workspace_dir),
    )

    planning_prompt = get_planning_prompt(CROW_AGENT_PLAN)
    planning_conversation.send_message(planning_prompt)
    planning_conversation.run()

    print("\nPlanning phase complete.")
    print()

    # Phase 2: Iterative Implementation & Critique
    current_score = 0.0
    iteration = 0

    while current_score < QUALITY_THRESHOLD and iteration < MAX_ITERATIONS:
        iteration += 1

        print("\n" + "=" * 80)
        print(f"ITERATION {iteration}")
        print("=" * 80)

        # Sub-phase 2a: Implementation
        print("\n--- Sub-phase: Implementation ---")

        implementation_agent = create_agent(
            with_mcp=False, usage_id=f"implementation_agent_iter{iteration}"
        )
        implementation_conversation = Conversation(
            agent=implementation_agent,
            workspace=str(workspace_dir),
        )

        implementation_prompt = f"""You are the IMPLEMENTATION AGENT for iteration {iteration}.

Your job is to execute the tasks from the plan.

**Previous Critique** (if iteration > 1):
{critique_file.read_text() if critique_file.exists() and iteration > 1 else "None - this is the first iteration"}

**Instructions**:
1. Use task_tracker to view the current task list
2. Execute tasks that are marked as "todo"
3. Update task status as you work (todo → in_progress → done)
4. Address any issues from the previous critique
5. Test your work before marking tasks as done

**Focus on**: Phase 0 - Integrate OpenHands into crow_ide as default ACP agent

Execute the tasks now. Report back when complete.
"""

        implementation_conversation.send_message(implementation_prompt)
        implementation_conversation.run()

        print("\nImplementation phase complete.")

        # Sub-phase 2b: Critique (WITH MCP TOOLS!)
        print("\n--- Sub-phase: Critique (with playwright + zai-vision) ---")

        critic_agent = create_agent(
            with_mcp=True, usage_id=f"critic_agent_iter{iteration}"
        )
        critic_conversation = Conversation(
            agent=critic_agent,
            workspace=str(workspace_dir),
        )

        previous_critique = critique_file if iteration > 1 else None
        critique_prompt = get_critic_prompt(
            phase=f"Iteration {iteration}",
            workspace_dir=workspace_dir,
            previous_critique=previous_critique,
        )

        critic_conversation.send_message(critique_prompt)
        critic_conversation.run()

        print("\nCritic phase complete.")

        # Parse the score
        current_score = parse_critique_score(critique_file)
        print(f"\nCurrent Score: {current_score:.1f}%")

        if current_score >= QUALITY_THRESHOLD:
            print(f"\n✓ Quality threshold ({QUALITY_THRESHOLD}%) met!")
        else:
            print(
                f"\n✗ Score below threshold ({QUALITY_THRESHOLD}%). "
                "Continuing refinement..."
            )

    # Final summary
    print("\n" + "=" * 80)
    print("ITERATIVE REFINEMENT COMPLETE")
    print("=" * 80)
    print(f"Total iterations: {iteration}")
    print(f"Final score: {current_score:.1f}%")
    print(f"Workspace: {workspace_dir}")

    # Show critique file location
    if critique_file.exists():
        print(f"\nFinal critique report: {critique_file}")
        print("\n--- Critique Summary ---")
        critique_content = critique_file.read_text()
        print(
            critique_content[:500] + "..."
            if len(critique_content) > 500
            else critique_content
        )

    # Report cost from all LLMs
    planning_llm = create_llm("planning")
    cost = planning_llm.metrics.accumulated_cost
    print(f"\nTotal Cost: ${cost:.4f}")

    if current_score >= QUALITY_THRESHOLD:
        print("\n✓ SUCCESS: Quality threshold met!")
    else:
        print(
            f"\n✗ MAX ITERATIONS REACHED: Final score {current_score:.1f}% below threshold {QUALITY_THRESHOLD}%"
        )


if __name__ == "__main__":
    run_iterative_refinement()
