# Trajectory Adjustment — Quantum Code

> **Date**: 2026-04-13
> **Purpose**: Correct Quantum Code's trajectory to align with the architecture described in `docs/` and the implementation blueprints in `research_*.md`.

---

## 1. Current State vs Intended State

### What Exists (Rust Implementation)

The current `src/` contains a **functional but shallow** CLI wrapper:

| Component | Status | Location |
|-----------|--------|----------|
| CLI parsing (clap) | ✅ Working | `src/cli.rs` |
| Multi-provider abstraction | ✅ Working | `src/providers/` |
| Bear-mode agent loop | ✅ Basic | `src/agent/executor.rs` |
| Tool execution (5 tools) | ✅ Working | `src/agent/tools.rs` |
| Mode prompts | ⚠️ Static strings, not wired | `src/prompts/modes.rs` |
| Model supervisor (llama.cpp) | ✅ Working | `src/supervisor/model_supervisor.rs` |
| TUI | ⚠️ Present | `src/tui/` |
| Config system | ⚠️ Minimal | `src/config/` |

### What's Missing (Per Research Documents)

The research documents describe a **7-layer intelligent router** and a **token-efficient quantum coding assistant**. None of it is implemented:

| Missing Component | Research Location | Impact |
|-------------------|-------------------|--------|
| **Intent classification** (16 intents, regex-based) | Research 02 §3 | Router cannot determine task type |
| **Complexity estimation** (5-level weighted scoring) | Research 02 §4 | Router cannot gauge difficulty |
| **Mode state machine** (5 modes, transitions, context preservation) | Research 02 §5 | Modes are decorative strings only |
| **Model tier selection** (local/fast/standard/capable) | Research 02 §6 | Always uses configured model, no adaptation |
| **Tool policy engine** (per-intent allowed/disallowed tools) | Research 02 §7 | All 5 tools always available |
| **Context strategy** (token budgets per complexity) | Research 02 §8 | No budget enforcement |
| **Memory system** (QUANTUM.md, relevance scoring) | Research 03 §2 | No project/user memory |
| **Token budget tracking** | Research 02 §2 | No tracking |
| **Parallel tool execution** | Research 02 §9 | Tools execute sequentially |
| **Rust acceleration** (`src/rust/`) | Research 01 §4 | No Rust code exists |
| **Minimal skill system** | Research 03 §4 | No skill loader |
| **Compressed tool schemas** (~120 tokens for 8 tools) | Research 03 §6 | Tool descriptions are verbose |
| **Optimized system prompts** (target: ~600 tokens) | Research 03 §6 | Current prompts are ~400+ tokens of prose |

---

## 2. Root Cause

The current implementation built a **CLI interface with a dumb agent loop** instead of the **intelligent router-driven system** described in the research documents.

The research was written to document the **original Claude Code TypeScript source** and define **how to build Quantum Code properly**. The current Rust implementation ignored the router architecture and went straight to building a Bear-mode chat wrapper.

**Evidence**:
- `src/agent/executor.rs` has a 50-iteration loop that executes tools blindly
- `src/prompts/modes.rs` has static prompt strings that are **not injected dynamically based on mode**
- `src/commands/agent.rs` calls `run_agentic()` without any routing decision
- No `src/router/` directory exists
- No `src/memory/` directory exists
- No `src/context.rs` equivalent exists

---

## 3. What Needs to Change

### 3.1 Architecture — Add the Router Module

**Reference**: Research 02 §11 (Minimal Router, 8 files, ~3K lines)

Create `src/router/` with:

```
src/router/
├── types.rs         # RoutingDecision, Intent, Complexity, Mode, ModelTier, etc.
├── router.rs        # route() function — orchestrates all 7 layers
├── analyzer.rs      # Intent classification (regex) + complexity scoring
├── mode.rs          # Mode state machine (chat/plan/build/review/debug)
├── model.rs         # Model tier selection (escalation/de-escalation)
├── tools.rs         # Tool policy (per-intent allowed/disallowed)
├── context.rs       # Context budget allocation (minimal/relevant/standard/comprehensive)
├── memory.rs        # Memory strategy (project/user/session relevance scoring)
└── index.rs         # Public exports
```

**Key design principles from Research 02 §11**:
- **Pure functions, no singletons** — `route(prompt: &str, cwd: &Path) -> RoutingDecision`
- **Regex-based intent classification** — no LLM call needed, < 1ms
- **No class hierarchy** — flat types + functions
- **Token budget on the RoutingDecision** — `context_budget: ContextBudget`

### 3.2 Integration — Wire Router Into Agent Loop

**Reference**: Research 04 §2 (Router ↔ QueryEngine Integration Gap)

The `AgentExecutor::run()` loop needs to consult the router **before each LLM call**:

```rust
// BEFORE (current):
let response = self.get_ai_response(provider).await;

// AFTER (correct):
let decision = router::route(&user_message, &cwd);
let system_prompt = build_system_prompt(decision, settings);
let filtered_tools = filter_tools_by_policy(get_tools(), &decision.tool_policy);
let response = self.get_ai_response_with(provider, system_prompt, filtered_tools).await;
```

**Integration points needed**:
1. Pre-query routing: `router::route()` before every LLM call
2. System prompt assembly: Mode-specific prompt from `router::route().mode`
3. Tool filtering: Only offer tools allowed by `decision.tool_policy`
4. Context budgeting: Trim conversation history to `decision.context_budget.max_tokens`
5. Mode transitions: User can switch modes, router validates transitions

### 3.3 Mode System — Make It Functional

**Reference**: Research 02 §5 (Mode Management)

Current `src/prompts/modes.rs` has static strings. They need to be **selected dynamically**:

```rust
// Current (wrong): static string always injected
const SYSTEM_PROMPT = format!("{}\n{}", CORE_IDENTITY, PLAN_MODE_PROMPT);

// Correct: router selects the mode, prompt is injected accordingly
let mode_prompt = match decision.mode {
    Mode::Plan => PLAN_MODE_PROMPT,
    Mode::Build => BUILD_MODE_PROMPT,
    Mode::Chat => CHAT_MODE_PROMPT,
    // ...
};
```

Mode transitions must be **stateful** — switching from `plan` to `build` preserves key files and findings per the preservation rules in Research 02 §5.

### 3.4 Tool System — Compress Schemas

**Reference**: Research 03 §6 (Compressed Tool Schemas)

Current tool descriptions are verbose strings. Compress to ~15 tokens each:

```rust
// Current (wasteful):
Tool { name: "Read".to_string(), description: "Read file contents. Arg: file path".to_string() }

// Target (compressed, ~15 tokens):
Tool { name: "read", description: "path:string → file contents" }
```

This reduces 5 tool schemas from ~200 tokens to ~75 tokens.

### 3.5 System Prompts — Target ~600 Tokens Total

**Reference**: Research 03 §6 (Token Budget Summary)

| Component | Current | Target |
|-----------|---------|--------|
| Base identity | ~200 | ~50 |
| Mode instruction | ~80 | ~30 |
| Tool schemas (5 tools) | ~200 | ~75 |
| Skill manifest | 0 | ~100 |
| Git context | 0 | ~100 |
| **Total** | **~480** | **~355** |

Current `CORE_IDENTITY` is ~160 tokens of prose. It should be restructured as:

```
IDENTITY: Quantum — local-first coding assistant
MODE: {mode} ({mode_instruction})
TOOLS: read, write, bash, grep, glob
```

### 3.6 Memory System — Add QUANTUM.md

**Reference**: Research 03 §4 (Minimal Skill System) + Research 04 Phase B2

Create `src/memory/` with:
- `ProjectMemory` — reads `.quantum/QUANTUM.md` at session start
- `UserMemory` — reads `~/.quantum/CLAUDE.md`
- `SessionMemory` — key decisions carried between turns within a session
- `RelevanceFilter` — keyword-scored memory loading (not blind loading)

### 3.7 Rust Module — Add Hot-Path Acceleration

**Reference**: Research 01 §4 + Research 02 §10

Create `rust/` workspace with NAPI-RS bindings for:
- `analyze_prompt()` — regex-based intent classification in Rust (40x faster than TS)
- `estimate_tokens()` — fast token counting
- `glob_search()` — file pattern search
- `grep_search()` — content search with regex

**Important**: The Rust module is **optional**. All functions must have pure-Rust fallbacks.

### 3.8 Skill System — Minimal Loader

**Reference**: Research 03 §4 (Minimal Skill Format)

```rust
// .quantum/skills/commit/SKILL.md
---
name: commit
desc: Git commit with generated message
tools: [bash]
---
Stage check → message → commit. Follow conventional commits.
```

Skill loader should be < 100 lines, single directory scan, frontmatter parsed on startup, body lazy-loaded on invoke.

---

## 4. What to Keep

These parts of the current implementation are correct and should be preserved:

| Component | Why Keep |
|-----------|----------|
| `src/providers/` | Multi-provider abstraction (Anthropic, OpenAI, Ollama, LlamaCpp, LmStudio) is correct architecture |
| `src/cli.rs` | Clap-based CLI is appropriate for a Rust CLI tool |
| `src/supervisor/model_supervisor.rs` | llama.cpp server management is well-implemented |
| `src/agent/tools.rs` | The 5 Bear tools (Read, Write, Bash, Grep, Glob) are the right core set |
| `src/agent/executor.rs` | Tool-call loop structure is correct — just needs router integration |
| `src/config/` | Config system is appropriate |

---

## 5. What to Drop or Redesign

| Component | Action | Why |
|-----------|--------|-----|
| `src/tui/` | Redesign | Research says "start with plain stdout" — Ink/React adds complexity without value |
| `src/prompts/modes.rs` | Redesign | Static strings aren't integrated with router; make dynamic |
| `src/prompts/system.rs` | Redesign | Too verbose; compress to target ~50 tokens |
| Provider implementations | Keep but simplify | Some may be over-engineered; audit after router is in |

---

## 6. Implementation Priority

### Phase 1: Router Core (Week 1)
1. Create `src/router/types.rs` — all routing types
2. Create `src/router/analyzer.rs` — intent classification + complexity scoring
3. Create `src/router/router.rs` — `route()` function
4. Create `src/router/mode.rs` — mode state machine
5. Create `src/router/tools.rs` — tool policy engine
6. Create `src/router/context.rs` — context budget allocation
7. Create `src/router/memory.rs` — memory relevance scoring
8. Wire router into `AgentExecutor::run()`

### Phase 2: Prompt Optimization (Week 1-2)
1. Compress `CORE_IDENTITY` to ~50 tokens
2. Make mode prompt selection dynamic via router decision
3. Compress tool schemas to ~15 tokens each
4. Add minimal skill loader (~100 lines)

### Phase 3: Memory & Context (Week 2)
1. Implement QUANTUM.md reading
2. Implement relevance-based memory loading
3. Wire context budget into conversation history trimming
4. Add session memory for multi-turn state

### Phase 4: Rust Acceleration (Week 2-3)
1. Create `rust/` Cargo workspace
2. Port `analyze_prompt()` to Rust with RegexSet
3. Add `estimate_tokens()` in Rust
4. Wire NAPI-RS bindings with JS fallbacks

### Phase 5: Polish (Week 3-4)
1. Add mode persistence (state survives restarts)
2. Implement mode transitions with context preservation
3. Add slash commands: `/mode`, `/skills`, `/compact`
4. Simplify TUI or replace with stdout

---

## 7. Key Metrics Targets

| Metric | Current | Target | Reference |
|--------|---------|--------|-----------|
| System prompt tokens | ~480 | < 600 | Research 03 §6 |
| Router latency | N/A (not exists) | < 1ms | Research 02 §3 |
| Tool schemas (5 tools) | ~200 tokens | ~75 tokens | Research 03 §6 |
| Memory overhead | 0 (no memory) | < 200 tokens loaded | Research 04 Phase B2 |
| Core router LOC | 0 | ~3,000 lines | Research 02 §11 |

---

## 8. Summary

Quantum Code's current implementation is a **functional CLI with a naive agent loop**. The research documents describe an **intelligent router-driven system** that:

1. Classifies every task via regex in < 1ms
2. Selects appropriate mode, model tier, and tools per task
3. Manages token budgets to stay within context windows
4. Loads memory by relevance, not blindly
5. Runs at < 600 tokens of system overhead

**The fix is not to add features — it's to build the router and wire it in.**

The good news: the current tool system, provider abstraction, and agent loop structure are all correct foundations. The work is adding the intelligence layer on top, not rebuilding from scratch.
