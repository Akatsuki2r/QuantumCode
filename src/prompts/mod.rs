//! System prompts for different operating modes
//!
//! Each mode has optimized prompts that are heavily compacted
//! to minimize token usage while preserving quality and precision.

mod modes;
mod system;

pub use modes::*;
pub use system::*;

/// Prompt mode types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Plan mode: analyze, decompose, strategize - no execution
    Plan,
    /// Build mode: execute, implement, modify - full capabilities
    Build,
    /// Chat mode: converse, explain, assist - minimal tools
    Chat,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Chat
    }
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Plan => write!(f, "plan"),
            Mode::Build => write!(f, "build"),
            Mode::Chat => write!(f, "chat"),
        }
    }
}

impl std::str::FromStr for Mode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "plan" | "planning" => Ok(Mode::Plan),
            "build" | "execute" | "implementation" => Ok(Mode::Build),
            "chat" | "converse" => Ok(Mode::Chat),
            _ => Err(format!("Unknown mode: {}. Use: plan, build, or chat", s)),
        }
    }
}

/// Get the system prompt for a given mode
pub fn get_system_prompt(mode: Mode) -> &'static str {
    match mode {
        Mode::Plan => PLAN_MODE_PROMPT,
        Mode::Build => BUILD_MODE_PROMPT,
        Mode::Chat => CHAT_MODE_PROMPT,
    }
}

/// Get the core identity prompt (always included)
pub fn get_core_identity() -> &'static str {
    CORE_IDENTITY
}

/// Combine core identity with mode-specific prompt
pub fn get_full_prompt(mode: Mode) -> String {
    format!("{}\n\n{}", CORE_IDENTITY, get_system_prompt(mode))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_mode_default() {
        assert_eq!(Mode::default(), Mode::Chat);
    }

    #[test]
    fn test_mode_from_str() {
        assert_eq!(Mode::from_str("plan").unwrap(), Mode::Plan);
        assert_eq!(Mode::from_str("BUILD").unwrap(), Mode::Build);
        assert_eq!(Mode::from_str("chat").unwrap(), Mode::Chat);
        assert!(Mode::from_str("invalid").is_err());
    }

    #[test]
    fn test_get_system_prompt() {
        assert!(!get_system_prompt(Mode::Plan).is_empty());
        assert!(!get_system_prompt(Mode::Build).is_empty());
        assert!(!get_system_prompt(Mode::Chat).is_empty());
    }

    #[test]
    fn test_get_full_prompt() {
        let prompt = get_full_prompt(Mode::Plan);
        assert!(prompt.contains("Quantumn Code"));
        assert!(prompt.contains("plan"));
    }
}
