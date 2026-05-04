# Router Implementation Plan

## Context

Quantum Code's current Rust implementation is a functional CLI with a naive 50-iteration agent loop. The research documents describe a **7-layer intelligent router** that classifies tasks via regex in <1ms and selects mode, model tier, tools, context budget, and memory policy per task.

**Root problem**: `AgentExecutor::run()` calls the LLM without any routing decision. The router doesn't exist.

---

## Phase 1: Router Module (`src/router/`)

### 1.1 Types (`src/router/types.rs`)

```rust
// Intent - 16 task types via regex classification
pub enum Intent {
    Read, Write, Edit, Delete,    // File ops
    Bash, Git,                      // Shell ops
    Grep, Glob, Find,              // Search ops
    Explain, Review, Debug,         // Analysis ops
    Plan, Design,                   // Planning ops
    Help, Chat,                     // Meta ops
    Unknown,
}

// Complexity - 5 levels
pub enum Complexity {
    Trivial, Simple, Moderate, Complex, Heavy,
}

// Mode - 5 execution modes
pub enum AgentMode {
    Chat,   // Minimal tools, conversational
    Plan,   // Read-only, analysis
    Build,  // Full execution
    Review, // Read-only analysis
    Debug,  // Diagnostic tools
}

// Model Tier - 4 capability levels
pub enum ModelTier {
    Local,    // Ollama/llama.cpp
    Fast,     // Haiku/mini
    Standard, // Sonnet/GPT-4o
    Capable,  // Opus/GPT-4
}

// Context Budget - 4 tiers (tokens)
pub enum ContextBudget {
    Minimal = 4_000,
    Relevant = 16_000,
    Standard = 50_000,
    Comprehensive = 100_000,
}

// Memory Policy
pub enum MemoryPolicy {
    None, Recent, Relevant, Full,
}

// RoutingDecision - complete output of route()
pub struct RoutingDecision {
    pub intent: Intent,
    pub complexity: Complexity,
    pub mode: AgentMode,
    pub model_tier: ModelTier,
    pub tools: ToolPolicy,
    pub context_budget: ContextBudget,
    pub memory_policy: MemoryPolicy,
    pub confidence: f32,
    pub reasoning: String,
}

// ToolPolicy - per-intent permissions
pub struct ToolPolicy {
    pub allowed_tools: Vec<String>,
    pub disallowed_tools: Vec<String>,
    pub require_confirmation: bool,
}

// RouterConfig
pub struct RouterConfig {
    pub prefer_local: bool,
    pub cost_limit: f32,
}
```

### 1.2 Analyzer (`src/router/analyzer.rs`)

Intent classification via `RegexSet` — matches ALL patterns in single pass, <1ms:

```rust
// Pattern order = priority (first match wins)
const INTENT_PATTERNS: &[(&str, Intent)] = &[
    (r"(?i)^(?:read|view|show|cat)\s+", Intent::Read),
    (r"(?i)^(?:write|create|new)\s+", Intent::Write),
    (r"(?i)^(?:edit|modify|update)\s+", Intent::Edit),
    (r"(?i)^(?:delete|remove|rm)\s+", Intent::Delete),
    (r"(?i)^(?:run|exec|bash|shell)\s+", Intent::Bash),
    (r"(?i)^(?:git|commit|push|pull)", Intent::Git),
    (r"(?i)^(?:grep|search|rg)\s+", Intent::Grep),
    (r"(?i)^(?:glob|find files)", Intent::Glob),
    (r"(?i)^(?:explain|what is|how does)", Intent::Explain),
    (r"(?i)^(?:review|check|analyze)\s+", Intent::Review),
    (r"(?i)^(?:debug|trace)\s+", Intent::Debug),
    (r"(?i)^(?:plan|decompose)\s+", Intent::Plan),
    (r"(?i)^(?:design|architect)\s+", Intent::Design),
    (r"(?i)^(?:help|\?|usage)", Intent::Help),
    (r"(?i)^(?:hi|hello|hey|chat)", Intent::Chat),
];

pub fn classify_intent(prompt: &str) -> Intent {
    let regex_set = RegexSet::new(INTENT_PATTERNS.iter().map(|(p, _)| p)).unwrap();
    let matches: Vec<usize> = regex_set.matches(prompt).into_iter().collect();
    matches.is_empty().then_some(Intent::Unknown)
        .or_else(|| INTENT_PATTERNS[*matches.first().unwrap()].1.into())
}
```

Complexity scoring via keyword weights:

```rust
const COMPLEXITY_KEYWORDS: &[(&str, i32)] = &[
    (r"(?i)\btrivia\b", -2),
    (r"(?i)\bread\b", 1),
    (r"(?i)\bwrite\b", 2),
    (r"(?i)\brefactor\b", 3),
    (r"(?i)\barchitect\b", 4),
    (r"(?i)\bsecurity\b", 4),
];

pub fn score_complexity(prompt: &str) -> Complexity {
    let score = COMPLEXITY_KEYWORDS.iter()
        .filter(|(p, _)| regex::Regex::new(p).unwrap().is_match(prompt))
        .map(|(_, w)| w)
        .sum::<i32>()
        .clamp(0, 4);
    match score { 0 => Trivial, 1 => Simple, 2 => Moderate, 3 => Complex, _ => Heavy }
}
```

### 1.3 Mode (`src/router/mode.rs`)

```rust
pub fn pick_mode(intent: Intent, complexity: Complexity) -> AgentMode {
    match intent {
        Intent::Read | Intent::Explain | Intent::Chat => AgentMode::Chat,
        Intent::Write | Intent::Edit | Intent::Delete | Intent::Bash | Intent::Git => AgentMode::Build,
        Intent::Review | Intent::Grep | Intent::Glob | Intent::Find => AgentMode::Review,
        Intent::Debug => AgentMode::Debug,
        Intent::Plan | Intent::Design => AgentMode::Plan,
        Intent::Help => AgentMode::Chat,
        Intent::Unknown => AgentMode::Chat,
    }
}
```

### 1.4 Model (`src/router/model.rs`)

```rust
pub fn pick_model_tier(complexity: Complexity, intent: Intent, config: &RouterConfig) -> ModelTier {
    match complexity {
        Complexity::Trivial | Complexity::Simple => {
            if config.prefer_local { ModelTier::Local } else { ModelTier::Fast }
        }
        Complexity::Moderate => ModelTier::Standard,
        Complexity::Complex | Complexity::Heavy => ModelTier::Capable,
    }
}
```

### 1.5 Tools (`src/router/tools.rs`)

```rust
pub fn pick_tools(intent: Intent, mode: AgentMode) -> ToolPolicy {
    match mode {
        AgentMode::Build => ToolPolicy::default(),
        AgentMode::Review | AgentMode::Plan => ToolPolicy::read_only(),
        AgentMode::Debug => ToolPolicy {
            allowed_tools: vec!["Read", "Grep", "Glob", "Bash"],
            disallowed_tools: vec!["Write"],
            require_confirmation: false,
        },
        AgentMode::Chat => ToolPolicy {
            allowed_tools: vec!["Read"],
            disallowed_tools: vec!["Write", "Bash"],
            require_confirmation: false,
        },
    }
}
```

### 1.6 Context (`src/router/context.rs`)

```rust
pub fn pick_budget(complexity: Complexity, mode: AgentMode) -> ContextBudget {
    match complexity {
        Complexity::Trivial | Complexity::Simple => ContextBudget::Minimal,
        Complexity::Moderate => ContextBudget::Relevant,
        Complexity::Complex => ContextBudget::Standard,
        Complexity::Heavy => ContextBudget::Comprehensive,
    }
}
```

### 1.7 Memory (`src/router/memory.rs`)

```rust
pub fn pick_memory_policy(intent: Intent, complexity: Complexity, mode: AgentMode) -> MemoryPolicy {
    if complexity == Complexity::Trivial { return MemoryPolicy::None; }
    match mode {
        AgentMode::Chat if complexity <= Complexity::Simple => MemoryPolicy::None,
        AgentMode::Plan => MemoryPolicy::Recent,
        AgentMode::Review | AgentMode::Debug => MemoryPolicy::Relevant,
        AgentMode::Build if complexity >= Complexity::Complex => MemoryPolicy::Full,
        _ => MemoryPolicy::Recent,
    }
}
```

### 1.8 Router Module (`src/router/mod.rs`)

```rust
pub fn route(prompt: &str, cwd: &str, config: &RouterConfig) -> RoutingDecision {
    let intent = classify_intent(prompt);
    let complexity = score_complexity(prompt);
    let mode = pick_mode(intent, complexity);
    let model_tier = pick_model_tier(complexity, intent, config);
    let tools = pick_tools(intent, mode);
    let context_budget = pick_budget(complexity, mode);
    let memory_policy = pick_memory_policy(intent, complexity, mode);

    RoutingDecision {
        intent,
        complexity,
        mode,
        model_tier,
        tools,
        context_budget,
        memory_policy,
        confidence: 0.85,
        reasoning: format!("{:?}/{:?}/{:?}", intent, mode, complexity),
    }
}
```

---

## Phase 2: Integration into AgentExecutor

Modify `src/agent/executor.rs`:

1. Add `use crate::router::{route, RouterConfig, RoutingDecision};`
2. Add `routing: Option<RoutingDecision>` field to `AgentExecutor`
3. Before first LLM call: `self.routing = Some(route(&user_msg, ".", &RouterConfig::default()));`
4. Filter tool calls: `tool_calls.retain(|c| routing.tools.allowed_tools.iter().any(|t| t.to_lowercase() == c.name.to_lowercase()));`

---

## Phase 3: Compress System Prompts

Update `src/prompts/system.rs`:

```
IDENTITY: QC — local-first coding assistant. Read/write/edit files, shell, analyze, search.
MODE: {mode}
TOOLS: read(file), write(file,content), bash(cmd), grep(pattern,path?), glob(pattern)
SKILLS: review, refactor, debug, test, git, docs
GIT: Never force push. Preserve history. Clear messages.
```

Target: ~400-500 tokens total.

---

## Phase 4: Dependencies

Add to `Cargo.toml`:
```toml
regex = "1.10"
```

---

## File List

### Create
- `src/router/mod.rs`
- `src/router/types.rs`
- `src/router/analyzer.rs`
- `src/router/mode.rs`
- `src/router/model.rs`
- `src/router/tools.rs`
- `src/router/context.rs`
- `src/router/memory.rs`

### Modify
- `src/agent/executor.rs` — wire router in
- `src/agent/mod.rs` — re-export router
- `src/main.rs` — add `mod router;`
- `src/prompts/system.rs` — compress prompts
- `Cargo.toml` — add regex

---

## Verification

1. `cargo build` — no compilation errors
2. `cargo test router` — unit tests pass
3. `cargo run -- agent "read src/main.rs"` — verify routing logs
4. Profile `route()` — verify <1ms latency
