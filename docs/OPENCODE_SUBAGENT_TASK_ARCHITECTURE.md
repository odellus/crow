# OpenCode Subagent and Task Architecture - Complete Documentation

> **Critical architectural documentation for understanding how OpenCode implements subagents, task delegation, dual-pair supervision, and session hierarchies.**

This document provides EXTREMELY thorough coverage of OpenCode's subagent and task system based on code analysis of the opencode submodule.

---

## Table of Contents

1. [Subagent/Task Architecture](#1-subagenttask-architecture)
2. [Dual-Pair as Subtask](#2-dual-pair-as-subtask)
3. [Tool/API for Spawning](#3-toolapi-for-spawning)
4. [Session Hierarchy](#4-session-hierarchy)
5. [Code Locations](#5-code-locations)
6. [Examples](#6-examples)
7. [Key Differences: Task vs Subagent vs Dual-Pair](#7-key-differences)
8. [Communication Flow](#8-communication-flow)

---

## 1. Subagent/Task Architecture

### Overview

OpenCode implements delegation through a **Task tool** that spawns **child sessions**. There's no separate "subagent" concept - instead, agents delegate work by creating child sessions with different agent configurations.

### Core Concepts

**Task**: A work unit that gets delegated to a specialized agent in a new session  
**Subagent**: An agent with `mode: "subagent"` that can only be invoked via the Task tool  
**Child Session**: A new session created with `parentID` pointing to the delegating session  
**Agent Mode**: Controls where an agent can be used (`primary`, `subagent`, or `all`)

### How an Agent Spawns a Subtask/Subagent

#### The Task Tool Definition

**File**: `/home/thomas/src/projects/opencode-project/opencode/packages/opencode/src/tool/task.ts`

```typescript
export const TaskTool = Tool.define("task", async () => {
  const agents = await Agent.list().then((x) => 
    x.filter((a) => a.mode !== "primary")  // Only show subagents
  )
  
  const description = DESCRIPTION.replace(
    "{agents}",
    agents
      .map((a) => `- ${a.name}: ${a.description ?? "This subagent should only be called manually by the user."}`)
      .join("\n"),
  )
  
  return {
    description,
    parameters: z.object({
      description: z.string().describe("A short (3-5 words) description of the task"),
      prompt: z.string().describe("The task for the agent to perform"),
      subagent_type: z.string().describe("The type of specialized agent to use for this task"),
    }),
    async execute(params, ctx) {
      // ... implementation
    }
  }
})
```

**Key Points**:
- Tool description is **dynamically generated** with list of available subagents
- LLM sees agent names and descriptions to pick the right one
- Parameters: description, prompt, and subagent_type

#### Delegation Restrictions

**File**: `task.ts:32-47`

```typescript
// Enforce subagent restrictions
const callingAgent = ctx.agent
if (callingAgent === "supervisor" || callingAgent === "orchestrator") {
  if (params.subagent_type !== "build") {
    throw new Error(
      `${callingAgent} can only delegate to BUILD agent. Attempted to invoke: ${params.subagent_type}`,
    )
  }
}
if (callingAgent === "architect") {
  if (params.subagent_type !== "supervisor" && params.subagent_type !== "orchestrator") {
    throw new Error(
      `architect can only delegate to SUPERVISOR/ORCHESTRATOR agent. Attempted to invoke: ${params.subagent_type}`,
    )
  }
}
```

**Delegation Hierarchy**:
```
ARCHITECT
  └─> Can only delegate to SUPERVISOR or ORCHESTRATOR

SUPERVISOR / ORCHESTRATOR  
  └─> Can only delegate to BUILD

BUILD
  └─> Can delegate to any subagent (general, explore, custom, etc.)
```

### Task vs Subagent: What's the Difference?

**Task**: The *action* of delegating work (via Task tool)  
**Subagent**: The *agent type* that executes the task (mode: "subagent")

**Example**:
```
Parent Agent: BUILD (mode: "all")
  ├─ Calls Task tool with subagent_type="general"
  └─> Creates child session with agent="general" (mode: "subagent")
```

**Agent Modes**:
- `mode: "primary"` - Can be selected in TUI, cannot be delegated to
- `mode: "subagent"` - Can only be invoked via Task tool
- `mode: "all"` - Can be both primary AND subagent (e.g., BUILD)

**File**: `/home/thomas/src/projects/opencode-project/opencode/packages/opencode/src/agent/agent.ts:95-170`

### How They Communicate with Parent

Communication is **asynchronous and unidirectional**:

1. **Parent → Child**: Via the initial prompt in Task tool
2. **Child → Parent**: Via the Task tool's return value

**There is NO ongoing bidirectional communication**. Once the child session completes, it returns a result object to the parent.

**Communication Mechanism** (`task.ts:65-89`):

```typescript
// Parent sets up child session
const session = await Session.create({
  parentID: ctx.sessionID,  // Link to parent
  title: params.description + ` (@${agent.name} subagent)`,
  metadata: {
    includeParentContext: true,     // Child sees parent messages
    includeSiblingContext: true,    // Child sees sibling sessions
  },
})

// Parent subscribes to child's tool call events
const parts: Record<string, MessageV2.ToolPart> = {}
const unsub = Bus.subscribe(MessageV2.Event.PartUpdated, async (evt) => {
  if (evt.properties.part.sessionID !== session.id) return
  if (evt.properties.part.type !== "tool") return
  parts[evt.properties.part.id] = evt.properties.part
  
  // Update parent's Task tool call metadata with child's progress
  ctx.metadata({
    title: params.description,
    metadata: {
      summary: Object.values(parts).sort((a, b) => a.id?.localeCompare(b.id)),
      sessionId: session.id,
    },
  })
})
```

**What the Parent Sees**:
- Child session ID
- Real-time updates of child's tool calls
- Final text response from child
- Summary of all tool calls made by child

**What the Parent Does NOT See**:
- Child's intermediate reasoning
- Child's text responses (except final)
- Child's errors (unless fatal)

### Lifecycle

#### 1. Invocation

```
Parent Agent LLM
  └─> Decides to delegate
      └─> Calls Tool: task(description="...", prompt="...", subagent_type="general")
```

#### 2. Child Session Creation

```typescript
// task.ts:48-55
const session = await Session.create({
  parentID: ctx.sessionID,  // Points to parent
  title: params.description + ` (@${agent.name} subagent)`,
  metadata: {
    includeParentContext: true,
    includeSiblingContext: true,
  },
})
```

**What happens**:
- New session ID generated
- Parent-child link established via `parentID`
- Child session gets metadata flags for context injection
- Session stored in storage layer

#### 3. Execution

**Normal Single-Agent Flow** (`task.ts:123-148`):

```typescript
const promptParts = await SessionPrompt.resolvePromptParts(params.prompt)
const result = await SessionPrompt.prompt({
  messageID,
  sessionID: session.id,
  model: model,
  agent: agent.name,  // e.g., "general", "build"
  tools: {
    todowrite: false,  // Subagents cannot use todos
    todoread: false,
    task: false,       // Subagents cannot recursively delegate
    ...agent.tools,    // Agent's configured tools
  },
  parts: promptParts,
})
```

**Dual-Pair Flow** (if `subagent_type === "dual-pair"`):

```typescript
// task.ts:90-121
const dualPairResult = await DualPair.run({
  sessionID: session.id,
  task: params.prompt,
  maxSteps: 50,
  model: model,
})
```

#### 4. Completion and Return

```typescript
// task.ts:149-159
// Collect all tool calls from child session
let all = await Session.messages({ sessionID: session.id })
all = all.filter((x) => x.info.role === "assistant")
const toolParts = all.flatMap((msg) => 
  msg.parts.filter((x) => x.type === "tool") as MessageV2.ToolPart[]
)

return {
  title: params.description,
  metadata: {
    summary: toolParts,      // All tool calls made by child
    sessionId: session.id,   // Link to child session
  },
  output: (result.parts.findLast((x) => x.type === "text") as any)?.text ?? "",
}
```

**What Gets Returned to Parent**:
- `title`: Task description
- `metadata.summary`: Array of all tool calls made by child
- `metadata.sessionId`: Child session ID for reference
- `output`: Final text response from child agent

**How Parent Receives It**:

The Task tool returns a result object that becomes part of the parent's tool call result. The parent LLM sees this in the next turn:

```xml
<tool_result name="task">
  <title>Search for authentication code</title>
  <sessionId>session_abc123</sessionId>
  <summary>
    [Array of tool calls: {tool: "grep", input: {...}, output: "..."}]
  </summary>
  <output>
    Found authentication implementation in src/auth.ts lines 45-89.
    Uses JWT with bcrypt for password hashing.
  </output>
</tool_result>
```

---

## 2. Dual-Pair as Subtask

### Overview

Dual-pair is a **supervision pattern** where two agents collaborate in one session:
- **Executor** (BUILD agent): Does the implementation work
- **Discriminator** (custom agent): Reviews and validates work

They share ONE session but see DIFFERENT perspectives through role inversion.

### How Dual-Pair is Invoked as a Subtask

**File**: `task.ts:86-121`

```typescript
// Check if this is a dual-pair subagent
if (params.subagent_type === "dual-pair") {
  // Run dual-pair executor/discriminator loop
  const dualPairResult = await DualPair.run({
    sessionID: session.id,
    task: params.prompt,
    maxSteps: 50,  // Default to 50 executor→discriminator cycles
    model: {
      modelID: model.modelID,
      providerID: model.providerID,
    },
  })

  unsub()

  // Get all tool calls from the session
  let all = await Session.messages({ sessionID: session.id })
  all = all.filter((x) => x.info.role === "assistant")
  const toolParts = all.flatMap((msg) => 
    msg.parts.filter((x: any) => x.type === "tool") as MessageV2.ToolPart[]
  )

  return {
    title: params.description,
    metadata: {
      summary: toolParts,
      sessionId: session.id,
      steps: dualPairResult.steps,      // Number of executor→discriminator cycles
      completed: dualPairResult.completed,  // Did discriminator approve?
    } as any,
    output: dualPairResult.summary || `Dual-pair session completed in ${dualPairResult.steps} steps`,
  }
}
```

### Which Agent Spawns Dual-Pair?

**Any agent** that has the Task tool can spawn dual-pair, but typically:

```
SUPERVISOR or ORCHESTRATOR
  └─> Task(subagent_type="dual-pair", prompt="Implement feature X")
      └─> Creates child session
          └─> DualPair.run() manages executor/discriminator loop
```

**Configuration**:

Dual-pair is defined as a custom agent in `.opencode/agent/dual-pair.md`:

```markdown
---
description: Supervised execution with executor/discriminator pair programming
mode: subagent
---

You are part of a DUAL-PAIR supervision system...
```

### How the Parent Agent Decides to Use Dual-Pair

The parent agent (e.g., SUPERVISOR) sees dual-pair listed in the Task tool description:

**File**: `task.txt`:
```
Available agent types and the tools they have access to:
- dual-pair: Supervised execution with executor/discriminator pair programming
- general: General-purpose agent for researching complex questions
- build: Standard implementation agent
```

The LLM chooses based on:
1. Task complexity (dual-pair for complex, high-risk tasks)
2. Need for validation (dual-pair provides review)
3. Parent agent's prompt/instructions

**Example Decision Logic** (in SUPERVISOR agent):
```
If task requires:
  - Multiple files
  - Critical functionality (auth, payment, security)
  - Quality verification
Then: Use dual-pair
Else: Use build
```

### Default Behavior

**Default**: Normal single-agent execution (BUILD agent)

**Dual-Pair**: Only used when explicitly invoked via `subagent_type="dual-pair"`

**File**: `task.ts:86`
```typescript
if (params.subagent_type === "dual-pair") {
  // Special handling for dual-pair
} else {
  // Normal single-agent flow (default)
}
```

### Dual-Pair Internal Implementation

**File**: `/home/thomas/src/projects/opencode-project/opencode/packages/opencode/src/session/dual-pair.ts`

```typescript
export async function run(config: Config): Promise<Result> {
  const { sessionID, task, model } = config
  const maxSteps = config.maxSteps ?? 50

  let currentAgent: "executor" | "discriminator" = "executor"
  let steps = 0  // A step = executor work + discriminator review

  while (steps < maxSteps) {
    const messages = await Session.messages({ sessionID })

    // Check if discriminator marked task as done
    const session = await Session.get(sessionID)
    if ((session.metadata as any)?.dualPairComplete) {
      log.info("dual-pair completed by discriminator")
      
      // Get discriminator's final text response as summary
      const lastMessage = messages[messages.length - 1]
      const summary = lastMessage?.parts
        .filter((p) => p.type === "text")
        .map((p) => p.text)
        .join("\n") || "Task completed"

      return {
        sessionID,
        steps,
        completed: true,
        summary,
      }
    }

    if (currentAgent === "executor") {
      // Executor's turn - do the work
      const executorView = DualPairPerspective.transformForExecutor(messages)

      await SessionPrompt.prompt({
        sessionID,
        messageID: Identifier.ascending("message"),
        agent: "build",  // Executor uses BUILD agent
        model,
        parts: [{
          type: "text",
          text: steps === 0 ? task : "Continue working on the task based on feedback",
          metadata: {
            dualPairAgent: "executor",
            dualPairStep: steps,
          },
        }],
      })

      currentAgent = "discriminator"  // Always switch after executor
    } else {
      // Discriminator's turn - review and provide feedback
      const discriminatorView = DualPairPerspective.transformForDiscriminator(messages)

      await SessionPrompt.prompt({
        sessionID,
        messageID: Identifier.ascending("message"),
        agent: "discriminator",  // Custom review agent
        model,
        tools: {
          task_done: true,  // Only discriminator can mark done
          todowrite: true,
          todoread: true,
          read: true,
          grep: true,
          bash: true,
          glob: true,
        },
        parts: [{
          type: "text",
          text: "Review the executor's work. Provide specific feedback, run tests if needed, or use task_done if everything is satisfactory.",
          metadata: {
            dualPairAgent: "discriminator",
            dualPairStep: steps,
          },
        }],
      })

      steps++  // Discriminator response completes the step
      currentAgent = "executor"
    }
  }

  // Max steps reached without completion
  return {
    sessionID,
    steps,
    completed: false,
  }
}
```

**Key Points**:
1. **Single Session, Two Perspectives**: Both agents share one session but see different views
2. **Turn-Based**: Executor → Discriminator → Executor → ...
3. **Step Counting**: One step = executor work + discriminator review
4. **Completion Signal**: Discriminator calls `task_done` tool to exit loop
5. **Max Steps**: Prevents infinite loops (default 50 steps)

### Role Inversion (Perspective Transformation)

**File**: `/home/thomas/src/projects/opencode-project/opencode/packages/opencode/src/session/dual-pair-perspective.ts`

```typescript
export function transformForExecutor(messages: MessageV2.WithParts[]): Array<{
  role: "user" | "assistant"
  content: string
}> {
  const transformed: Array<{ role: "user" | "assistant"; content: string }> = []

  for (const msg of messages) {
    if (msg.info.role === "user") {
      // Original user message
      transformed.push({
        role: "user",
        content: ToolRenderer.renderMessage(msg),
      })
    } else if (msg.info.role === "assistant") {
      const agentType = getDualPairAgent(msg)

      if (agentType === "executor") {
        // My own messages = assistant
        transformed.push({
          role: "assistant",
          content: ToolRenderer.renderMessage(msg),
        })
      } else if (agentType === "discriminator") {
        // Discriminator's messages = user (feedback from supervisor)
        transformed.push({
          role: "user",
          content: ToolRenderer.renderMessage(msg),
        })
      }
    }
  }

  return transformed
}
```

**What This Means**:

**Executor sees**:
```
USER: Implement fibonacci function
ASSISTANT (executor): I'll implement it with memoization [tool: write]
USER (discriminator's feedback): Add type hints and tests
ASSISTANT (executor): Adding type hints now [tool: edit]
```

**Discriminator sees**:
```
USER: Implement fibonacci function
USER (executor's work): ## Tools Used
  ### write
  **Input**: file="fib.py", content="def fib(n): ..."
ASSISTANT (discriminator): Good start, but add type hints and tests [tool: todowrite]
USER (executor's work): ## Tools Used
  ### edit
  **Input**: file="fib.py", ...
ASSISTANT (discriminator): Perfect! [tool: task_done]
```

**Why This Works**:
- Each agent sees their own messages as "assistant" (continuity)
- Each agent sees the OTHER agent's messages as "user" (instructions/feedback)
- Tool calls are rendered as markdown for readability
- Metadata tags identify which agent sent which message

---

## 3. Tool/API for Spawning

### The Task Tool

**Tool Name**: `task`

**File**: `/home/thomas/src/projects/opencode-project/opencode/packages/opencode/src/tool/task.ts`

### API Signature

```typescript
interface TaskToolInput {
  description: string      // Short (3-5 words) description of task
  prompt: string          // Detailed task for the agent to perform
  subagent_type: string   // Which agent to use (e.g., "general", "build", "dual-pair")
}

interface TaskToolOutput {
  title: string                          // Task description
  output: string                         // Final text response from subagent
  metadata: {
    summary: MessageV2.ToolPart[]       // All tool calls made by subagent
    sessionId: string                    // Child session ID
    steps?: number                       // For dual-pair: number of cycles
    completed?: boolean                  // For dual-pair: did discriminator approve?
  }
}
```

### How the Parent Agent Calls It

**From LLM Perspective** (what Claude sees):

```xml
<function_calls>
<invoke name="task">
<parameter name="description">Search for auth code