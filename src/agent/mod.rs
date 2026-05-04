//! Minimalistic agentic workflow - Bear tools for AI
//!
//! Ultra-simple tool system that adapts to the AI's needs.
//! Just the essentials: read, write, bash, grep, glob.

mod executor;
mod parser;
mod tools;

pub use crate::router::{route, RouterConfig, RoutingDecision};
pub use executor::{run_agentic, AgentExecutor};
pub use parser::parse_tool_calls;
pub use tools::{get_tools, Tool, ToolCall, ToolHandler, ToolRegistry, ToolResult};

/// Compact system prompt for agentic mode.
pub const AGENT_SYSTEM_PROMPT: &str = r#"Quantumn Code agent.
Goal: finish the user's coding task with minimum context, tokens, and tool calls while preserving correctness.
Policy: inspect before edits; prefer Glob/Grep then Read; obey router tool policy; never use blocked tools; avoid destructive shell/write/delete unless explicit; preserve unrelated user changes; verify when feasible.
Tools:
{{TOOLS_LIST}}
{{TOOL_CALL_FORMAT}}
Final answer: concise changed files, verification, and residual risk."#;

/// Build the final agent prompt with compact tool metadata injected.
pub fn build_agent_system_prompt(tool_registry: &ToolRegistry) -> String {
    build_agent_system_prompt_for_tools(tool_registry, None)
}

/// Build an agent prompt that only advertises tools allowed by routing policy.
pub fn build_agent_system_prompt_for_tools(
    tool_registry: &ToolRegistry,
    allowed_tools: Option<&[String]>,
) -> String {
    let tools = tool_registry.list_tools_for(allowed_tools);
    let format = tool_registry.tool_call_format_for(allowed_tools);

    AGENT_SYSTEM_PROMPT
        .replace("{{TOOLS_LIST}}", tools.trim_end())
        .replace("{{TOOL_CALL_FORMAT}}", format.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_prompt_is_injected_and_compact() {
        let registry = ToolRegistry::new();
        let prompt = build_agent_system_prompt(&registry);

        assert!(prompt.contains("Read("));
        assert!(prompt.contains("<tool><name>Read</name>"));
        assert!(!prompt.contains("{{"));
        assert!(prompt.len() < 1400);
    }

    #[test]
    fn test_agent_prompt_respects_allowed_tools() {
        let registry = ToolRegistry::new();
        let allowed = vec!["Read".to_string(), "Grep".to_string()];
        let prompt = build_agent_system_prompt_for_tools(&registry, Some(&allowed));

        assert!(prompt.contains("Read("));
        assert!(prompt.contains("Grep("));
        assert!(!prompt.contains("Write("));
        assert!(!prompt.contains("<tool><name>Write</name>"));
    }
}
