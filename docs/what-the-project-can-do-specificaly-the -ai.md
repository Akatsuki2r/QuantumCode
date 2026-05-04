# AI Workflow Rules

## Overview

This document explains what AI can do in this project and how it executes tasks. The AI operates as an intelligent agent with access to tools, constrained by policies, and guided by a multi-layer routing system.

---

## Core Capabilities

### Tool System

The AI has access to 6 core tools:

| Tool | Purpose | Safety | Confirmation |
|------|---------|--------|--------------|
| **Read** | Read file contents | Low | Never |
| **Write** | Create or modify files | Medium | Write/Bash/Git |
| **Bash** | Execute shell commands | High | Write/Bash/Git |
| **Grep** | Search file contents | Low | Never |
| **Glob** | Find files by pattern | Low | Never |
| **Find** | Locate paths/directories | Low | Never |

**Safety Model**:
- Read operations: Always allowed, read-only
- Grep/Glob/Find: Always allowed, search-only
- Write: Requires confirmation for complex operations
- Bash: Requires confirmation (especially destructive commands)
- Git: Requires confirmation (commits, pushes, branches)

### Provider System

The AI can use multiple AI providers:

| Provider | Type | Cost | Availability |
|----------|------|------|--------------|
| **Ollama** | Local | Free | Always available if running |
| **llama.cpp** | Local | Free | Requires model path |
| **Anthropic (Claude)** | Cloud | $ | API key required |
| **OpenAI (GPT)** | Cloud | $ | API key required |
| **LM Studio** | Local | Free | API on localhost |

**Provider Selection**:
1. Check configuration (prefer_local flag)
2. Check local availability (Ollama/llama.cpp/LM Studio)
3. Check API key availability (Anthropic/OpenAI)
4. Fall back to local if configured
5. Fail gracefully if no provider available

---

## Execution Workflow

### Step 1: Intent Classification (Regex-based, < 1ms)

The AI analyzes your prompt using regex patterns to identify the task type. This happens in milliseconds.

**16 Intent Types**:

```rust
// File operations
Read      - "read src/main.rs"
Write     - "write new_feature.rs"
Edit      - "edit config.toml"
Delete    - "delete temp.txt"

// Shell operations
Bash      - "run cargo build"
Git       - "git commit -m 'fix'"

// Search operations
Grep      - "grep pattern file"
Glob      - "glob **/*.rs"
Find      - "find path/to/file"

// Analysis operations
Explain   - "what is a mutex?"
Review    - "review this code"
Debug     - "debug the race condition"

// Planning operations
Plan      - "plan the architecture"
Design    - "design the API"

// Meta operations
Help      - "help"
Chat      - "hi", "hello", "thanks"
Unknown   - Fallback for unrecognized patterns
```

**How it works**:
- RegexSet matches all patterns in single pass
- First match wins (priority order matters)
- Empty prompt → Intent::Unknown

### Step 2: Complexity Estimation (Keyword-weighted, < 1ms)

The AI scores your task complexity using weighted keywords.

**5 Complexity Levels**:

```
Trivial (0)  - System commands, trivial tasks
Simple (1)   - Single action, read/list
Moderate (2) - Write/edit, test, function/method
Complex (3)  - Refactor, optimize, API
Heavy (4)    - Security, architecture, ML/AI
```

**Example**:
- "ls" → Trivial (-3)
- "read src/main.rs" → Simple (1)
- "refactor authentication" → Complex (3)
- "design distributed microservices with security" → Heavy (4)

### Step 3: Mode Selection (5 modes)

Based on intent and complexity, the AI selects an execution mode.

**5 Execution Modes**:

```rust
Chat     - Conversational, minimal tools, quick responses
Plan     - Analysis only, read-only, no execution
Build    - Full execution capabilities, can modify files
Review   - Read-only analysis, code review, no modifications
Debug    - Diagnostic tools only, limited execution
```

**Mode Selection Logic**:

```
Read/Explain/Help/Chat          → Chat (simple) or Review (complex)
Write/Edit/Delete               → Build
Git/Bash                         → Build
Grep/Glob/Find                   → Review
Review                           → Review
Debug                            → Debug
Plan/Design                      → Plan
Chat                             → Chat
```

**Mode State Machine**:

```
Valid Transitions:
chat → plan, build, debug
plan → build, review, chat
build → review, debug, plan, chat
review → build, plan, chat
debug → build, plan, chat

Forbidden Transitions:
plan → debug
review → debug
```

**Mode Behavior**:

| Mode | Tools Allowed | Execution | Confirmation |
|------|---------------|-----------|--------------|
| Chat | Read only | No | Never |
| Plan | Read, Grep, Glob | No | Never |
| Build | All tools | Yes | Write/Bash/Git |
| Review | Read, Grep, Glob | No | Never |
| Debug | Read, Grep, Glob, Bash | Yes | Yes |

### Step 4: Model Tier Selection

Based on complexity and mode, the AI selects a model tier.

**5 Model Tiers**:

```
Local     - Ollama/llama.cpp (free, offline)
OpenCode  - qwen-2.5-coder-7b (free, variable quality)
Fast      - claude-haiku/gpt-4o-mini (fast, $)
Standard  - claude-sonnet/gpt-4o (balanced, $$$)
Capable   - claude-opus/gpt-4 (complex, $$$)
```

**Selection Rules**:
- Prefer local if `prefer_local` is true
- Use Fast for simple tasks
- Use Standard for moderate tasks
- Use Capable for complex/heavy tasks
- Use Local by default if available

### Step 5: Tool Policy Determination

The AI determines which tools are allowed for this specific task.

**Policy Rules**:

```
Read intent       → Read, Grep, Glob allowed
Write intent      → All tools allowed (if Build mode)
Git intent        → All tools, require confirmation
Bash intent       → All tools, require confirmation
Grep intent       → Grep, Glob allowed
Review intent     → Read, Grep, Glob allowed
Debug intent      → Read, Grep, Glob, Bash allowed
Plan intent       → Read, Grep, Glob allowed
Chat intent       → Read only (minimal tools)
```

**Safety Checks**:
- Destructive operations (Write, Bash, Git) → require confirmation in Build mode
- Write-only operations (Write, Edit, Delete) → require confirmation in Build mode
- Search-only operations (Grep, Glob) → never require confirmation

### Step 6: Context Budget Allocation

The AI allocates token budget for conversation context.

**4 Budget Levels**:

```
Minimal (4K)   - Trivial/simple tasks, chat
Relevant (16K) - Moderate tasks, review
Standard (50K) - Complex tasks, build
Comprehensive (100K) - Heavy tasks, planning
```

**Allocation**:
- Trivial → Minimal
- Simple → Minimal
- Moderate → Relevant
- Complex → Standard
- Heavy → Comprehensive

**Multiplies**:
- Chat mode: Always Minimal
- Plan mode: Relevant or Standard
- Build mode: Based on complexity
- Review mode: Relevant or Standard
- Debug mode: Relevant

### Step 7: Memory Policy Selection

The AI determines which memory to load.

**4 Memory Policies**:

```
None          - No memory loading, fresh start
Recent        - Last N modified files
Relevant      - Files matching intent patterns
Full          - All recent context
```

**Selection**:
- Trivial/Simple tasks → None
- Chat mode → None
- Plan mode → Recent
- Review/Debug → Relevant
- Complex/Heavy → Full

---

## End-to-End Execution Flow

### Example 1: Read File

```
User Input: "read src/main.rs"

Step 1: Intent Classification
  → Intent::Read (regex matches "read \S+")

Step 2: Complexity Estimation
  → Complexity::Simple (single read operation)

Step 3: Mode Selection
  → AgentMode::Chat (simple, conversational)

Step 4: Model Tier Selection
  → ModelTier::Fast (simple task)

Step 5: Tool Policy
  → Allowed: Read, Grep, Glob
  → Confirmation: Never

Step 6: Context Budget
  → ContextBudget::Minimal (4K tokens)

Step 7: Memory Policy
  → MemoryPolicy::None (trivial task)

Execution:
  → Call Read tool
  → Display file contents
  → Done
```

### Example 2: Write File

```
User Input: "write new_feature.rs with content"

Step 1: Intent Classification
  → Intent::Write (regex matches "write \S+")

Step 2: Complexity Estimation
  → Complexity::Moderate (write operation)

Step 3: Mode Selection
  → AgentMode::Build (write operations)

Step 4: Model Tier Selection
  → ModelTier::Standard (moderate complexity)

Step 5: Tool Policy
  → Allowed: All tools
  → Confirmation: Required (Write)

Step 6: Context Budget
  → ContextBudget::Standard (50K tokens)

Step 7: Memory Policy
  → MemoryPolicy::Full (complex task)

Execution:
  → Show confirmation: "This will create new_feature.rs. Continue?"
  → User confirms
  → Call Write tool
  → Write file
  → Report success
  → Done
```

### Example 3: Code Review

```
User Input: "review src/main.rs"

Step 1: Intent Classification
  → Intent::Review (regex matches "review \S+")

Step 2: Complexity Estimation
  → Complexity::Moderate to Complex (depends on file size)

Step 3: Mode Selection
  → AgentMode::Review (review operation)

Step 4: Model Tier Selection
  → ModelTier::Standard (moderate to complex)

Step 5: Tool Policy
  → Allowed: Read, Grep, Glob
  → Confirmation: Never

Step 6: Context Budget
  → ContextBudget::Standard (50K tokens)

Step 7: Memory Policy
  → MemoryPolicy::Relevant (code review)

Execution:
  → Read src/main.rs
  → Grep for potential issues (patterns: error handling, unsafe, etc.)
  → Analyze code structure
  → Output review report
  → Done
```

### Example 4: Debug Task

```
User Input: "debug the segfault in src/main.rs"

Step 1: Intent Classification
  → Intent::Debug (regex matches "debug \S+")

Step 2: Complexity Estimation
  → Complexity::Complex (debugging is complex)

Step 3: Mode Selection
  → AgentMode::Debug (debug operation)

Step 4: Model Tier Selection
  → ModelTier::Standard (complex task)

Step 5: Tool Policy
  → Allowed: Read, Grep, Glob, Bash
  → Confirmation: Required (Bash)

Step 6: Context Budget
  → ContextBudget::Standard (50K tokens)

Step 7: Memory Policy
  → MemoryPolicy::Relevant (debugging)

Execution:
  → Read src/main.rs
  → Grep for segfault patterns
  → Ask confirmation for "bash cargo test -- --nocapture" or similar
  → User confirms
  → Execute bash command
  → Analyze output
  → Suggest fix
  → Done
```

### Example 5: Planning

```
User Input: "plan a microservices architecture for this project"

Step 1: Intent Classification
  → Intent::Plan (regex matches "plan \S+")

Step 2: Complexity Estimation
  → Complexity::Heavy (architecture planning is very complex)

Step 3: Mode Selection
  → AgentMode::Plan (planning operation)

Step 4: Model Tier Selection
  → ModelTier::Capable (heavy complexity)

Step 5: Tool Policy
  → Allowed: Read, Grep, Glob
  → Confirmation: Never

Step 6: Context Budget
  → ContextBudget::Comprehensive (100K tokens)

Step 7: Memory Policy
  → MemoryPolicy::Full (comprehensive planning)

Execution:
  → Read multiple files to understand structure
  → Glob to find all Rust files
  → Grep for dependencies and frameworks
  → Plan mode instruction: "Analyze and plan. Do NOT execute. Read-only."
  → Generate comprehensive architecture plan
  → Done
```

---

## AI Behavior Guidelines

### In Chat Mode

**Behavior**:
- Answer directly and concisely
- Suggest tools only if needed
- Minimal context usage
- Fast responses
- Low confidence routing

**Example**:
```
User: "What is a mutex?"
AI: "A mutex (mutual exclusion) is a synchronization primitive that..."
```

**Tools**:
- Only Read tool (if needed for context)

**No Execution**:
- Never call Write, Bash, Grep, Glob tools
- Never suggest file modifications

### In Plan Mode

**Behavior**:
- Analyze thoroughly
- Structure output (Analysis → Approach → Steps → Risks)
- Read-only operations
- No file modifications
- Use full context for analysis

**Example**:
```
User: "How should we refactor this authentication module?"
AI: "Analysis:
- Current module has 3 responsibilities (password, OAuth, JWT)
- Too tightly coupled with user service
- No test coverage

Approach:
1. Extract password handling to PasswordManager
2. Extract OAuth to OAuthService
3. Keep JWT as gateway between them

Steps:
1. Create PasswordManager trait and implementation
2. Refactor existing code to use PasswordManager
3. Write tests for PasswordManager
4. Repeat for OAuth

Risks:
- Breaking existing authentication flow
- Need extensive testing
```

**Tools**:
- Read, Grep, Glob only
- No Write, Bash

### In Build Mode

**Behavior**:
- Implement changes
- Verify results
- Report progress
- Execute shell commands
- Modify files
- **Require confirmation** for destructive operations

**Example**:
```
User: "Add error handling to the fetch function"

AI: "I'll add error handling. This requires modifying src/main.rs.
I'll use the Result type and handle the error case.
Confirming Write operation..."

AI: "Creating modified src/main.rs..."

AI: "Modifications complete. Here's what changed:
- Wrapped fetch function with Result<T, Error>
- Added error handling with error! macro
- Added test to verify error handling
"
```

**Tools**:
- All tools allowed
- Write requires confirmation
- Bash requires confirmation
- Git requires confirmation

### In Review Mode

**Behavior**:
- Review code critically
- Structure output (Summary → Issues → Suggestions)
- Read-only operations
- No modifications
- Focus on quality, bugs, security

**Example**:
```
User: "Review src/main.rs"

AI: "Summary:
File has 237 lines, uses Vec for all collections.

Issues:
1. Line 45: Potential overflow in increment operation
2. Line 78: Missing null check for API response
3. Line 112: Magic number (1000) not documented

Suggestions:
1. Use checked_add for overflow-safe arithmetic
2. Add Option type for nullable values
3. Extract magic number to constant with documentation
```

**Tools**:
- Read, Grep, Glob only
- No Write, Bash

### In Debug Mode

**Behavior**:
- Investigate systematically
- Find root cause
- Suggest fixes
- Execute diagnostic commands
- **Require confirmation** for commands

**Example**:
```
User: "Why is this test failing?"

AI: "Investigating test failure...

Running cargo test -- --nocapture...

Output:
thread 'test_example' failed at line 23: assertion failed: `a == b`
expected: 42
  found: 0

Root cause:
- `a` is initialized to 0 (default value)
- `b` is set via environment variable (not provided)
- Test doesn't handle missing environment variable

Fix:
Add fallback value for environment variable, or use #[env_var]
macro from the env_logger crate.
```

**Tools**:
- Read, Grep, Glob, Bash (with confirmation)
- No Write (read-only investigation)

---

## Decision Flowchart

```
User Prompt
      │
      ▼
┌─────────────────────┐
│ Intent Classification│ (Regex, <1ms)
│ 16 intent types     │
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ Complexity Estimation│ (Keywords, <1ms)
│ 5 levels            │
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ Mode Selection      │ (5 modes)
│ Chat/Plan/Build     │
│ Review/Debug        │
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ Model Tier Selection│ (5 tiers)
│ Local/Fast/Standard │
│ Capable             │
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ Tool Policy         │ (6 tools)
│ Allowed/Disallowed  │
│ Confirmation?       │
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ Context Budget      │ (4 levels)
│ 4K/16K/50K/100K     │
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ Memory Policy       │ (4 policies)
│ None/Recent/Relevant│
│ Full                │
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ Tool Execution      │ (with policies)
│ Read/Write/Bash/etc │
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ Response Generation │
│ AI generates output  │
└─────────────────────┘
```

---

## Error Handling

### Provider Failures

If a provider fails:

1. Log the error
2. Try next available provider (if configured)
3. Fall back to local if available
4. If no provider available, return error to user

### Tool Failures

If a tool fails:

1. Log error with context
2. Inform user
3. Suggest alternative approach
4. Never crash the application

### Insufficient Context

If context budget is exceeded:

1. Truncate context (keep most relevant)
2. Notify user
3. Prioritize important information

### Invalid Mode Transitions

If user tries invalid mode transition:

1. Reject transition
2. Suggest valid alternatives
3. Log transition attempt

---

## Performance Considerations

### Router Performance

- Intent classification: < 1ms
- Complexity estimation: < 1ms
- All routing decisions: < 5ms total

### Tool Performance

- Read: Depends on file size
- Write: Depends on file size
- Bash: Depends on command execution time
- Grep: O(n) where n is file size
- Glob: O(n) where n is directory size

### Context Management

- Memory policy affects response time
- Full memory can slow down on large projects
- Relevant memory optimizes for current task

---

## Best Practices

### For AI Agents

1. **Always check tool policy** before executing
2. **Request confirmation** for destructive operations
3. **Report progress** during long operations
4. **Log failures** for debugging
5. **Use appropriate context** - don't overload with irrelevant info
6. **Follow mode instructions** strictly
7. **Document your reasoning** for complex tasks

### For Users

1. **Be specific** in prompts for better intent classification
2. **Ask for planning** before large changes (use Plan mode)
3. **Review suggested changes** before accepting (Review mode)
4. **Confirm destructive operations** (Build mode)
5. **Use chat for quick questions** (Chat mode)
6. **Debug systematically** (Debug mode)

---

## System Prompts

### Base System Prompt

```
You are Quantum Code, a local-first AI coding assistant.
You help developers write better code through intelligent assistance.
You have access to tools for reading, writing, searching, and executing.
You operate in different modes (Chat/Plan/Build/Review/Debug).
Each mode affects your tools, behavior, and execution.
```

### Mode-Specific Instructions

**Chat Mode**:
```
Answer directly. Suggest tools only if needed.
Use minimal context.
Be concise and conversational.
```

**Plan Mode**:
```
Analyze and plan. Do NOT execute. Read-only.
Structure your output with clear sections.
Focus on understanding before implementation.
```

**Build Mode**:
```
Implement changes. Verify. Report progress.
Require confirmation for destructive operations.
Report all changes made to files.
```

**Review Mode**:
```
Review code. Report issues and suggestions.
Be critical and thorough.
Focus on bugs, security, and quality.
```

**Debug Mode**:
```
Investigate. Find root cause. Suggest fix.
Execute diagnostic commands with confirmation.
Be systematic in your investigation.
```

---

## Key Files for Understanding

- `src/router/analyzer.rs` - Intent classification and complexity
- `src/router/types.rs` - Type definitions and routing logic
- `src/router/mode.rs` - Mode selection and state machine
- `src/tools/mod.rs` - Tool definitions and policies
- `docs/MODE_SYSTEM.md` - Detailed mode documentation
- `context/ARCHITECTURE-context.md` - Project architecture
