# Router Deep Dive

## Overview

The Quantum Code Router is a **7-layer policy engine** for routing user prompts to appropriate execution contexts. It's designed as pure functions with no side effects, making it highly testable and explainable.

## Router Philosophy

The router is NOT a simple switchboard. It makes layered decisions that cascade through seven stages:

```
User Prompt
    │
    ▼
┌─────────────────┐
│ 1. Intent        │  What kind of task? (read, write, explain, plan...)
│    Classification│  16 intents via regex, < 1ms
└────────┬────────┘
         ▼
┌─────────────────┐
│ 2. Complexity    │  How hard? (trivial → simple → moderate → complex → heavy)
│    Estimation    │  Keyword-weighted scoring
└────────┬────────┘
         ▼
┌─────────────────┐
│ 3. Mode          │  What execution mode? (chat/plan/build/review/debug)
│    Selection     │  5 modes with state machine
└────────┬────────┘
         ▼
┌─────────────────┐
│ 4. Model Tier    │  What capability level? (local/fast/standard/capable)
│    Selection     │  4 tiers based on complexity + intent
└────────┬────────┘
         ▼
┌─────────────────┐
│ 5. Tool Policy   │  Which tools? (allowed, disallowed, confirmation)
│    Determination │  Per-intent tool permissions
└────────┬────────┘
         ▼
┌─────────────────┐
│ 6. Context       │  How much context to load? (4K → 16K → 50K → 100K tokens)
│    Strategy      │  Based on complexity + mode
└────────┬────────┘
         ▼
┌─────────────────┐
│ 7. Memory Policy │  What memory to load? (none/recent/relevant/full)
└────────┬────────┘
         ▼
    RoutingDecision
```

## Layer 1: Intent Classification

**File**: `src/router/analyzer.rs`

**Purpose**: Classify the user's intent from their prompt using regex pattern matching.

**Performance**: < 1ms (compiled regex via `lazy_static!`)

### 16 Intents

| Category | Intents |
|----------|---------|
| File Operations | `Read`, `Write`, `Edit`, `Delete` |
| Shell Operations | `Bash`, `Git` |
| Search Operations | `Grep`, `Glob`, `Find` |
| Analysis Operations | `Explain`, `Review`, `Debug` |
| Planning Operations | `Plan`, `Design` |
| Meta Operations | `Help`, `Chat`, `Unknown` |

### Pattern Matching

Patterns are ordered by priority (first match wins):

```rust
const INTENT_PATTERNS: &[(&str, Intent)] = &[
    // File operations - checked first
    (r"^(?i)(?:read|view|show|cat|open|get)\s+\S+", Intent::Read),
    (r"^(?i)(?:write|create|new|touch)\s+\S+", Intent::Write),
    (r"^(?i)(?:edit|modify|update|change)\s+\S+", Intent::Edit),
    (r"^(?i)(?:delete|remove|rm|del|unlink)\s+\S+", Intent::Delete),
    
    // Shell operations
    (r"^(?i)(?:run|exec|execute|bash|shell|cmd|sh)\s+", Intent::Bash),
    (r"^(?i)(?:git|commit|push|pull|branch|merge|checkout|clone)\s*", Intent::Git),
    
    // Search operations
    (r"^(?i)(?:grep|rg|search|rip)\s+", Intent::Grep),
    (r"^(?i)(?:glob|find files)", Intent::Glob),
    (r"^(?i)(?:find|locate)\s+(?:file|path)", Intent::Find),
    
    // Analysis operations
    (r"^(?i)(?:explain|what is|how does|tell me about|describe)\s+", Intent::Explain),
    (r"^(?i)(?:review|check|analyze|audit)\s+", Intent::Review),
    (r"^(?i)(?:debug|debugger|breakpoint|trace|inspect)\s+", Intent::Debug),
    
    // Planning operations
    (r"^(?i)(?:plan|design|architecture|decompose)\s+", Intent::Plan),
    (r"^(?i)(?:design|architect|blueprint)\s+", Intent::Design),
    
    // Meta operations
    (r"^(?i)(?:help|\?|usage|commands|man)\s*$", Intent::Help),
    (r"^(?i)(?:hi|hello|hey|howdy|sup)\s*$", Intent::Chat),
    (r"^(?i)(?:thanks|thank you|thx)\s*$", Intent::Chat),
];
```

### Implementation

```rust
lazy_static! {
    static ref INTENT_REGEX_SET: RegexSet = RegexSet::new(
        INTENT_PATTERNS.iter().map(|(p, _)| *p)
    ).expect("Failed to compile intent patterns");
}

pub fn classify_intent(prompt: &str) -> Intent {
    let prompt = prompt.trim();
    if prompt.is_empty() {
        return Intent::Unknown;
    }

    let matches: Vec<usize> = INTENT_REGEX_SET.matches(prompt).into_iter().collect();

    if matches.is_empty() {
        return Intent::Unknown;
    }

    // Return the first (highest priority) matching intent
    INTENT_PATTERNS[matches[0]].1
}
```

## Layer 2: Complexity Estimation

**File**: `src/router/analyzer.rs`

**Purpose**: Estimate task difficulty using keyword-weighted scoring.

### 5 Complexity Levels

| Level | Score | Description |
|-------|-------|-------------|
| `Trivial` | 0 | System commands, quick lookups |
| `Simple` | 1 | Single file read, basic questions |
| `Moderate` | 2 | Write/edit operations, single function |
| `Complex` | 3 | Refactoring, multi-file changes |
| `Heavy` | 4 | Architecture, security, ML, full-stack |

### Complexity Keywords

```rust
const COMPLEXITY_KEYWORDS: &[(&str, i32)] = &[
    // Trivial indicators (negative = simpler)
    (r"(?i)\b(?:ls|dir|pwd|whoami|date|echo)\s*$", -3),
    (r"(?i)\b(?:trivia|quick|simple|easy|just)\s*$", -2),
    
    // Simple indicators
    (r"(?i)\b(?:read|view|show|get|list)\b", 1),
    (r"(?i)\b(?:file|path|dir|directory)\b", 1),
    
    // Moderate indicators
    (r"(?i)\b(?:write|create|edit|modify|update)\b", 2),
    (r"(?i)\b(?:function|method|class|module|struct|enum|trait)\b", 2),
    (r"(?i)\b(?:test|spec|assert|expect)\b", 2),
    
    // Complex indicators
    (r"(?i)\b(?:refactor|optimize|migrate|port|convert)\b", 3),
    (r"(?i)\b(?:algorithm|data structure|performance|cache|concurrency)\b", 3),
    (r"(?i)\b(?:api|rest|graphql|protocol|network)\b", 3),
    
    // Heavy indicators
    (r"(?i)\b(?:security|authentication|authorization|encryption)\b", 4),
    (r"(?i)\b(?:architecture|microservice|distributed|system design)\b", 4),
    (r"(?i)\b(?:machine learning|ai|llm|neural|transformer)\b", 4),
    (r"(?i)\b(?:full.stack|multi.platform|integration)\b", 4),
];
```

### Scoring Algorithm

```rust
pub fn score_complexity(prompt: &str) -> Complexity {
    let prompt = prompt.trim();
    if prompt.is_empty() {
        return Complexity::Simple;
    }

    let matches: Vec<usize> = COMPLEXITY_REGEX_SET.matches(prompt).into_iter().collect();

    // Sum weights of all matched patterns
    let score: i32 = matches.iter().map(|idx| COMPLEXITY_KEYWORDS[*idx].1).sum();

    // Clamp to 0-4 range
    let score = score.clamp(0, 4);

    match score {
        0 => Complexity::Trivial,
        1 => Complexity::Simple,
        2 => Complexity::Moderate,
        3 => Complexity::Complex,
        _ => Complexity::Heavy,
    }
}
```

## Layer 3: Mode Selection

**File**: `src/router/mode.rs`

**Purpose**: Select the appropriate execution mode based on intent and complexity.

### 5 Execution Modes

| Mode | Writes? | Tool Access | Prompt Shape | Use Case |
|------|---------|-------------|--------------|----------|
| `Chat` | No | Minimal | Concise | Quick questions |
| `Plan` | No | Read-only | Structured | Architecture, planning |
| `Build` | Yes | Full | Implementation | Writing code |
| `Review` | No | Read-only | Analysis | Code review |
| `Debug` | Limited | Read + Bash | Diagnostic | Debugging |

### Mode Selection Logic

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

### Mode State Machine

```
chat ──→ plan, build, debug
plan ──→ build, review, chat
build ──→ review, debug, plan, chat
review ──→ build, plan, chat
debug ──→ build, plan, chat
```

```rust
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
```

### Mode Instructions

Each mode has a system prompt instruction:

```rust
pub fn instruction(&self) -> &'static str {
    match self {
        AgentMode::Chat => "Answer directly. Suggest tools only if needed.",
        AgentMode::Plan => "Analyze and plan. Do NOT execute. Read-only.",
        AgentMode::Build => "Implement changes. Verify. Report progress.",
        AgentMode::Review => "Review code. Report issues and suggestions.",
        AgentMode::Debug => "Investigate. Find root cause. Suggest fix.",
    }
}
```

## Layer 4: Model Tier Selection

**File**: `src/router/model.rs`

**Purpose**: Select appropriate model tier based on complexity, intent, and config.

### 4 Model Tiers

| Tier | Models | Context | Cost/1K | Latency | Reasoning |
|------|--------|---------|---------|---------|-----------|
| `Local` | llama3.2, mistral | 8K-32K | $0 | Fast | Shallow |
| `Fast` | claude-haiku, gpt-4o-mini | 200K | $0.25 | Fast | Shallow |
| `Standard` | claude-sonnet, gpt-4o | 200K | $1.00 | Moderate | Moderate |
| `Capable` | claude-opus, gpt-4 | 200K+ | $3.00 | Slow | Deep |

### Selection Algorithm

```rust
pub fn pick_model_tier(
    complexity: Complexity,
    intent: Intent,
    mode: AgentMode,
    config: &RouterConfig,
) -> ModelTier {
    // Heavy/complex tasks need capable models
    if complexity >= Complexity::Complex {
        return ModelTier::Capable;
    }

    // Planning benefits from standard tier
    if intent == Intent::Plan || intent == Intent::Design {
        return ModelTier::Standard;
    }

    // Review/debug can use standard tier
    if intent == Intent::Review || intent == Intent::Debug {
        return ModelTier::Standard;
    }

    // Build mode for complex work needs standard
    if mode == AgentMode::Build && complexity >= Complexity::Moderate {
        return ModelTier::Standard;
    }

    // Trivial/simple tasks can use local or fast
    if complexity <= Complexity::Simple {
        if config.prefer_local {
            return ModelTier::Local;
        }
        return ModelTier::Fast;
    }

    // Default to standard
    ModelTier::Standard
}
```

### Cost Estimation

```rust
pub fn estimate_cost_per_1k(tier: ModelTier) -> f64 {
    match tier {
        ModelTier::Local => 0.0,    // No API cost
        ModelTier::Fast => 0.25,    // Haiku pricing
        ModelTier::Standard => 1.0, // Sonnet pricing
        ModelTier::Capable => 3.0,  // Opus pricing
    }
}
```

## Layer 5: Tool Policy

**File**: `src/router/tools.rs`

**Purpose**: Determine which tools are allowed/disallowed based on intent and mode.

### Tool Policy Structure

```rust
pub struct ToolPolicy {
    pub allowed_tools: Vec<String>,
    pub disallowed_tools: Vec<String>,
    pub require_confirmation: bool,
}
```

### Tool Policy by Mode

```rust
pub fn pick_tools(intent: Intent, mode: AgentMode) -> ToolPolicy {
    match mode {
        AgentMode::Build => build_mode_tools(intent),
        AgentMode::Review => ToolPolicy::read_only(),
        AgentMode::Debug => debug_mode_tools(),
        AgentMode::Plan => plan_mode_tools(),
        AgentMode::Chat => chat_mode_tools(),
    }
}
```

### Default Tool Policy

```rust
pub fn default_policy() -> Self {
    Self {
        allowed_tools: vec![
            "Read".to_string(),
            "Write".to_string(),
            "Bash".to_string(),
            "Grep".to_string(),
            "Glob".to_string(),
        ],
        disallowed_tools: vec![],
        require_confirmation: false,
    }
}
```

### Read-Only Policy

```rust
pub fn read_only() -> Self {
    Self {
        allowed_tools: vec!["Read".to_string(), "Grep".to_string(), "Glob".to_string()],
        disallowed_tools: vec!["Write".to_string(), "Bash".to_string()],
        require_confirmation: false,
    }
}
```

### Confirmation Required

Destructive operations require confirmation:

```rust
fn build_mode_tools(intent: Intent) -> ToolPolicy {
    match intent {
        Intent::Delete | Intent::Bash | Intent::Git => {
            ToolPolicy::with_confirmation(ToolPolicy::default_policy())
        }
        _ => ToolPolicy::default_policy(),
    }
}
```

## Layer 6: Context Budget

**File**: `src/router/context.rs`

**Purpose**: Allocate token budget for conversation context.

### 4 Budget Levels

| Strategy | Max Tokens | System | History | Memory | Tool Results |
|----------|----------:|-------:|--------:|-------:|-------------:|
| `Minimal` | 4,000 | 1,000 | 1,000 | 500 | 1,500 |
| `Relevant` | 16,000 | 2,000 | 6,000 | 2,000 | 6,000 |
| `Standard` | 50,000 | 4,000 | 20,000 | 5,000 | 21,000 |
| `Comprehensive` | 100,000 | 6,000 | 40,000 | 10,000 | 44,000 |

### Budget Selection

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

### Complexity to Budget Mapping

```rust
pub fn from_complexity(complexity: Complexity) -> Self {
    match complexity {
        Complexity::Trivial => ContextBudget::Minimal,
        Complexity::Simple => ContextBudget::Minimal,
        Complexity::Moderate => ContextBudget::Relevant,
        Complexity::Complex => ContextBudget::Standard,
        Complexity::Heavy => ContextBudget::Comprehensive,
    }
}
```

## Layer 7: Memory Policy

**File**: `src/router/memory.rs`

**Purpose**: Determine memory loading strategy based on intent and mode.

### 4 Memory Policies

| Policy | Description | Use Case |
|--------|-------------|----------|
| `None` | No memory loading | Trivial tasks, simple chat |
| `Recent` | Last N files modified | Planning, simple tasks |
| `Relevant` | Files relevant to current task | Review, debug |
| `Full` | All recent project context | Complex build tasks |

### Policy Selection

```rust
pub fn pick_memory_policy(intent: Intent, complexity: Complexity, mode: AgentMode) -> MemoryPolicy {
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

## Routing Decision

**File**: `src/router/types.rs`

The final output of the 7-layer pipeline:

```rust
pub struct RoutingDecision {
    /// Classified intent of the user prompt
    pub intent: Intent,
    /// Estimated complexity level
    pub complexity: Complexity,
    /// Selected operating mode
    pub mode: AgentMode,
    /// Selected model tier
    pub model_tier: ModelTier,
    /// Tool policy - which tools are allowed
    pub tools: ToolPolicy,
    /// Context budget allocation
    pub context_budget: ContextBudget,
    /// Memory loading policy
    pub memory_policy: MemoryPolicy,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Human-readable reasoning for debugging
    pub reasoning: String,
}
```

## Main Routing Function

**File**: `src/router/mod.rs`

```rust
pub fn route(prompt: &str, cwd: &str, config: &RouterConfig) -> RoutingDecision {
    // Layer 1: Intent Classification
    let intent = classify_intent(prompt);

    // Layer 2: Complexity Estimation
    let complexity = score_complexity(prompt);

    // Layer 3: Mode Selection
    let mode = pick_mode(intent, complexity);

    // Layer 4: Model Tier Selection
    let model_tier = pick_model_tier(complexity, intent, mode, config);

    // Layer 5: Tool Policy
    let tools = pick_tools(intent, mode);

    // Layer 6: Context Budget
    let context_budget = pick_budget(complexity, mode);

    // Layer 7: Memory Policy
    let memory_policy = pick_memory_policy(intent, complexity, mode);

    // Calculate confidence
    let confidence = calculate_confidence(intent, complexity);

    // Generate reasoning
    let reasoning = format!(
        "intent={}, complexity={}, mode={}, model={}, tools={}, budget={}, memory={}",
        intent.as_str(),
        complexity.as_str(),
        mode.as_str(),
        model_tier.as_str(),
        tools.allowed_tools.len(),
        context_budget.tokens(),
        memory_policy.as_str()
    );

    RoutingDecision::new(
        intent,
        complexity,
        mode,
        model_tier,
        tools,
        context_budget,
        memory_policy,
        confidence,
        reasoning,
    )
}
```

## Router Configuration

```rust
pub struct RouterConfig {
    /// Prefer local models (Ollama/llama.cpp) over API models
    pub prefer_local: bool,
    /// Maximum cost per million tokens
    pub cost_limit: f32,
    /// RAG configuration
    pub rag: RagRouterConfig,
    /// Prompt compaction configuration
    pub prompt_compaction: PromptCompactionConfig,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            prefer_local: false,
            cost_limit: 1.0,
            rag: RagRouterConfig::default(),
            prompt_compaction: PromptCompactionConfig::default(),
        }
    }
}
```

## Testing

The router has comprehensive unit tests:

### Intent Tests
- All 16 intents tested
- Priority/ordering tested
- Empty prompt handling

### Complexity Tests
- Trivial/Simple/Moderate/Complex/Heavy tested
- Keyword matching verified
- Score clamping verified

### Mode Tests
- All mode selections tested
- State machine transitions tested
- Invalid transitions rejected

### Model Tier Tests
- Complexity-based selection
- Intent-based adjustments
- Config overrides (prefer_local)

### Integration Tests
- End-to-end routing tests
- Confidence calculation
- Reasoning generation

## Performance

| Operation | Target | Actual |
|-----------|--------|--------|
| Intent Classification | < 1ms | ~0.05ms |
| Complexity Scoring | < 1ms | ~0.05ms |
| Full Routing Pipeline | < 5ms | ~0.5ms |

## Future Enhancements

1. **Learning Router**: Track which decisions lead to successful outcomes
2. **User Preferences**: Learn from user overrides
3. **Provider Health**: Track provider latency/errors for smarter selection
4. **Cost Tracking**: Real-time cost estimation and budget enforcement
5. **Embedding-Based Intent**: Use small local model for nuanced intent classification
