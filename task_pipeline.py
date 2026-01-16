#!/usr/bin/env python3
"""
Task Splitter: Split a large PLAN.md into smaller tasks

This reads a big PLAN.md and uses an LLM to split it into smaller,
well-scoped tasks with clear acceptance criteria.

Usage:
    python task_splitter.py --plan-file PLAN.md --output-dir tasks/
"""

import argparse
import json
from pathlib import Path
from typing import Any
from datetime import datetime

from dotenv import load_dotenv

load_dotenv()

from openhands.sdk import LLM, Agent, Conversation, Tool
from openhands.tools.file_editor import FileEditorTool
from openhands.tools.terminal import TerminalTool


def split_plan_into_tasks(
    plan_file: Path,
    output_dir: Path,
    llm: LLM,
) -> list[dict[str, Any]]:
    """
    Split a large PLAN.md into smaller tasks with acceptance criteria.
    
    Args:
        plan_file: Path to the big PLAN.md
        output_dir: Directory to write task_N.md files
        llm: LLM instance to use for splitting
    
    Returns:
        List of task metadata (id, title, file, etc.)
    """
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Read the big plan
    plan_content = plan_file.read_text()
    
    # Create splitter agent
    splitter_agent = Agent(
        llm=llm,
        tools=[
            Tool(name=FileEditorTool.name),
            Tool(name=TerminalTool.name),
        ],
    )
    
    splitter_conversation = Conversation(
        agent=splitter_agent,
        workspace=str(output_dir),
    )
    
    # Prompt to split the plan
    split_prompt = f"""# Task Splitter Agent

You are a TASK SPLITTER. Your job is to read a large PLAN.md file and split it into smaller, well-scoped tasks.

**Input Plan**: {plan_file}

**Your Job**:
1. Read the plan file carefully
2. Split it into 5-15 smaller tasks (each should take 1-3 hours to complete)
3. Each task must have CLEAR ACCEPTANCE CRITERIA (how do we know it's done?)
4. Write each task to a separate file: task_001.md, task_002.md, etc.
5. Create a task_manifest.json with metadata about all tasks

**Task Format** (for each task_N.md):
```markdown
# Task N: [Title]

## Description
[Clear description of what this task accomplishes]

## Acceptance Criteria
1. [Specific, testable criterion 1]
2. [Specific, testable criterion 2]
3. [Specific, testable criterion 3]
...

## Dependencies
- List any other tasks that must complete first (by task number)

## Implementation Notes
[Any helpful context for the implementation agent]
```

**Important**:
- Tasks should be INDEPENDENT (minimize dependencies)
- Tasks should be TESTABLE (clear acceptance criteria)
- Tasks should be SEQUENTIAL (logical order)
- Each task should be 1-3 hours of work
- Acceptance criteria must be SPECIFIC and VERIFIABLE

**Process**:
1. Use file_editor to read {plan_file}
2. Analyze the plan and identify logical task boundaries
3. Create task_001.md, task_002.md, etc. in {output_dir}
4. Create task_manifest.json with metadata

Begin splitting the plan now.
"""
    
    splitter_conversation.send_message(split_prompt)
    splitter_conversation.run()
    
    # Load the manifest
    manifest_file = output_dir / "task_manifest.json"
    if not manifest_file.exists():
        raise RuntimeError("Splitter agent didn't create task_manifest.json")
    
    with open(manifest_file) as f:
        manifest = json.load(f)
    
    tasks = manifest.get("tasks", [])
    print(f"\n✅ Split plan into {len(tasks)} tasks")
    
    return tasks


class TaskPipeline:
    """Execute a pipeline of tasks through iterative refinement."""
    
    def __init__(self, tasks_dir: Path, workspace_dir: Path):
        self.tasks_dir = tasks_dir
        self.workspace_dir = workspace_dir
        self.tasks: list[dict[str, Any]] = []
        self.completed_tasks: dict[str, Any] = {}
        self.failed_tasks: dict[str, str] = {}
        self.results_file = workspace_dir / "task_pipeline_results.json"
        
    def load_tasks(self):
        """Load task manifest from tasks directory."""
        manifest_file = self.tasks_dir / "task_manifest.json"
        
        if not manifest_file.exists():
            raise RuntimeError(f"No task_manifest.json found in {self.tasks_dir}")
        
        with open(manifest_file) as f:
            manifest = json.load(f)
        
        self.tasks = manifest.get("tasks", [])
        print(f"Loaded {len(self.tasks)} tasks from {manifest_file}")
        
    def run_task(self, task: dict[str, Any]) -> dict[str, Any]:
        """Run a single task through iterative refinement."""
        task_id = task["id"]
        task_file = self.tasks_dir / task["file"]
        
        print(f"\n{'='*80}")
        print(f"TASK {task_id}: {task['title']}")
        print(f"File: {task_file}")
        print(f"Priority: {task.get('priority', 'medium')}")
        print(f"{'='*80}\n")
        
        # Check dependencies
        dependencies = task.get("dependencies", [])
        for dep_id in dependencies:
            if dep_id not in self.completed_tasks:
                error = f"Task {task_id} depends on {dep_id} which is not complete"
                print(f"❌ {error}")
                self.failed_tasks[task_id] = error
                return {"status": "failed", "error": error}
        
        # Import here to avoid circular dependency
        from iterative_refinement import run_iterative_refinement
        
        # Create task-specific output directory
        task_output_dir = self.workspace_dir / f"task_{task_id}"
        task_output_dir.mkdir(exist_ok=True)
        
        # Run iterative refinement for this task
        try:
            result = run_iterative_refinement(
                plan_file=task_file,
                workspace_dir=task_output_dir,
                quality_threshold=task.get("quality_threshold", 90.0),
                max_iterations=task.get("max_iterations", 3),
                skip_documentation=True,
            )
            
            # Record result
            task_result = {
                "task_id": task_id,
                "task_title": task["title"],
                "status": "completed" if result.get("success") else "failed",
                "final_score": result.get("final_score", 0),
                "iterations": result.get("iterations", 0),
                "output_dir": str(task_output_dir),
                "completed_at": datetime.now().isoformat(),
            }
            
            if result.get("success"):
                print(f"✅ Task {task_id} completed successfully!")
                self.completed_tasks[task_id] = task_result
            else:
                print(f"❌ Task {task_id} failed after {result.get('iterations', 0)} iterations")
                self.failed_tasks[task_id] = task_result
            
            return task_result
            
        except Exception as e:
            error = f"Exception running task {task_id}: {e}"
            print(f"❌ {error}")
            task_result = {
                "task_id": task_id,
                "task_title": task["title"],
                "status": "error",
                "error": str(e),
                "completed_at": datetime.now().isoformat(),
            }
            self.failed_tasks[task_id] = error
            return task_result
    
    def run_pipeline(self, start_at: int = 1):
        """Run all tasks in the pipeline."""
        print(f"\n{'='*80}")
        print("TASK PIPELINE STARTING")
        print(f"Total tasks: {len(self.tasks)}")
        print(f"Workspace: {self.workspace_dir}")
        print(f"{'='*80}\n")
        
        # Load previous results if exists
        if self.results_file.exists():
            with open(self.results_file) as f:
                previous_results = json.load(f)
                self.completed_tasks = previous_results.get("completed", {})
                self.failed_tasks = previous_results.get("failed", {})
            print(f"Loaded previous results: {len(self.completed_tasks)} completed, {len(self.failed_tasks)} failed")
        
        # Run tasks
        for i in range(start_at - 1, len(self.tasks)):
            task = self.tasks[i]
            task_id = task["id"]
            
            # Skip if already completed
            if task_id in self.completed_tasks:
                print(f"\n⏭️  Skipping task {task_id} (already completed)")
                continue
            
            # Run the task
            result = self.run_task(task)
            
            # Save results after each task
            self.save_results()
            
            # Stop if task failed and it's critical
            if result.get("status") != "completed" and task.get("priority") == "critical":
                print(f"\n🛑 Critical task {task_id} failed, stopping pipeline")
                break
        
        # Final summary
        self.print_summary()
    
    def save_results(self):
        """Save pipeline results to JSON file."""
        results = {
            "completed": self.completed_tasks,
            "failed": self.failed_tasks,
            "updated_at": datetime.now().isoformat(),
        }
        
        with open(self.results_file, "w") as f:
            json.dump(results, f, indent=2)
    
    def print_summary(self):
        """Print final pipeline summary."""
        print(f"\n{'='*80}")
        print("TASK PIPELINE SUMMARY")
        print(f"{'='*80}\n")
        
        print(f"Total tasks: {len(self.tasks)}")
        print(f"✅ Completed: {len(self.completed_tasks)}")
        print(f"❌ Failed: {len(self.failed_tasks)}")
        print(f"⏳ Remaining: {len(self.tasks) - len(self.completed_tasks) - len(self.failed_tasks)}")
        
        if self.completed_tasks:
            print("\n✅ Completed Tasks:")
            for task_id, result in self.completed_tasks.items():
                print(f"  {task_id}: {result['task_title']} (score: {result.get('final_score', 'N/A')})")
        
        if self.failed_tasks:
            print("\n❌ Failed Tasks:")
            for task_id, error in self.failed_tasks.items():
                print(f"  {task_id}: {error}")
        
        print(f"\nResults saved to: {self.results_file}")


def main():
    parser = argparse.ArgumentParser(
        description="Split a large plan into tasks and run through iterative refinement"
    )
    parser.add_argument(
        "--plan-file",
        type=Path,
        required=True,
        help="Path to the large PLAN.md file to split",
    )
    parser.add_argument(
        "--tasks-dir",
        type=Path,
        default=Path("tasks"),
        help="Directory to write task files (default: tasks/)",
    )
    parser.add_argument(
        "--workspace-dir",
        type=Path,
        default=Path.cwd(),
        help="Workspace directory (default: current directory)",
    )
    parser.add_argument(
        "--start-at",
        type=int,
        default=1,
        help="Start at task number N (1-indexed)",
    )
    parser.add_argument(
        "--skip-split",
        action="store_true",
        help="Skip splitting phase if tasks already exist",
    )
    
    args = parser.parse_args()
    
    # Create LLM
    llm = LLM(
        model=os.getenv("LLM_MODEL", "anthropic/claude-sonnet-4-5-20250929"),
        api_key=os.getenv("ZAI_API_KEY"),
        base_url=os.getenv("ZAI_BASE_URL"),
    )
    
    # Phase 1: Split the plan into tasks (unless skipped)
    if not args.skip_split:
        print("\n" + "="*80)
        print("PHASE 1: SPLITTING PLAN INTO TASKS")
        print("="*80 + "\n")
        
        tasks = split_plan_into_tasks(
            plan_file=args.plan_file,
            output_dir=args.tasks_dir,
            llm=llm,
        )
    
    # Phase 2: Run the task pipeline
    print("\n" + "="*80)
    print("PHASE 2: RUNNING TASK PIPELINE")
    print("="*80 + "\n")
    
    pipeline = TaskPipeline(args.tasks_dir, args.workspace_dir)
    pipeline.load_tasks()
    pipeline.run_pipeline(start_at=args.start_at)


if __name__ == "__main__":
    import os
    main()
