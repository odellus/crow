# Refactor Plan: Config-Driven Task Pipeline

## Goal
Refactor `universal_refinement.py` to be config-driven and support running multiple tasks sequentially through iterative refinement.

## Acceptance Criteria

### Phase 1: Config System (Priority: Critical)
1. **Config YAML exists** at `pipeline_config.yaml` with:
   - Prompt file paths (planning, implementation, critic, documentation)
   - LLM settings (model, api_key, base_url)
   - Default values (quality_threshold, max_iterations)
   - Paths (tasks_dir, workspace_dir)
2. **Config loader function** `load_config(config_path: Path) -> dict`
3. **CLI args override config** - Can still pass `--quality-threshold 95` to override YAML
4. **Config validation** - Validates required fields exist
5. **Environment variable support** - Config can use `${ENV_VAR}` syntax

### Phase 2: Refactor run_universal_refinement (Priority: Critical)
1. **Function is importable** - Can do `from universal_refinement import run_universal_refinement`
2. **Takes dict parameter** - `run_universal_refinement(config: dict) -> dict`
3. **Returns results dict** with keys:
   - `success` (bool)
   - `final_score` (float)
   - `iterations` (int)
   - `output_dir` (Path)
   - `error` (str | None)
4. **No global state** - Doesn't rely on argparse args or globals
5. **Clean workspace** - Each run gets its own workspace dir
6. **Testable** - Can unit test the function

### Phase 3: Task Splitter (Priority: High)
1. **Splitter agent** reads PLAN.md and creates:
   - `tasks/task_001.md` with acceptance criteria
   - `tasks/task_002.md` with acceptance criteria
   - ...
   - `tasks/task_manifest.json` with metadata
2. **Task format** includes:
   - Title
   - Description
   - Acceptance criteria (3-7 specific, testable criteria)
   - Dependencies (list of task IDs)
   - Priority (critical/high/medium/low)
3. **Splitter prompt** at `prompts/task_splitter_prompt.txt`
4. **Configurable split** - Can adjust min/max tasks in config
5. **Validation** - Validates each task has acceptance criteria

### Phase 4: Task Pipeline (Priority: High)
1. **Pipeline class** `TaskPipeline` that:
   - Loads task_manifest.json
   - Tracks completed/failed tasks in JSON
   - Can resume from specific task
   - Runs tasks sequentially
2. **Runs run_universal_refinement** on each task:
   - Creates workspace: `task_001/`, `task_002/`, etc.
   - Passes task_N.md as plan_file
   - Uses config from YAML
   - Tracks results
3. **Progress tracking**:
   - Saves after each task
   - Can resume: `--start-at 5` or `--start-at-id task_003`
   - Shows progress: "Task 3/10: Implement session modes"
4. **Dependency checking** - Skips tasks if dependencies not met
5. **Stop on critical failure** - Stops if critical task fails

### Phase 5: CLI Interface (Priority: Medium)
1. **Main command**:
   ```bash
   python task_pipeline.py --plan-file PLAN.md
   ```
2. **Config override**:
   ```bash
   python task_pipeline.py --plan-file PLAN.md --config my_config.yaml
   ```
3. **Resume options**:
   ```bash
   python task_pipeline.py --skip-split --start-at 5
   python task_pipeline.py --skip-split --start-at-id task_003
   ```
4. **Dry run** - `--dry-run` to show what would run without running
5. **List tasks** - `--list-tasks` to show all tasks and status

### Phase 6: Testing (Priority: Medium)
1. **Unit test** `test_run_universal_refinement()`:
   - Mock LLM calls
   - Test return values
   - Test error handling
2. **Unit test** `test_load_config()`:
   - Test valid config
   - Test missing required fields
   - Test env var expansion
3. **Integration test** `test_task_pipeline()`:
   - Create fake PLAN.md
   - Run splitter
   - Run pipeline on 1-2 tasks
   - Verify results

## Implementation Order

1. **Phase 1** - Config system (foundation)
2. **Phase 2** - Refactor run_universal_refinement (enables Phase 4)
3. **Phase 3** - Task splitter (creates tasks for Phase 4)
4. **Phase 4** - Task pipeline (main feature)
5. **Phase 5** - CLI interface (usability)
6. **Phase 6** - Testing (quality assurance)

## File Structure

```
crow/
├── universal_refinement.py      # Refactored: importable function
├── task_pipeline.py             # New: main pipeline runner
├── task_splitter.py             # New: splits PLAN.md into tasks
├── config_loader.py             # New: loads and validates config
├── pipeline_config.yaml         # New: default config
├── prompts/
│   ├── task_splitter_prompt.txt # New: splitter prompt
│   ├── acp_planning_prompt.txt
│   ├── acp_implementation_prompt.txt
│   ├── acp_critic_prompt.txt
│   └── acp_documentation_prompt.txt
└── tasks/
    ├── task_manifest.json       # Created by splitter
    ├── task_001.md              # Created by splitter
    ├── task_002.md              # Created by splitter
    └── ...
```

## Success Metrics

- ✅ Can run: `python task_pipeline.py --plan-file PLAN.md`
- ✅ Config in YAML, not bash scripts
- ✅ Splits big plan into 5-15 small tasks
- ✅ Each task has clear acceptance criteria
- ✅ Runs tasks sequentially through iterative refinement
- ✅ Tracks progress, can resume after failure
- ✅ `run_universal_refinement()` is importable
