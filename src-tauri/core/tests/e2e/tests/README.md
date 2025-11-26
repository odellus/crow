# E2E Test Scripts

Each script tests one tool. Run individually and evaluate the output.

## Usage

```bash
# Set crow-cli path (or add to PATH)
export CROW_CLI="/path/to/crow-cli"

# Run a single test
bash 01_session.sh

# Or run and evaluate
bash 05_edit.sh
# Then check: Did file change? How many tool calls? Any red flags?
```

## Tests

| Script | Tool | What to Check |
|--------|------|---------------|
| 01_session.sh | Session | ID returned, appears in list |
| 02_bash.sh | Bash | Output correct, 1 tool call each |
| 03_write.sh | Write | File exists with correct content |
| 04_read.sh | Read | Agent reports content correctly |
| 05_edit.sh | Edit | File changed on disk |
| 06_grep.sh | Grep | Correct files found |
| 07_glob.sh | Glob | Correct files, watch call count |
| 08_list.sh | List | Directory contents shown |
| 09_workflow.sh | Multi | Correct sequence, synthesizes |
| 10_errors.sh | Errors | No hallucination |

## Agent Evaluation

After running each test, the script prints "AGENT: Verify" with specific things to check. Use judgment - a test can "pass" but still have issues (like too many tool calls).

## Red Flags

- Tool called 5+ times for simple task
- File unchanged after edit
- Agent uses bash when dedicated tool exists
- Agent hallucinates content
- Context usage above 50% for simple test
