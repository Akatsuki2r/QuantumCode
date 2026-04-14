//! Context budget allocation
//!
//! Handles token budget allocation for conversation context.

use crate::router::types::{AgentMode, Complexity, ContextBudget};

/// Pick the context budget based on complexity and mode
pub fn pick_budget(complexity: Complexity, mode: AgentMode) -> ContextBudget {
    let base = ContextBudget::from_complexity(complexity);

    // Reduce budget for certain modes that need less context
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

/// Calculate available tokens for agent after system prompt
pub fn agent_token_budget(budget: ContextBudget, system_prompt_tokens: usize) -> usize {
    budget.tokens().saturating_sub(system_prompt_tokens)
}

/// Estimate tokens in a prompt
pub fn estimate_prompt_tokens(prompt: &str) -> usize {
    // Rough estimate: ~4 characters per token for English
    prompt.len() / 4
}
