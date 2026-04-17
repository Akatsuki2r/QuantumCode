# Mode System

## Overview

Quantum Code operates in different modes for different workflows. Each mode affects tool access, prompt shaping, model selection, and execution behavior.

**Status**: 90% Implemented

## Five Execution Modes

| Mode | Writes? | Tool Access | Model Tier | Use Case |
|------|---------|-------------|------------|----------|
| `Chat` | No | Minimal | Fast | Quick questions |
| `Plan` | No | Read-only | Standard | Architecture, planning |
| `Build` | Yes | Full | Standard/Capable | Implementation |
| `Review` | No | Read-only | Standard | Code review |
| `Debug` | Limited | Read + Bash | Standard | Debugging |

## Mode Definitions

**File**: `src/router/types.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentMode {
    /// Conversational - minimal tools, quick responses
    Chat,
    /// Planning - analysis only, no execution
    Plan,
    /// Building - full execution capabilities
    Build,
    /// Code review - read-only analysis
    Review,
    /// Debugging - diagnostic tools only
    Debug,
}
```

## Mode Instructions

Each mode has a system prompt instruction that shapes AI behavior:

**File**: `src/router/mode.rs`

```rust
impl AgentMode {
    pub fn instruction(&self) -> &'static str {
        match self {
            AgentMode::Chat => "Answer directly. Suggest tools only if needed.",
            AgentMode::Plan => "Analyze and plan. Do NOT execute. Read-only.",
            AgentMode::Build => "Implement changes. Verify. Report progress.",
            AgentMode::Review => "Review code. Report issues and suggestions.",
            AgentMode::Debug => "Investigate. Find root cause. Suggest fix.",
        }
    }
}
```

## Mode Selection

**File**: `src/router/mode.rs`

Mode is selected based on intent and complexity:

```rust
pub fn pick_mode(intent: Intent, complexity: Complexity) -> AgentMode {
    match intent {
        // Read operations - typically chat or review
        Intent::Read | Intent::Explain | Intent::Chat | Intent::Help => {
            if complexity >= Complexity::Complex {
                AgentMode::Review
            } else {
                AgentMode::Chat
            }
        }

        // Write operations - build mode
        Intent::Write | Intent::Edit => AgentMode::Build,

        // Delete operations - build with caution
        Intent::Delete => AgentMode::Build,

        // Shell operations - build mode
        Intent::Bash | Intent::Git => AgentMode::Build,

        // Search operations - review mode
        Intent::Grep | Intent::Glob | Intent::Find => AgentMode::Review,

        // Review operations - review mode
        Intent::Review => AgentMode::Review,

        // Debug operations - debug mode
        Intent::Debug => AgentMode::Debug,

        // Planning operations - plan mode
        Intent::Plan | Intent::Design => AgentMode::Plan,

        // Fallback
        Intent::Unknown => AgentMode::Chat,
    }
}
```

## Mode State Machine

### Valid Transitions

```
chat ──→ plan, build, debug
plan ──→ build, review, chat
build ──→ review, debug, plan, chat
review ──→ build, plan, chat
debug ──→ build, plan, chat
```

**File**: `src/router/types.rs`

```rust
impl AgentMode {
    pub fn can_transition_to(&self, target: AgentMode) -> bool {
        match (self, target) {
            // Forward transitions (typical workflow)
            (AgentMode::Chat, AgentMode::Plan) => true,
            (AgentMode::Chat, AgentMode::Build) => true,
            (AgentMode::Plan, AgentMode::Build) => true,
            // Backward transitions (replanning)
            (AgentMode::Build, AgentMode::Plan) => true,
            // Special cases
            (AgentMode::Chat, AgentMode::Review) => true,
            (AgentMode::Chat, AgentMode::Debug) => true,
            (AgentMode::Build, AgentMode::Debug) => true,
            // Same mode always allowed
            _ if *self == target => true,
            // All other transitions disallowed
            _ => false,
        }
    }
}
```

### Transition Function

```rust
pub fn transition(current: AgentMode, target: AgentMode) -> Option<AgentMode> {
    if can_transition(current, target) {
        Some(target)
    } else {
        None
    }
}
```

## Mode Characteristics

### Chat Mode

**Purpose**: Conversational assistance with minimal tool usage

**Characteristics**:
- Quick responses
- Minimal context
- No file modifications
- Suggests rather than executes

**Tool Policy**:
- Allowed: Read
- Disallowed: Write, Bash, Grep, Glob
- Confirmation: Not required

**Model Tier**: Fast (for quick responses)

**Context Budget**: Minimal (4K tokens)

**Memory Policy**: None

**Example Prompts**:
- "What is a mutex?"
- "Explain async/await in Rust"
- "How do I use generics?"

---

### Plan Mode

**Purpose**: Analysis and planning without execution

**Characteristics**:
- Structured output (Analysis → Approach → Steps → Risks)
- Read-only access
- No file modifications
- Focus on understanding

**Tool Policy**:
- Allowed: Read, Grep, Glob
- Disallowed: Write, Bash
- Confirmation: Not required

**Model Tier**: Standard (for better reasoning)

**Context Budget**: Relevant (16K tokens)

**Memory Policy**: Recent

**Example Prompts**:
- "Plan a microservices architecture"
- "How should I refactor this module?"
- "Design an authentication system"

---

### Build Mode

**Purpose**: Implementation and code changes

**Characteristics**:
- Full tool access
- Can modify files
- Can run commands
- Implementation-focused

**Tool Policy**:
- Allowed: Read, Write, Bash, Grep, Glob
- Disallowed: None
- Confirmation: Required for Delete, Bash, Git

**Model Tier**: Standard or Capable (depending on complexity)

**Context Budget**: Based on complexity

**Memory Policy**: Full (for complex tasks)

**Example Prompts**:
- "Add error handling to this function"
- "Create a new REST endpoint"
- "Fix the null pointer exception"

---

### Review Mode

**Purpose**: Code review and analysis

**Characteristics**:
- Read-only access
- Structured output (Summary → Issues → Suggestions)
- No modifications
- Critical analysis

**Tool Policy**:
- Allowed: Read, Grep, Glob
- Disallowed: Write, Bash
- Confirmation: Not required

**Model Tier**: Standard

**Context Budget**: Relevant or Standard

**Memory Policy**: Relevant

**Example Prompts**:
- "Review this pull request"
- "Check for security issues"
- "Find potential bugs"

---

### Debug Mode

**Purpose**: Debugging and root cause analysis

**Characteristics**:
- Read + limited execution
- Diagnostic focus
- Systematic investigation
- Fix suggestions

**Tool Policy**:
- Allowed: Read, Grep, Glob, Bash
- Disallowed: Write
- Confirmation: Required

**Model Tier**: Standard

**Context Budget**: Relevant

**Memory Policy**: Relevant

**Example Prompts**:
- "Debug this race condition"
- "Why is this test failing?"
- "Trace the request flow"

---

## Mode Persistence

**Status**: Not Implemented

Modes should persist across session restarts:

```rust
// TODO: Implement mode persistence
pub struct ModeState {
    current_mode: AgentMode,
    entered_at: u64,
    turn_count: u32,
    preserved_context: PreservedContext,
}

pub struct PreservedContext {
    key_files: Vec<String>,      // Max 20 files
    findings: Vec<String>,       // Max 10 findings
    task_description: String,
    decisions: Vec<String>,      // Max 10 decisions
}
```

---

## Mode Switching in TUI

**File**: `src/tui/widgets/tabs.rs`

Modes can be switched via the TUI:

```rust
// Slash commands in interactive mode
/mode plan    // Switch to plan mode
/mode build   // Switch to build mode
/mode chat    // Switch to chat mode
/mode review  // Switch to review mode
/mode debug   // Switch to debug mode
```

---

## Mode Effects on System Prompt

The mode instruction is injected into the system prompt:

```rust
pub fn build_system_prompt(mode: AgentMode, context: &str) -> String {
    format!(
        "{}\n\n{}\n\n{}",
        BASE_SYSTEM_PROMPT,
        mode.instruction(),
        context
    )
}
```

Example for Build mode:
```
You are Quantum Code, a local-first AI coding assistant.

Implement changes. Verify. Report progress.

## Context
[retrieved context here]
```

---

## Mode and Model Selection

Mode affects model tier selection:

**File**: `src/router/model.rs`

```rust
pub fn pick_model_tier(
    complexity: Complexity,
    intent: Intent,
    mode: AgentMode,
    config: &RouterConfig,
) -> ModelTier {
    // Build mode for complex work needs standard
    if mode == AgentMode::Build && complexity >= Complexity::Moderate {
        return ModelTier::Standard;
    }
    
    // ... rest of selection logic
}
```

---

## Mode and Context Budget

Mode affects context budget allocation:

**File**: `src/router/context.rs`

```rust
pub fn pick_budget(complexity: Complexity, mode: AgentMode) -> ContextBudget {
    let base = ContextBudget::from_complexity(complexity);

    match mode {
        // Chat needs minimal context
        AgentMode::Chat => ContextBudget::Minimal,

        // Plan needs moderate context for analysis
        AgentMode::Plan => ContextBudget::Relevant.max(base),

        // Review needs good context for understanding code
        AgentMode::Review => ContextBudget::Relevant.max(base),

        // Debug needs moderate context
        AgentMode::Debug => ContextBudget::Relevant.max(base),

        // Build needs full context for implementation
        AgentMode::Build => base,
    }
}
```

---

## Mode and Memory Policy

Mode affects memory loading:

**File**: `src/router/memory.rs`

```rust
pub fn pick_memory_policy(
    intent: Intent,
    complexity: Complexity,
    mode: AgentMode,
) -> MemoryPolicy {
    // Trivial tasks don't need memory
    if complexity == Complexity::Trivial {
        return MemoryPolicy::None;
    }

    // Simple chat doesn't need memory
    if mode == AgentMode::Chat && complexity <= Complexity::Simple {
        return MemoryPolicy::None;
    }

    // Planning benefits from recent context
    if mode == AgentMode::Plan {
        return MemoryPolicy::Recent;
    }

    // Review/debug need relevant context
    if mode == AgentMode::Review || mode == AgentMode::Debug {
        return MemoryPolicy::Relevant;
    }

    // Build mode needs full context for complex tasks
    if mode == AgentMode::Build && complexity >= Complexity::Complex {
        return MemoryPolicy::Full;
    }

    // Default to recent
    MemoryPolicy::Recent
}
```

---

## Testing

### Unit Tests

**File**: `src/router/mode.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pick_mode_read_operations() {
        assert_eq!(pick_mode(Intent::Read, Complexity::Simple), AgentMode::Chat);
        assert_eq!(pick_mode(Intent::Explain, Complexity::Simple), AgentMode::Chat);
        assert_eq!(pick_mode(Intent::Chat, Complexity::Simple), AgentMode::Chat);
    }

    #[test]
    fn test_pick_mode_write_operations() {
        assert_eq!(pick_mode(Intent::Write, Complexity::Simple), AgentMode::Build);
        assert_eq!(pick_mode(Intent::Edit, Complexity::Simple), AgentMode::Build);
    }

    #[test]
    fn test_mode_transitions() {
        assert!(can_transition(AgentMode::Chat, AgentMode::Plan));
        assert!(can_transition(AgentMode::Chat, AgentMode::Build));
        assert!(can_transition(AgentMode::Plan, AgentMode::Build));
        assert!(can_transition(AgentMode::Build, AgentMode::Plan));
        assert!(!can_transition(AgentMode::Plan, AgentMode::Debug));
    }

    #[test]
    fn test_mode_instructions() {
        assert!(!get_mode_instruction(AgentMode::Chat).is_empty());
        assert!(!get_mode_instruction(AgentMode::Plan).is_empty());
        assert!(!get_mode_instruction(AgentMode::Build).is_empty());
        assert!(!get_mode_instruction(AgentMode::Review).is_empty());
        assert!(!get_mode_instruction(AgentMode::Debug).is_empty());
    }
}
```

---

## Mode Workflow Example

Typical workflow progression:

```
1. User: "What is the structure of this project?"
   → Intent: Read/Explain
   → Mode: Chat (simple) or Review (complex)

2. User: "Plan a refactor to use Result types"
   → Intent: Plan
   → Mode: Plan

3. User: "Implement the plan"
   → Intent: Write
   → Mode: Build

4. User: "Review the changes"
   → Intent: Review
   → Mode: Review

5. User: "Why is this test failing?"
   → Intent: Debug
   → Mode: Debug
```

---

## Future Enhancements

### 1. Mode Persistence

Persist mode across session restarts:
- Save mode to session file
- Restore on load
- Preserve context on transition

### 2. Auto-Mode Detection

Automatically detect when mode should change:
- Detect planning language → suggest Plan mode
- Detect debugging → suggest Debug mode
- User can override

### 3. Mode-Specific UI

Different UI elements per mode:
- Plan mode: Outline/tree view
- Build mode: Diff viewer
- Review mode: Annotation view
- Debug mode: Stack trace view

### 4. Mode History

Track mode transitions:
- How long in each mode
- Common transition patterns
- Optimize suggestions
