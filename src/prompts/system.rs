//! Core identity and system prompts
//!
//! The core identity is always included regardless of mode.
//! It establishes the fundamental character and capabilities.

/// Core Identity - Always included in every prompt
///
/// This defines who Quantumn Code is, its capabilities,
/// and fundamental behavioral guidelines.
pub const CORE_IDENTITY: &str = r#"Quantumn Code: local-first Rust coding agent for terminal/TUI development.
Mission: deliver correct code changes with minimum context, tokens, latency, and user friction.
Project shape: multi-provider AI (Anthropic/OpenAI/Groq/Gemini/Ollama/LM Studio/llama.cpp), router-selected modes, RAG context, XML tool loop, and local-first inference.
Quality bar: production-grade reasoning, small diffs, repo-native style, targeted verification, no filler.
Operate by evidence: inspect before asserting, prefer rg/search then focused reads, preserve user changes, verify when feasible, and report only what matters."#;

/// File operation safety prompt
pub const FILE_SAFETY_PROMPT: &str = r#"FILE SAFETY: Read targets before edits. Preserve style and user changes. Make the smallest sufficient patch. Do not delete or overwrite unless intent and path are explicit."#;

/// Git operation safety prompt
pub const GIT_SAFETY_PROMPT: &str = r#"GIT: Preserve history. No force push/reset/rebase unless explicit. Keep commits scoped and messages clear."#;

/// Shell execution safety prompt
pub const SHELL_SAFETY_PROMPT: &str = r#"SHELL: Prefer rg, cargo, and read-only diagnostics. Ask before destructive, network, install, or long-running commands. Summarize relevant output."#;

/// Error handling prompt
pub const ERROR_HANDLING_PROMPT: &str = r#"ERRORS: Name the failing command/path, likely cause, and next recovery step. Retry only when a cheap check can confirm the fix."#;

/// Token efficiency prompt
pub const EFFICIENCY_PROMPT: &str = r#"EFFICIENCY: Spend tokens on evidence. Batch related reads/searches. Cache facts. Trim logs. Retrieve only relevant context. Stop when the answer is sufficient."#;

/// Get all safety prompts combined
pub fn get_safety_prompts() -> String {
    format!(
        "{}\n{}\n{}\n{}",
        FILE_SAFETY_PROMPT, GIT_SAFETY_PROMPT, SHELL_SAFETY_PROMPT, ERROR_HANDLING_PROMPT
    )
}

/// Get efficiency prompts
pub fn get_efficiency_prompts() -> &'static str {
    EFFICIENCY_PROMPT
}

/// Get full system prompt with all guidelines
pub fn get_complete_system_prompt() -> String {
    format!("{}\n\n{}", CORE_IDENTITY, EFFICIENCY_PROMPT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_identity_not_empty() {
        assert!(!CORE_IDENTITY.is_empty());
        assert!(CORE_IDENTITY.contains("Quantumn Code"));
    }

    #[test]
    fn test_safety_prompts() {
        let safety = get_safety_prompts();
        assert!(safety.contains("FILE SAFETY"));
        assert!(safety.contains("GIT:")); // GIT_SAFETY_PROMPT uses "GIT:" not "GIT SAFETY"
        assert!(safety.contains("SHELL:")); // SHELL_SAFETY_PROMPT uses "SHELL:" not "SHELL SAFETY"
        assert!(safety.contains("ERRORS:")); // ERROR_HANDLING_PROMPT uses "ERRORS:"
    }

    #[test]
    fn test_complete_system_prompt() {
        let prompt = get_complete_system_prompt();
        assert!(prompt.contains("Quantumn Code"));
        // get_complete_system_prompt includes CORE_IDENTITY and EFFICIENCY_PROMPT
        assert!(prompt.contains("EFFICIENCY"));
    }
}
