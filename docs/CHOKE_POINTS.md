# Choke Points & Areas for Improvement

## Overview

This document identifies performance bottlenecks, architectural weaknesses, and areas needing improvement in Quantum Code.

## Critical Choke Points

### 1. Router-Agent Disconnection

**Severity**: HIGH
**File**: `src/router/mod.rs`, `src/agent/executor.rs`

**Problem**: The router makes sophisticated routing decisions, but the agent executor doesn't fully utilize them. The router output is not piped into the agent's execution loop.

**Current State**:
```rust
// Router makes decision
let decision = route(prompt, cwd, &config);

// But agent doesn't use decision.mode, decision.tools, etc.
agent.execute(prompt).await?;
```

**Impact**:
- Router decisions are essentially ignored
- Mode selection has no effect on execution
- Tool policies not enforced
- Model tier selection not used

**Fix Required**:
```rust
// Router makes decision
let decision = route(prompt, cwd, &config);

// Agent uses decision
agent.mode = decision.mode;
agent.allowed_tools = decision.tools.allowed_tools;
agent.model = get_model_for_tier(decision.model_tier);
agent.context_budget = decision.context_budget.tokens();
agent.execute(prompt).await?;
```

**Estimated Effort**: 4-8 hours

---

### 2. RAG Not Integrated

**Severity**: HIGH
**File**: `src/rag/mod.rs`

**Problem**: RAG system exists but is never called during actual inference. Context retrieval is completely disconnected from the agent workflow.

**Current State**:
```rust
// RAG index exists but is never populated or queried
let rag_index = RagIndex::new(RagConfig::default());
// ... never used
```

**Impact**:
- No context-aware responses
- AI lacks project-specific knowledge
- Reduced response quality for codebase questions

**Fix Required**:
1. Populate RAG index on startup (scan project files)
2. Query RAG before sending prompt to LLM
3. Augment prompt with retrieved context
4. Handle RAG failures gracefully

**Estimated Effort**: 8-16 hours

---

### 3. No Provider Failover

**Severity**: MEDIUM
**File**: `src/providers/mod.rs`

**Problem**: If a provider fails (rate limit, network error), there's no automatic fallback to another provider. User must manually switch.

**Current State**:
```rust
// Single provider, no fallback
let response = provider.chat(messages).await?;
// If this fails, entire request fails
```

**Impact**:
- Poor user experience on API failures
- No resilience to rate limiting
- Cannot combine free tier limits across providers

**Fix Required**:
```rust
// Tiered fallback system
pub struct FailoverRouter {
    providers: Vec<(Priority, Box<dyn Provider>)>,
}

impl FailoverRouter {
    pub async fn chat(&self, messages: Vec<Message>) -> Result<String, ProviderError> {
        for (priority, provider) in &self.providers {
            match provider.chat(messages.clone()).await {
                Ok(response) => return Ok(response),
                Err(ProviderError::RateLimit | ProviderError::Network) => continue,
                Err(e) => return Err(e),
            }
        }
        Err(ProviderError::AllProvidersFailed)
    }
}
```

**Estimated Effort**: 8-12 hours

---

### 4. Token Estimation Inaccuracy

**Severity**: MEDIUM
**File**: `src/router/context.rs`

**Problem**: Token estimation uses a naive `len() / 4` heuristic, which is inaccurate for code (especially with identifiers, special characters).

**Current State**:
```rust
pub fn estimate_prompt_tokens(prompt: &str) -> usize {
    prompt.len() / 4  // ~4 chars per token for English
}
```

**Impact**:
- Context budget miscalculation
- Potential truncation of important content
- Cost estimation errors

**Fix Required**:
1. Use tiktoken-rs or similar library
2. Model-specific tokenizers (Claude vs GPT differ)
3. Cache token counts to avoid recomputation

**Estimated Effort**: 2-4 hours

---

### 5. No Streaming Support

**Severity**: MEDIUM
**File**: `src/providers/provider_trait.rs`

**Problem**: While `stream()` is defined in the trait, it's not implemented for any provider. Users wait for full response instead of seeing real-time output.

**Current State**:
```rust
pub trait Provider {
    fn chat(&self, messages: Vec<Message>) -> Result<String, ProviderError>;
    fn stream(&self, messages: Vec<Message>) -> Result<impl Stream<Item = StreamChunk>, ProviderError>;
    // stream() is never implemented
}
```

**Impact**:
- Poor UX for long responses
- Cannot interleave tool execution with generation
- Higher perceived latency

**Fix Required**:
1. Implement streaming for Anthropic
2. Implement streaming for OpenAI
3. Implement streaming for local providers
4. Update TUI to render streaming output

**Estimated Effort**: 8-16 hours

---

### 6. Memory Policy Not Implemented

**Severity**: MEDIUM
**File**: `src/router/memory.rs`

**Problem**: Memory policy is selected but never actually loads any memory. The `MemoryPolicy` enum exists but has no backing implementation.

**Current State**:
```rust
pub fn pick_memory_policy(...) -> MemoryPolicy {
    // Returns a policy, but nothing acts on it
    MemoryPolicy::Recent
}
```

**Impact**:
- No conversation history across turns
- No project memory persistence
- AI lacks context from previous interactions

**Fix Required**:
1. Implement memory storage (SQLite or JSON files)
2. Load memory based on policy
3. Inject memory into context
4. Update memory after each turn

**Estimated Effort**: 8-12 hours

---

### 7. Tool Execution Not Integrated

**Severity**: HIGH
**File**: `src/agent/executor.rs`, `src/tools/mod.rs`

**Problem**: Tools are defined but not fully integrated into the agent execution loop. Tool calls from AI are not parsed and executed.

**Current State**:
```rust
// Tools exist
pub trait Tool {
    fn execute(&self, params: &Value) -> Result<Value, ToolError>;
}

// But agent doesn't call them
agent.execute(prompt).await?;  // No tool handling
```

**Impact**:
- AI cannot actually modify files
- Cannot run shell commands
- Essentially a chatbot, not a coding assistant

**Fix Required**:
1. Parse tool calls from AI response
2. Validate against tool policy
3. Execute tools with proper error handling
4. Feed results back to AI

**Estimated Effort**: 16-24 hours

---

### 8. No Rate Limiting

**Severity**: MEDIUM (Security)
**File**: N/A (missing)

**Problem**: No rate limiting on tool execution or API calls. Could lead to:
- Accidental API quota exhaustion
- Runaway bash command execution
- Resource exhaustion

**Impact**:
- Potential for accidental abuse
- API bills could spike
- System stability risks

**Fix Required**:
```rust
pub struct RateLimiter {
    max_requests_per_minute: usize,
    tokens: Arc<Mutex<TokenBucket>>,
}

impl RateLimiter {
    pub async fn acquire(&self) -> Result<(), RateLimitExceeded> {
        // Token bucket or sliding window
    }
}
```

**Estimated Effort**: 4-6 hours

---

### 9. Local Discovery Only on Startup

**Severity**: LOW
**File**: `src/providers/local_discover.rs`

**Problem**: Local models are discovered once on startup. If user installs a new Ollama model while app is running, it won't appear until restart.

**Current State**:
```rust
pub fn discover_all_models() -> LocalModelConfig {
    // Runs once on startup
    // Never re-runs
}
```

**Impact**:
- Stale model list
- User must restart to see new models
- Poor UX

**Fix Required**:
1. Add `refresh_discovery()` method
2. Call on model command invocation
3. Optional: file watching for model directories

**Estimated Effort**: 2-4 hours

---

### 10. No Input Validation

**Severity**: MEDIUM (Security)
**File**: Multiple

**Problem**: AI-generated tool calls are not validated before execution. Could lead to:
- Path traversal attacks
- Command injection
- Unauthorized file access

**Impact**:
- Security vulnerabilities
- Potential for data loss
- Privilege escalation risks

**Fix Required**:
```rust
// Validate file paths
pub fn validate_path(path: &str, allowed_dirs: &[PathBuf]) -> Result<PathBuf, ValidationError> {
    let resolved = fs::canonicalize(path)?;
    if allowed_dirs.iter().any(|dir| resolved.starts_with(dir)) {
        Ok(resolved)
    } else {
        Err(ValidationError::PathNotAllowed)
    }
}

// Validate bash commands
pub fn validate_command(cmd: &str) -> Result<(), ValidationError> {
    // Block dangerous commands (rm -rf /, etc.)
}
```

**Estimated Effort**: 8-12 hours

---

## Performance Issues

### 11. Regex Recompilation (Fixed)

**Severity**: LOW (Fixed)
**File**: `src/router/analyzer.rs`

**Status**: Fixed with `lazy_static!`

**Before**:
```rust
// Compiled on every call - SLOW
let regex = Regex::new(pattern).unwrap();
```

**After**:
```rust
lazy_static! {
    static ref INTENT_REGEX_SET: RegexSet = RegexSet::new(patterns).unwrap();
}
```

---

### 12. No Connection Pooling

**Severity**: LOW
**File**: `src/providers/`

**Problem**: HTTP connections are created per request, not pooled.

**Impact**:
- Higher latency
- More TCP handshakes
- Resource inefficiency

**Fix Required**:
Use `reqwest::Client` as a singleton (already done, but verify connection pooling is enabled).

**Estimated Effort**: 1-2 hours

---

### 13. TUI Render Performance

**Severity**: LOW
**File**: `src/tui/render.rs`

**Problem**: Full re-render on every update, no dirty tracking.

**Impact**:
- Higher CPU usage
- Battery drain on laptops
- Potential flicker

**Fix Required**:
1. Track dirty regions
2. Only re-render changed areas
3. Use ratatui's `Frame::render_widget` selectively

**Estimated Effort**: 4-8 hours

---

## Architectural Concerns

### 14. Tight Coupling: TUI ↔ Agent

**Severity**: MEDIUM
**File**: `src/tui/app.rs`

**Problem**: TUI directly invokes agent logic, making it hard to:
- Test agent independently
- Use agent without TUI
- Swap agent implementations

**Fix Required**:
1. Define Agent trait
2. Inject agent into TUI
3. Use dependency injection pattern

**Estimated Effort**: 4-6 hours

---

### 15. No Error Recovery Strategy

**Severity**: MEDIUM
**File**: Multiple

**Problem**: Errors propagate up but no recovery strategy. App may become unusable after transient errors.

**Impact**:
- Poor resilience
- User frustration
- Lost work context

**Fix Required**:
1. Define error categories (recoverable vs fatal)
2. Implement retry logic for transient errors
3. Graceful degradation

**Estimated Effort**: 4-8 hours

---

## Testing Gaps

### 16. Low Test Coverage

**Severity**: MEDIUM
**Current**: ~60% of core modules
**Target**: 80%+

**Missing Tests**:
- Provider integration tests
- Tool execution tests
- TUI widget tests
- End-to-end workflow tests

**Estimated Effort**: 20-40 hours

---

### 17. No Performance Tests

**Severity**: LOW
**File**: N/A (missing)

**Problem**: No benchmarks or performance regression tests.

**Fix Required**:
```rust
#[cfg(test)]
mod benchmarks {
    use test::Bencher;

    #[bench]
    fn bench_intent_classification(b: &mut Bencher) {
        b.iter(|| classify_intent("read src/main.rs"));
    }
}
```

**Estimated Effort**: 4-6 hours

---

## Security Concerns

### 18. No Command Sandboxing

**Severity**: HIGH (Security)
**File**: `src/tools/bash.rs`

**Problem**: Bash commands run with full user privileges. No sandboxing, no restrictions.

**Impact**:
- AI could delete files
- Could exfiltrate data
- Could modify system files

**Fix Required**:
1. Whitelist allowed commands
2. Sandbox execution (e.g., using containers or sandboxing libs)
3. Audit logging

**Estimated Effort**: 16-24 hours

---

### 19. No Secret Detection

**Severity**: MEDIUM (Security)
**File**: N/A (missing)

**Problem**: AI could accidentally commit API keys, passwords, etc.

**Impact**:
- Credential leakage
- Security breaches

**Fix Required**:
```rust
pub fn detect_secrets(content: &str) -> Vec<SecretFinding> {
    // Check for patterns like:
    // - API keys (sk-..., ghp_..., etc.)
    // - Private keys (-----BEGIN RSA PRIVATE KEY-----)
    // - Passwords in config
}
```

**Estimated Effort**: 4-8 hours

---

## Summary by Priority

### Critical (Fix Immediately)
1. Router-Agent Disconnection
2. RAG Not Integrated
3. Tool Execution Not Integrated
4. Command Sandboxing

### High (Fix Soon)
5. No Provider Failover
6. Memory Policy Not Implemented
7. Input Validation
8. No Error Recovery

### Medium (Plan to Fix)
9. Token Estimation Inaccuracy
10. No Streaming Support
11. Rate Limiting
12. Secret Detection
13. Tight Coupling

### Low (Nice to Have)
14. Local Discovery Refresh
15. Connection Pooling
16. TUI Render Optimization
17. Performance Tests

---

## Effort Estimates

| Priority | Hours | Weeks (1 dev) |
|----------|-------|---------------|
| Critical | 48-72 | 1.5-2 |
| High | 32-48 | 1-1.5 |
| Medium | 24-36 | 0.5-1 |
| Low | 12-20 | 0.25-0.5 |
| **Total** | **116-176** | **3.5-5** |

---

## Recommendations

1. **Start with Router-Agent Integration** - This unlocks all router capabilities
2. **Then Tool Execution** - This makes the agent actually useful
3. **Then RAG Integration** - Improves response quality
4. **Then Security** - Sandboxing, validation, rate limiting
5. **Then Polish** - Streaming, failover, performance
