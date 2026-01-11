#!/home/thomas/src/projects/orchestrator-project/crow/.venv/bin/python
"""
Crow Agent Status Check

Quick script to check where we are in the implementation plan.
"""
import os
import json
from pathlib import Path
from dotenv import load_dotenv

load_dotenv()

def get_task_list():
    """Get task list from task_tracker."""
    tasks_file = Path.home() / ".openhands/conversations" / "TASKS.json"
    
    if not tasks_file.exists():
        print("✗ No task list found")
        print(f"  Looking for: {tasks_file}")
        return None
    
    tasks = json.loads(tasks_file.read_text())
    return tasks

def print_status(tasks):
    """Print task status summary."""
    if not tasks:
        return
    
    print("=" * 80)
    print("CROW AGENT STATUS")
    print("=" * 80)
    print()
    
    # Count by status
    todo = sum(1 for t in tasks if t['status'] == 'todo')
    in_progress = sum(1 for t in tasks if t['status'] == 'in_progress')
    done = sum(1 for t in tasks if t['status'] == 'done')
    
    print(f"Total Tasks: {len(tasks)}")
    print(f"  ✓ Done: {done}")
    print(f"  → In Progress: {in_progress}")
    print(f"  ⏳ Todo: {todo}")
    print()
    
    # Show current task (in progress)
    current = [t for t in tasks if t['status'] == 'in_progress']
    if current:
        print("CURRENT TASK:")
        for t in current:
            print(f"  → {t['title']}")
            if t.get('notes'):
                print(f"     {t['notes'][:80]}...")
        print()
    
    # Show next tasks (todo)
    next_tasks = [t for t in tasks if t['status'] == 'todo'][:3]
    if next_tasks:
        print("UP NEXT:")
        for t in next_tasks:
            print(f"  ⏳ {t['title']}")
        print()
    
    # Show completed tasks (last 3)
    completed = [t for t in tasks if t['status'] == 'done'][-3:]
    if completed:
        print("RECENTLY COMPLETED:")
        for t in completed:
            print(f"  ✓ {t['title']}")
        print()
    
    # Show all tasks
    print("ALL TASKS:")
    for i, t in enumerate(tasks, 1):
        status_icon = {
            'todo': '⏳',
            'in_progress': '→',
            'done': '✓'
        }.get(t['status'], '?')
        print(f"  {i:2d}. {status_icon} {t['title']}")

def main():
    tasks = get_task_list()
    if tasks:
        print_status(tasks)

if __name__ == "__main__":
    main()
