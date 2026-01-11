#!/usr/bin/env python3
"""
Universal Iterative Refinement: Generalized Planning-Execution-Critique Workflow

This script implements a generalized iterative refinement workflow where:
1. A planning agent reads a PLAN.md file and creates a structured task list
2. An implementation agent executes tasks from the plan
3. A critic agent (with playwright + zai-vision tools!) tests and scores each iteration
4. A documentation agent documents everything and cleans up when done
5. The workflow loops until quality threshold is met or max iterations reached

Usage:
    # Basic usage with default plan
    python universal_refinement.py

    # Specify custom plan file
    python universal_refinement.py --plan-file MY_PLAN.md

    # Override quality threshold
    python universal_refinement.py --quality-threshold 85

    # Custom workspace
    python universal_refinement.py --workspace-dir /path/to/project

    # Skip documentation phase
    python universal_refinement.py --no-documentation

Environment Variables:
    ZAI_API_KEY=your_key
    ZAI_BASE_URL=https://your-base-url  # optional
    LLM_MODEL=anthropic/claude-sonnet-4-5-20250929  # optional
"""

import argparse
import os
import re
import shutil
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


# Default prompts (can be overridden via CLI or config files)

DEFAULT_PLANNING_PROMPT = """You are a PLANNING AGENT. Your job is to read the plan at {plan_file} and create a structured task list.

Read the plan file and create a task list using the task_tracker tool. The task list should:
1. Break down the plan into actionable tasks
2. Include proper phases if applicable
3. Add notes for each task explaining what needs to be done
4. Mark all tasks as "todo" initially

Use the task_tracker tool with the "plan" command to create the task list.

After creating the task list, report back with a summary of the tasks created.
"""

DEFAULT_IMPLEMENTATION_PROMPT = """You are an IMPLEMENTATION AGENT. Your job is to execute tasks from the plan.

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

DEFAULT_CRITIC_PROMPT = """You are a CRITIC AGENT with advanced testing capabilities. Your job is to evaluate the quality of work for {phase}.

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
[PASS if score >= {quality_threshold}, NEEDS_IMPROVEMENT otherwise]

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

DEFAULT_DOCUMENTATION_PROMPT = """You are a DOCUMENTATION AGENT. Your job is to document the completed work and clean up temporary files.

**Workspace**: {workspace_dir}

**Instructions**:
1. Review all completed work in the workspace
2. Create comprehensive documentation in {workspace_dir}/PROJECT_SUMMARY.md
3. Consolidate all PHASE_*.md reports into a single summary
4. Archive or remove individual phase report files
5. Document:
   - What was built
   - How it works
   - How to use it
   - Any important decisions or trade-offs
   - Next steps or future work

**Output Format**:

```markdown
# Project Summary

## Overview
[Brief description of what was accomplished]

## What Was Built
[List of components, files, features created]

## How It Works
[Architecture, design decisions, key implementation details]

## How To Use It
[Usage instructions, examples, commands]

## Key Decisions
[Important architectural or implementation decisions]

## Next Steps
[Future work, improvements, follow-up tasks]

## Iteration History
{consolidated_phase_reports}
```

After documenting, clean up:
- Remove individual PHASE_*.md files
- Remove temporary critique reports
- Archive any other intermediate files

Begin documentation and cleanup now.
"""


def create_llm(usage_id: str) -> LLM:
    """Create LLM with ZAI configuration."""
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


def parse_critique_score(critique_file: Path) -> float:
    """Parse the overall score from the critique report."""
    if not critique_file.exists():
        return 0.0

    content = critique_file.read_text()

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


def run_universal_refinement(
    plan_file: Path,
    workspace_dir: Path,
    quality_threshold: float = 90.0,
    max_iterations: int = 5,
    planning_prompt: str | None = None,
    implementation_prompt: str | None = None,
    critic_prompt: str | None = None,
    documentation_prompt: str | None = None,
    skip_documentation: bool = False,
) -> None:
    """Run the universal iterative refinement workflow."""

    critique_file = workspace_dir / "critique_report.md"

    print("=" * 80)
    print("UNIVERSAL ITERATIVE REFINEMENT")
    print("=" * 80)
    print(f"Plan File: {plan_file}")
    print(f"Workspace: {workspace_dir}")
    print(f"Quality Threshold: {quality_threshold}%")
    print(f"Max Iterations: {max_iterations}")
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

    prompt = planning_prompt or DEFAULT_PLANNING_PROMPT
    planning_conversation.send_message(prompt.format(plan_file=plan_file))
    planning_conversation.run()

    print("\nPlanning phase complete.")
    print()

    # Phase 2: Iterative Implementation & Critique
    current_score = 0.0
    iteration = 0

    while current_score < quality_threshold and iteration < max_iterations:
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

        impl_prompt = implementation_prompt or DEFAULT_IMPLEMENTATION_PROMPT
        impl_conversation.send_message(
            f"""You are the IMPLEMENTATION AGENT for iteration {iteration}.

**Previous Critique** (if iteration > 1):
{critique_file.read_text() if critique_file.exists() and iteration > 1 else "None - this is the first iteration"}

**Instructions**:
1. Use task_tracker to view the current task list
2. Execute tasks that are marked as "todo"
3. Update task status as you work (todo → in_progress → done)
4. Address any issues from the previous critique
5. Test your work before marking tasks as done

Execute the tasks now. Report back when complete.
"""
        )
        implementation_conversation.run()

        print("\nImplementation phase complete.")

        # Sub-phase 2b: Critique
        print("\n--- Sub-phase: Critique ---")

        critic_agent = create_agent(
            with_mcp=True, usage_id=f"critic_agent_iter{iteration}"
        )
        critic_conversation = Conversation(
            agent=critic_agent,
            workspace=str(workspace_dir),
        )

        crit_prompt_template = critic_prompt or DEFAULT_CRITIC_PROMPT
        previous_critique = critique_file if iteration > 1 else None
        
        # Add previous critique context if exists
        if previous_critique and previous_critique.exists():
            crit_prompt_template += f"""

**PREVIOUS CRITIQUE**: A previous iteration was evaluated. Please review:
{previous_critique.read_text()}

Focus your evaluation on whether the issues from the previous critique were addressed.
"""

        critic_conversation.send_message(
            crit_prompt_template.format(
                phase=f"Iteration {iteration}",
                workspace_dir=workspace_dir,
                quality_threshold=quality_threshold,
            )
        )
        critic_conversation.run()

        print("\nCritic phase complete.")

        # Parse the score
        current_score = parse_critique_score(critique_file)
        print(f"\nCurrent Score: {current_score:.1f}%")

        if current_score >= quality_threshold:
            print(f"\n✓ Quality threshold ({quality_threshold}%) met!")
        else:
            print(
                f"\n✗ Score below threshold ({quality_threshold}%). "
                "Continuing refinement..."
            )

    # Phase 3: Documentation & Cleanup
    if not skip_documentation:
        print("\n" + "=" * 80)
        print("PHASE 3: DOCUMENTATION & CLEANUP")
        print("=" * 80)

        documentation_agent = create_agent(
            with_mcp=False, usage_id="documentation_agent"
        )
        documentation_conversation = Conversation(
            agent=documentation_agent,
            workspace=str(workspace_dir),
        )

        doc_prompt = documentation_prompt or DEFAULT_DOCUMENTATION_PROMPT
        documentation_conversation.send_message(
            doc_prompt.format(
                workspace_dir=workspace_dir,
            )
        )
        documentation_conversation.run()

        print("\nDocumentation phase complete.")

    # Final summary
    print("\n" + "=" * 80)
    print("UNIVERSAL ITERATIVE REFINEMENT COMPLETE")
    print("=" * 80)
    print(f"Total iterations: {iteration}")
    print(f"Final score: {current_score:.1f}%")
    print(f"Workspace: {workspace_dir}")

    # Report cost
    planning_llm = create_llm("planning")
    cost = planning_llm.metrics.accumulated_cost
    print(f"\nTotal Cost: ${cost:.4f}")

    if current_score >= quality_threshold:
        print("\n✓ SUCCESS: Quality threshold met!")
    else:
        print(
            f"\n✗ MAX ITERATIONS REACHED: Final score {current_score:.1f}% below threshold {quality_threshold}%"
        )


def main():
    """CLI entry point."""
    parser = argparse.ArgumentParser(
        description="Universal Iterative Refinement - Generalized planning-execution-critique workflow"
    )
    
    parser.add_argument(
        "--plan-file",
        type=Path,
        default=Path("CROW_AGENT_PLAN.md"),
        help="Path to plan file (default: CROW_AGENT_PLAN.md)",
    )
    
    parser.add_argument(
        "--workspace-dir",
        type=Path,
        default=Path.cwd(),
        help="Workspace directory (default: current directory)",
    )
    
    parser.add_argument(
        "--quality-threshold",
        type=float,
        default=90.0,
        help="Quality threshold percentage (default: 90.0)",
    )
    
    parser.add_argument(
        "--max-iterations",
        type=int,
        default=5,
        help="Maximum refinement iterations (default: 5)",
    )
    
    parser.add_argument(
        "--planning-prompt",
        type=str,
        help="Custom planning prompt (overrides default)",
    )
    
    parser.add_argument(
        "--implementation-prompt",
        type=str,
        help="Custom implementation prompt (overrides default)",
    )
    
    parser.add_argument(
        "--critic-prompt",
        type=str,
        help="Custom critic prompt (overrides default)",
    )
    
    parser.add_argument(
        "--documentation-prompt",
        type=str,
        help="Custom documentation prompt (overrides default)",
    )
    
    parser.add_argument(
        "--no-documentation",
        action="store_true",
        help="Skip documentation phase",
    )
    
    args = parser.parse_args()
    
    # Validate plan file exists
    if not args.plan_file.exists():
        print(f"ERROR: Plan file not found: {args.plan_file}")
        print(f"Current directory: {Path.cwd()}")
        print(f"Please provide a valid plan file with --plan-file")
        return 1
    
    # Validate workspace directory
    if not args.workspace_dir.exists():
        print(f"ERROR: Workspace directory not found: {args.workspace_dir}")
        return 1
    
    # Run the refinement workflow
    run_universal_refinement(
        plan_file=args.plan_file,
        workspace_dir=args.workspace_dir,
        quality_threshold=args.quality_threshold,
        max_iterations=args.max_iterations,
        planning_prompt=args.planning_prompt,
        implementation_prompt=args.implementation_prompt,
        critic_prompt=args.critic_prompt,
        documentation_prompt=args.documentation_prompt,
        skip_documentation=args.no_documentation,
    )
    
    return 0


if __name__ == "__main__":
    exit(main())
