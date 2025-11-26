# E2E Testing Guide for Agents

This document is for AI agents running E2E tests on crow. These are not scripted tests - they require intelligence to evaluate.

## Philosophy

Traditional CI/CD: Script runs, checks exit code, pass/fail.

Agent CI/CD: Agent runs commands, observes behavior, uses judgment to determine if things are working correctly. Catches issues like:
- Model calling tools repeatedly for no reason
- Tool succeeding but agent not reporting result
- Subtle behavioral regressions
- Context bloat
- Wrong tool selection

---

## Before You Start

### 1. Build crow-cli
```bash
cd crow-tauri/src-tauri
cargo build --bin crow-cli
```

### 2. Verify provider is working
```bash
crow-cli chat "Say: HELLO"
```
You should see output with thinking, response, cost stats. If it fails, check API keys.

### 3. Create a fresh test directory
```bash
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"
```

---

## Test 1: Session Management

### Run
```bash
crow-cli new "Session Test"
```

### Verify
- Output shows a session ID like `ses_xxxxx`
- `crow-cli sessions` lists the new session

### What to watch for
- Session created in correct working directory
- Title appears in session list

---

## Test 2: Bash Tool

### Setup
```bash
crow-cli new "Bash Test"
# Note the session ID
```

### Run
```bash
crow-cli chat --session ses_XXX "Run this bash command: echo CROW_BASH_TEST"
```

### Verify
- Output contains `CROW_BASH_TEST`
- Tool was called exactly ONCE
- Exit code was 0

### What to watch for
- **RED FLAG**: Tool called multiple times for simple echo
- **RED FLAG**: Agent uses wrong approach (like trying to write a file instead of running bash)
- Check the stats line: `✓ ~XX thinking, ~XX response | 1 tool calls`

### Additional bash tests
```bash
crow-cli chat --session ses_XXX "Run: pwd"
# Should show the test directory

crow-cli chat --session ses_XXX "Run: echo hello | tr a-z A-Z"  
# Should show HELLO, tests pipe handling
```

---

## Test 3: Write Tool

### Setup
```bash
crow-cli new "Write Test"
cd "$TEST_DIR"  # Make sure you're in test dir
```

### Run
```bash
crow-cli chat --session ses_XXX "Create a file named hello.txt with content: Hello from Crow"
```

### Verify
```bash
cat hello.txt
# Should contain "Hello from Crow"
```

### What to watch for
- File actually created (not just agent saying it did)
- Content is correct
- Agent used Write tool, not Bash with echo redirect
- Check snapshots: `ls ~/.local/share/crow/snapshots/` - new project should appear

---

## Test 4: Read Tool

### Setup
```bash
echo "This is line one" > read_test.txt
echo "This is line two" >> read_test.txt
crow-cli new "Read Test"
```

### Run
```bash
crow-cli chat --session ses_XXX "Read read_test.txt and tell me what's on line two"
```

### Verify
- Agent mentions "line two" or the content
- Agent used Read tool (check output shows `read` tool call)

### What to watch for
- Agent should NOT use bash `cat` - should use Read tool
- Agent should understand and report the content, not just dump it

---

## Test 5: Edit Tool

### Setup
```bash
echo "Hello World" > edit_test.txt
crow-cli new "Edit Test"
```

### Run
```bash
crow-cli chat --session ses_XXX "Edit edit_test.txt: replace World with Crow"
```

### Verify
```bash
cat edit_test.txt
# Should say "Hello Crow"
```

### What to watch for
- **CRITICAL**: File actually changed on disk
- Agent read the file first (should see `read` then `edit` in tool calls)
- Edit tool showed the diff in output
- **RED FLAG**: Agent calls edit multiple times
- **RED FLAG**: Agent says it edited but file unchanged

### Check snapshot captured the change
```bash
ls -la ~/.local/share/crow/snapshots/
# Find the project for this test dir
cd ~/.local/share/crow/snapshots/proj-XXX
git status
# Should show the edited file
```

---

## Test 6: Grep Tool

### Setup
```bash
echo "ERROR: something failed" > log1.txt
echo "INFO: all systems go" > log2.txt
echo "ERROR: another failure" > log3.txt
crow-cli new "Grep Test"
```

### Run
```bash
crow-cli chat --session ses_XXX "Use grep to find files containing ERROR"
```

### Verify
- Agent reports log1.txt and log3.txt (not log2.txt)
- Used the Grep tool (not bash grep)

### What to watch for
- Should find exactly 2 files
- **RED FLAG**: Agent runs bash `grep` instead of Grep tool
- **RED FLAG**: Calls grep tool multiple times

---

## Test 7: Glob Tool

### Setup
```bash
mkdir -p src/components
touch src/app.js src/index.ts
touch src/components/Button.tsx src/components/Input.tsx
crow-cli new "Glob Test"
```

### Run
```bash
crow-cli chat --session ses_XXX "Find all .tsx files using glob"
```

### Verify
- Agent finds Button.tsx and Input.tsx
- Used Glob or List tool with pattern

### What to watch for
- Should find exactly 2 .tsx files
- **RED FLAG**: Tool called many times (seen this happen - model gets confused)

---

## Test 8: List Tool

### Setup
Use same directory structure from Glob test.

### Run
```bash
crow-cli chat --session ses_XXX "List the contents of the src directory"
```

### Verify
- Agent lists app.js, index.ts, components/
- Understands directory structure

---

## Test 9: Multi-Tool Workflow

### Setup
```bash
crow-cli new "Workflow Test"
```

### Run
```bash
crow-cli chat --session ses_XXX "Create a file called config.json with content {\"name\": \"test\"}, then read it back and tell me what the name field is"
```

### Verify
- File created with valid JSON
- Agent correctly reports name is "test"
- Used Write then Read tools

### What to watch for
- Correct tool sequence (write, read)
- Agent synthesizes information from multiple tool calls

---

## Test 10: Error Handling

### Run
```bash
crow-cli chat --session ses_XXX "Read the file nonexistent_file_12345.txt"
```

### Verify
- Agent gracefully handles error
- Reports file doesn't exist
- Does NOT hallucinate file contents

### What to watch for
- Agent should acknowledge the error, not make up content
- Should not retry excessively

---

## Evaluating Results

After running tests, evaluate:

### Per-Test Checklist
- [ ] Tool produced correct result
- [ ] Tool called appropriate number of times (usually 1-2)
- [ ] Agent reported result correctly
- [ ] No hallucinated information
- [ ] Reasonable token usage (check Context: XX/128k)

### Overall Health Indicators
- [ ] Sessions persisting correctly: `crow-cli sessions`
- [ ] Snapshots being created: `ls ~/.local/share/crow/snapshots/`
- [ ] Logs being written: `cat ~/.local/state/crow/logs/tool-calls.jsonl | tail -5`

### Red Flags That Need Investigation
1. Tool called 5+ times for simple task → Model confusion, check prompts
2. Context above 50% for simple test → Token bloat, check tool output truncation  
3. Agent says success but filesystem unchanged → Tool execution bug
4. Agent uses bash when dedicated tool exists → Prompt needs adjustment
5. Consistent timeouts → Process management issue

---

## Reporting Results

When reporting test results, include:
1. Which tests passed/failed
2. Any red flags observed
3. Token/cost stats from final test
4. Any behavioral issues (even if test "passed")

Example:
```
E2E Test Results:
- Session: PASS
- Bash: PASS  
- Write: PASS
- Read: PASS
- Edit: PASS (but called tool 3x, investigate)
- Grep: PASS
- Glob: FAIL - tool called 10 times, model confused
- List: PASS
- Workflow: PASS
- Error handling: PASS

Issues:
- Glob test shows model calling tool repeatedly
- Edit test has unnecessary extra read at end

Stats from final test:
- Context: 31k/128k (24.8%)
- Cost: $0.0058
- Time: 32.0s
```

---

## Quick Smoke Test

If you just need to verify crow is working:

```bash
TEST_DIR=$(mktemp -d) && cd "$TEST_DIR"
SESSION=$(crow-cli new "Smoke Test" 2>&1 | grep -o 'ses_[a-zA-Z0-9]*')
echo "Session: $SESSION"

# Test 1: Bash
crow-cli chat --session "$SESSION" "Run: echo SMOKE_TEST"

# Test 2: Write + Read
crow-cli chat --session "$SESSION" "Create test.txt with 'hello', then read it back"
cat test.txt

# Test 3: Edit
echo "foo bar" > edit.txt
crow-cli chat --session "$SESSION" "Edit edit.txt: change foo to baz"
cat edit.txt

echo "Smoke test complete. Check outputs above."
```

If all three work and files are correct, crow is healthy.
