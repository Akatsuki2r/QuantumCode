//! Mode selection and state machine
//!
//! Handles selection of execution mode based on intent and complexity.

use crate::router::types::{AgentMode, Complexity, Intent};

/// Pick the appropriate mode based on intent and complexity
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

/// Check if a mode transition is valid
pub fn can_transition(current: AgentMode, target: AgentMode) -> bool {
    current.can_transition_to(target)
}

/// Transition to a new mode if valid, returns Some(new_mode) or None
pub fn transition(current: AgentMode, target: AgentMode) -> Option<AgentMode> {
    if can_transition(current, target) {
        Some(target)
    } else {
        None
    }
}

/// Get the mode instruction for system prompt injection
pub fn get_mode_instruction(mode: AgentMode) -> &'static str {
    mode.instruction()
}

/// Get the mode display name
pub fn get_mode_display(mode: AgentMode) -> &'static str {
    mode.as_str()
}
