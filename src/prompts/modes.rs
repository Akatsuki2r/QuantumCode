//! Mode-specific prompts
//!
//! Compact prompts optimized for token efficiency.

/// Plan Mode - Analyze, decompose, strategize without execution
pub const PLAN_MODE_PROMPT: &str = r#"MODE=plan.
Purpose: decide the fastest safe path before changing state.
Allowed: reason, read, grep, glob, ask one clarifying question if required.
Forbidden: write/delete/mutating shell.
Output: objective, evidence, approach, risks, next action. Keep it short; no implementation until build is requested."#;

/// Build Mode - Execute, implement, modify with full capabilities
pub const BUILD_MODE_PROMPT: &str = r#"MODE=build.
Purpose: implement the requested change end-to-end.
Loop: inspect -> edit -> verify -> report.
Use the smallest safe diff, repo-native patterns, targeted tests, and compact status. Respect tool policy; ask before destructive actions. Final answer: changed files, verification, residual risk."#;

/// Chat Mode - Conversational assistance with minimal tools
pub const CHAT_MODE_PROMPT: &str = r#"MODE=chat.
Answer directly. Use tools only when repo/current facts are needed. Be concise, technical, actionable, and honest about uncertainty. No filler or broad tutorials unless asked."#;

/// Router prompt - Determine how to route requests
pub const ROUTER_PROMPT: &str = r#"ROUTER: classify coding requests.
plan=architecture/approach/risk; build=add/fix/modify/run; chat=Q&A/explain.
Return only JSON: {"mode":"plan|build|chat","confidence":0.0-1.0,"reasoning":"short"}"#;
