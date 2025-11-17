# OpenCode Subagent & Task System - Complete Documentation

## Critical Discoveries

1. **Dual-agent is a SUBAGENT invoked via the `task` tool, NOT a top-level API endpoint.**
2. **In Crow, we rename OpenCode's "dual-pair" to "dual-agent"** (less redundant)

## Crow Naming Convention

**Throughout this document:**
- ✅ Crow uses: `dual-agent` 
- ❌ OpenCode uses: `dual-pair` (we don't use this)
- All code examples use Crow's cleaner "dual-agent" naming

## The Task Tool

### Tool Definition
**File:** `opencode/packages/opencode/src/tool/task.ts`

**Purpose:** Spawn a subagent to handle a complex task autonomously

**Parameters:**
```typescript
{
  description: string,    // Short 3-5 word task description
  prompt: string,         // Full task for the agent
  subagent_type: string   // Which agent to use (build, dual-agent, etc.)
}
```

### How It Works

1. **Parent agent calls task tool:**
   ```
   task(
     description="Implement fibonacci",
     prompt="Write a fibonacci function with tests",
     subagent_type="dual-agent"
   )
   ```

2. **Task tool creates child session:**
   ```typescript
   const session = await Session.create({
     parentID: ctx.sessionID,  // Links to parent
     title: params.description + ` (@${agent.name} subagent)`,
     metadata: {
       includeParentContext: true,
       includeSiblingContext: true,
     },
   })
   ```

3. **Task tool checks subagent type:**
   ```typescript
   if (params.subagent_type === "dual-agent") {
     // Run dual-agent executor/discriminator loop
     const dualPairResult = await DualPair.run({
       sessionID: session.id,
       task: params.prompt,
       maxSteps: 50,
       model: { modelID, providerID },
     })
     return {
       title: params.description,
       metadata: {
         summary: toolParts,  // All tool calls from session
         sessionId: session.id,
         steps: dualPairResult.steps,
         completed: dualPairResult.completed,
       },
       output: dualPairResult.summary,
     }
   }
   ```

4. **For normal agents:**
   ```typescript
   const result = await SessionPrompt.prompt({
     messageID,
     sessionID: session.id,
     agent: agent.name,
     tools: { ...agent.tools },
     parts: promptParts,
   })
   return {
     title: params.description,
     metadata: {
       summary: all,  // All tool calls
       sessionId: session.id,
     },
     output: result.text,
   }
   ```

## Agent Hierarchy & Delegation Rules

**From task.ts:**

```typescript
// Supervisor/Orchestrator can ONLY delegate to BUILD
if (callingAgent === "supervisor" || callingAgent === "orchestrator") {
  if (params.subagent_type !== "build") {
    throw new Error(`${callingAgent} can only delegate to BUILD agent`)
  }
}

// Architect can ONLY delegate to SUPERVISOR/ORCHESTRATOR
if (callingAgent === "architect") {
  if (params.subagent_type !== "supervisor" && params.subagent_type !== "orchestrator") {
    throw new Error(`architect can only delegate to SUPERVISOR/ORCHESTRATOR`)
  }
}
```

**Hierarchy:**
```
User
  └─> General/Primary agents (unrestricted)
       └─> Architect
            └─> Supervisor/Orchestrator
                 └─> Build (can spawn dual-agent, explore, plan, etc.)
                      └─> Dual-Pair (executor + discriminator)
                      └─> Explore
                      └─> Plan
                      └─> Docs
                      └─> etc.
```

## Dual-Pair Agent

### Agent Definition
**File:** `opencode/.opencode/agent/dual-agent.md`

**Mode:** `subagent` (NOT primary - can only be invoked via task tool)

**Description:** "Supervised execution with executor/discriminator pair programming"

### How Dual-Pair Works

**One session with TWO perspectives:**

1. **DualPair.run()** orchestrates the loop:
   ```typescript
   let currentAgent: "executor" | "discriminator" = "executor"
   let steps = 0
   
   while (steps < maxSteps) {
     if (currentAgent === "executor") {
       // Transform conversation for executor's POV
       const executorView = DualPairPerspective.transformForExecutor(messages)
       
       // Executor does work with BUILD agent
       await SessionPrompt.prompt({
         sessionID,
         agent: "build",
         parts: [{ text: task }],
         metadata: {
           dualPairAgent: "executor",
           dualPairStep: steps,
         },
       })
       
       currentAgent = "discriminator"
       
     } else {
       // Transform for discriminator's POV
       const discriminatorView = DualPairPerspective.transformForDiscriminator(messages)
       
       // Discriminator reviews with DISCRIMINATOR agent
       await SessionPrompt.prompt({
         sessionID,
         agent: "discriminator",
         tools: {
           task_done: true,  // Only discriminator has this
           bash: true,
           read: true,
           grep: true,
           // etc.
         },
         parts: [{ text: "Review the executor's work..." }],
         metadata: {
           dualPairAgent: "discriminator",
           dualPairStep: steps,
         },
       })
       
       // Check if discriminator called task_done
       const session = await Session.get(sessionID)
       if (session.metadata?.dualPairComplete) {
         return {
           sessionID,
           steps,
           completed: true,
           summary: "...",
         }
       }
       
       currentAgent = "executor"
       steps++
     }
   }
   ```

2. **Perspective transformation:**
   - Executor sees discriminator's messages as "user" (feedback)
   - Discriminator sees executor's messages as "user" (work done)
   - Tool calls are rendered as markdown in the transformed view

### Example Invocation

**User to primary agent:**
```
Implement a fibonacci function in Python with tests
```

**Primary agent (general/build) decides to use dual-agent:**
```
I'll use the dual-agent agent for supervised implementation.

🔧 task(
  description="Implement fibonacci",
  prompt="Write a fibonacci function in Python with tests. Include type hints and comprehensive test cases.",
  subagent_type="dual-agent"
)
```

**Task tool response:**
```json
{
  "title": "Implement fibonacci",
  "metadata": {
    "sessionId": "ses-child-123",
    "steps": 3,
    "completed": true,
    "summary": [
      { "tool": "write", "file": "fib.py" },
      { "tool": "write", "file": "test_fib.py" },
      { "tool": "bash", "command": "python test_fib.py" }
    ]
  },
  "output": "Fibonacci function implemented with type hints and comprehensive tests. All tests passing."
}
```

**Primary agent to user:**
```
I've implemented the fibonacci function using supervised dual-agent execution:

✅ Created fib.py with type-hinted fibonacci function
✅ Created test_fib.py with comprehensive test cases
✅ All tests passing

The implementation was reviewed and verified by the discriminator agent.
```

## Session Structure

### Parent-Child Linking

```typescript
// Child session
{
  id: "ses-child-123",
  parentID: "ses-parent-456",  // Links to parent
  title: "Implement fibonacci (@dual-agent subagent)",
  metadata: {
    includeParentContext: true,
    includeSiblingContext: true,
    dualPairAgent: "executor",  // If dual-agent
    dualPairStep: 2,
    dualPairComplete: true,
  }
}
```

### Tool Call Metadata

Parent agent sees child's work as tool metadata:

```typescript
{
  tool: "task",
  metadata: {
    title: "Implement fibonacci",
    sessionId: "ses-child-123",
    summary: [
      // All tool calls from child session
      { tool: "write", file: "fib.py", ... },
      { tool: "bash", command: "python test_fib.py", ... }
    ],
    steps: 3,  // Only for dual-agent
    completed: true,  // Only for dual-agent
  },
  output: "Fibonacci function implemented..."
}
```

## Key Architecture Points

### 1. Task Tool is THE spawning mechanism
- Not a REST endpoint
- Not a special API call
- Just a regular tool that agents can use

### 2. Dual-pair is a subagent type
- Invoked via `task(subagent_type="dual-agent")`
- Not directly callable by user
- Must be spawned by another agent

### 3. Session hierarchy
- Parent session spawns child session
- Child session has `parentID` link
- Metadata tracks relationship

### 4. Return value
- Task tool returns summary + output
- Parent agent decides what to tell user
- Child session is stateless from parent's POV

### 5. Default behavior
- **OpenCode**: Primary agents can choose any subagent
- **Crow should**: BUILD agent defaults to dual-agent for implementation tasks

## What Crow Needs

### 1. Task Tool ✅ (already exists as TodoWrite/TodoRead pattern)
Need to implement:
```rust
pub struct TaskTool;

impl Tool for TaskTool {
    fn name(&self) -> &str { "task" }
    
    fn description(&self) -> &str {
        // Dynamic - lists available subagents
    }
    
    async fn execute(&self, args: TaskArgs, ctx: ToolContext) -> ToolResult {
        // 1. Create child session with parentID
        // 2. Check subagent_type
        // 3. If "dual-agent" -> run DualAgentRuntime
        // 4. Else -> run single agent
        // 5. Return summary + output
    }
}
```

### 2. Agent Mode Field ✅ (already exists)
```rust
pub enum AgentMode {
    Primary,   // Can be used directly by user
    Subagent,  // Only via task tool
    All,       // Both
}
```

### 3. Dual-Pair as Subagent
Make dual-agent agent:
```rust
Agent {
    name: "dual-agent",
    mode: AgentMode::Subagent,
    description: "Supervised execution with executor/discriminator",
    // ...
}
```

### 4. Default BUILD Behavior
BUILD agent should automatically use dual-agent:
```rust
// In BUILD agent prompt:
"When implementing code, use the task tool with subagent_type='dual-agent' 
for supervised execution. This ensures quality through discriminator review."
```

### 5. Session Parent Tracking ✅ (already exists)
```rust
pub struct Session {
    pub parent_id: Option<String>,  // ✅ Already there
    pub metadata: Option<serde_json::Value>,  // ✅ Added
}
```

## Implementation Priority

1. **Task Tool** - Core spawning mechanism
2. **Dual-Pair as Subagent** - Register it properly
3. **BUILD Agent Defaults** - Auto-spawn dual-agent
4. **Telemetry** - Export both sessions to markdown
5. **Metadata Passing** - Return tool summaries to parent

## Example Crow Flow (Target)

```bash
# User sends message to BUILD agent
POST /session/ses-123/message
{
  "agent": "build",
  "parts": [{"text": "Implement fibonacci function"}]
}

# BUILD agent decides to use dual-agent
# Calls task tool internally:
task(
  description="Implement fibonacci",
  prompt="Write fibonacci with tests",
  subagent_type="dual-agent"
)

# Task tool:
# 1. Creates child session (ses-child-456) with parentID=ses-123
# 2. Runs DualAgentRuntime.run(ses-child-456, task)
# 3. Executor and discriminator work in ses-child-456
# 4. Exports ses-child-456 to .crow/sessions/ses-child-456.md
# 5. Returns to BUILD agent:
{
  "output": "Fibonacci implemented with tests",
  "metadata": {
    "sessionId": "ses-child-456",
    "steps": 3,
    "summary": [...]
  }
}

# BUILD agent synthesizes response to user
# User sees: "I've implemented fibonacci using supervised execution..."
```

This is the architecture we need to build!
