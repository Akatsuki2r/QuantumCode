//! Help command - Comprehensive command documentation and usage guide

use color_eyre::eyre::Result;

/// Command information structure
pub struct CommandInfo {
    pub name: &'static str,
    pub short_desc: &'static str,
    pub long_desc: &'static str,
    pub usage: &'static str,
    pub examples: &'static [&'static str],
    pub aliases: &'static [&'static str],
}

/// Get all available commands with descriptions
pub fn get_commands() -> Vec<CommandInfo> {
    vec![
        CommandInfo {
            name: "chat",
            short_desc: "Start interactive chat or one-shot query",
            long_desc: "Chat with the AI assistant. Can be used interactively (no arguments) or for one-shot queries with a prompt.",
            usage: "quantumn chat [PROMPT] [--model MODEL]",
            examples: &[
                "quantumn chat                              # Start interactive session",
                "quantumn chat \"Explain this function\"      # One-shot query",
                "quantumn chat --model claude-opus           # Use specific model",
            ],
            aliases: &["c", "ask"],
        },
        CommandInfo {
            name: "edit",
            short_desc: "Edit a file with AI assistance",
            long_desc: "Open a file for AI-assisted editing. Provide instructions and the AI will modify the file accordingly.",
            usage: "quantumn edit <FILE> [--prompt PROMPT] [--model MODEL]",
            examples: &[
                "quantumn edit src/main.rs                  # Interactive edit",
                "quantumn edit src/main.rs --prompt \"Add error handling\"",
                "quantumn edit config.toml --model gpt-4o",
            ],
            aliases: &["e", "modify"],
        },
        CommandInfo {
            name: "commit",
            short_desc: "Generate AI-powered git commit message",
            long_desc: "Analyze staged changes and generate an intelligent commit message. Supports conventional commits format.",
            usage: "quantumn commit [--message MESSAGE] [--model MODEL]",
            examples: &[
                "quantumn commit                            # Generate from staged changes",
                "quantumn commit --message \"Fix login bug\"  # Use custom message",
                "quantumn commit --model claude-opus        # Use specific model",
            ],
            aliases: &["ci", "git"],
        },
        CommandInfo {
            name: "review",
            short_desc: "AI-powered code review",
            long_desc: "Perform comprehensive code review on specified files or staged changes. Analyzes code quality, security, and best practices.",
            usage: "quantumn review [FILES...] [--model MODEL]",
            examples: &[
                "quantumn review                            # Review staged changes",
                "quantumn review src/lib.rs                 # Review specific file",
                "quantumn review src/**/*.rs                # Review multiple files",
            ],
            aliases: &["r", "code-review"],
        },
        CommandInfo {
            name: "test",
            short_desc: "Run tests with AI analysis",
            long_desc: "Execute tests and provide AI-powered analysis of failures, suggestions for fixes, and coverage insights.",
            usage: "quantumn test [PATH] [--model MODEL]",
            examples: &[
                "quantumn test                              # Run all tests",
                "quantumn test src/auth_tests.rs           # Run specific tests",
                "quantumn test --model gpt-4o               # Use specific model",
            ],
            aliases: &["t"],
        },
        CommandInfo {
            name: "scaffold",
            short_desc: "Create new project from templates",
            long_desc: "Generate new projects from pre-built templates. Supports multiple languages and frameworks.",
            usage: "quantumn scaffold <TYPE> <NAME>",
            examples: &[
                "quantumn scaffold rust my-app              # New Rust project",
                "quantumn scaffold python my-script         # New Python project",
                "quantumn scaffold node my-api              # New Node.js project",
                "quantumn scaffold web my-website           # New web project",
            ],
            aliases: &["s", "new", "create"],
        },
        CommandInfo {
            name: "session",
            short_desc: "Manage conversation sessions",
            long_desc: "Save, load, list, and manage conversation sessions for resuming work later.",
            usage: "quantumn session <COMMAND>",
            examples: &[
                "quantumn session list                      # List saved sessions",
                "quantumn session save feature-x            # Save current session",
                "quantumn session resume feature-x         # Resume session",
                "quantumn session delete feature-x         # Delete session",
            ],
            aliases: &["sess"],
        },
        CommandInfo {
            name: "config",
            short_desc: "Manage configuration settings",
            long_desc: "View and modify Quantumn Code configuration. Settings are stored in ~/.config/quantumn-code/config.toml",
            usage: "quantumn config <COMMAND>",
            examples: &[
                "quantumn config show                       # Show all settings",
                "quantumn config get model.provider         # Get specific value",
                "quantumn config set ui.theme oxidized      # Set theme",
                "quantumn config set model.default_model claude-sonnet-4-20250514",
                "quantumn config reset                      # Reset to defaults",
                "quantumn config edit                       # Open in editor",
            ],
            aliases: &["cfg"],
        },
        CommandInfo {
            name: "theme",
            short_desc: "Manage terminal themes",
            long_desc: "List, set, and preview terminal UI themes. Supports built-in and custom themes.",
            usage: "quantumn theme <COMMAND>",
            examples: &[
                "quantumn theme list                        # List available themes",
                "quantumn theme set oxidized                # Set theme",
                "quantumn theme current                     # Show current theme",
                "quantumn theme preview tokyo_night         # Preview theme",
            ],
            aliases: &["th"],
        },
        CommandInfo {
            name: "model",
            short_desc: "Switch AI models and providers",
            long_desc: "List available models, switch between providers (Anthropic, OpenAI, Ollama, llama.cpp), and configure API keys.",
            usage: "quantumn model [PROVIDER] [--list]",
            examples: &[
                "quantumn model list                        # List all models",
                "quantumn model                             # Show current model",
                "quantumn model anthropic                   # Switch to Claude",
                "quantumn model openai                      # Switch to OpenAI",
                "quantumn model ollama                      # Switch to Ollama (local)",
                "quantumn model llama_cpp                  # Switch to llama.cpp (local)",
            ],
            aliases: &["m", "provider"],
        },
        CommandInfo {
            name: "status",
            short_desc: "Show system status",
            long_desc: "Display current configuration, model, provider, and git status.",
            usage: "quantumn status",
            examples: &[
                "quantumn status                            # Show status",
            ],
            aliases: &["st"],
        },
        CommandInfo {
            name: "version",
            short_desc: "Show version information",
            long_desc: "Display Quantumn Code version and build information.",
            usage: "quantumn version",
            examples: &[
                "quantumn version                           # Show version",
            ],
            aliases: &["v", "-v", "--version"],
        },
    ]
}

/// Get provider information
pub fn get_providers() -> Vec<ProviderInfo> {
    vec![
        ProviderInfo {
            name: "anthropic",
            display_name: "Anthropic Claude",
            description: "Advanced AI assistant with Claude models",
            models: &[
                "claude-opus-4-20250514   - Most capable (Opus 4)",
                "claude-sonnet-4-20250514 - Balanced (Sonnet 4) [default]",
                "claude-haiku-4-20250514  - Fast (Haiku 4)",
                "claude-3-5-sonnet-20241022 - Legacy (Sonnet 3.5)",
                "claude-3-5-haiku-20241022  - Legacy (Haiku 3.5)",
            ],
            env_key: "ANTHROPIC_API_KEY",
            setup: "export ANTHROPIC_API_KEY=your_key_here",
        },
        ProviderInfo {
            name: "openai",
            display_name: "OpenAI",
            description: "GPT models from OpenAI",
            models: &[
                "gpt-4o       - GPT-4 Omni (recommended)",
                "gpt-4o-mini  - GPT-4 Omni Mini (fast, cheap)",
                "gpt-4-turbo  - GPT-4 Turbo",
                "o1           - O1 (advanced reasoning)",
                "o1-mini      - O1 Mini",
            ],
            env_key: "OPENAI_API_KEY",
            setup: "export OPENAI_API_KEY=your_key_here",
        },
        ProviderInfo {
            name: "ollama",
            display_name: "Ollama (Local)",
            description: "Run models locally with Ollama - No API key required",
            models: &[
                "llama3.2       - Meta Llama 3.2",
                "llama3.1       - Meta Llama 3.1",
                "mistral        - Mistral",
                "codellama      - Code Llama",
                "deepseek-coder - DeepSeek Coder",
                "qwen2.5-coder  - Qwen 2.5 Coder",
            ],
            env_key: "N/A (local)",
            setup: "1. Install: curl https://ollama.ai/install.sh | sh\n2. Run: ollama serve\n3. Pull model: ollama pull llama3.2",
        },
        ProviderInfo {
            name: "llama_cpp",
            display_name: "llama.cpp (Local, High-Performance)",
            description: "High-performance local inference with llama.cpp",
            models: &[
                "llama3.2       - Meta Llama 3.2 (GGUF)",
                "llama3.1       - Meta Llama 3.1 (GGUF)",
                "mistral        - Mistral (GGUF)",
                "qwen2.5        - Qwen 2.5 (GGUF)",
                "deepseek-coder - DeepSeek Coder (GGUF)",
            ],
            env_key: "N/A (local)",
            setup: "1. Install llama-server binary\n2. Download GGUF model files\n3. Configure paths in ~/.config/quantumn-code/config.toml",
        },
    ]
}

/// Provider information structure
pub struct ProviderInfo {
    pub name: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub models: &'static [&'static str],
    pub env_key: &'static str,
    pub setup: &'static str,
}

/// Get theme information
pub fn get_themes() -> Vec<ThemeInfo> {
    vec![
        ThemeInfo {
            name: "oxidized",
            description: "Rusty brown on deep black - Elegant, unique (default)",
            author: "Quantumn",
        },
        ThemeInfo {
            name: "default",
            description: "Classic Claude-inspired purple theme",
            author: "Quantumn",
        },
        ThemeInfo {
            name: "tokyo_night",
            description: "Purple and blue accents, popular dark theme",
            author: "Quantumn",
        },
        ThemeInfo {
            name: "hacker",
            description: "Matrix-style green on black",
            author: "Quantumn",
        },
        ThemeInfo {
            name: "deep_black",
            description: "Minimal high-contrast dark theme",
            author: "Quantumn",
        },
    ]
}

/// Theme information structure
pub struct ThemeInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub author: &'static str,
}

/// Get keyboard shortcuts
pub fn get_shortcuts() -> Vec<ShortcutInfo> {
    vec![
        ShortcutInfo { key: "Ctrl+S", action: "Save current session" },
        ShortcutInfo { key: "Ctrl+L", action: "Load saved session" },
        ShortcutInfo { key: "Ctrl+T", action: "Cycle through themes" },
        ShortcutInfo { key: "Ctrl+P", action: "Switch AI provider" },
        ShortcutInfo { key: "Ctrl+M", action: "Switch AI model" },
        ShortcutInfo { key: "Ctrl+Q", action: "Quit application" },
        ShortcutInfo { key: "Tab", action: "Switch between panes" },
        ShortcutInfo { key: "Shift+Tab", action: "Switch panes (reverse)" },
        ShortcutInfo { key: "Enter", action: "Send message" },
        ShortcutInfo { key: "Esc", action: "Cancel/exit" },
        ShortcutInfo { key: "Up/Down", action: "Navigate history" },
        ShortcutInfo { key: "Ctrl+C", action: "Cancel current operation" },
        ShortcutInfo { key: "Ctrl+R", action: "Clear screen" },
        ShortcutInfo { key: "Ctrl+H", action: "Show help" },
        ShortcutInfo { key: "/", action: "Enter command mode (prefix commands with /)" },
    ]
}

/// Shortcut information
pub struct ShortcutInfo {
    pub key: &'static str,
    pub action: &'static str,
}

/// Command mode commands (prefixed with /)
pub fn get_slash_commands() -> Vec<SlashCommandInfo> {
    vec![
        SlashCommandInfo {
            command: "/help",
            description: "Show this help message",
            autocomplete: true,
        },
        SlashCommandInfo {
            command: "/clear",
            description: "Clear the chat history",
            autocomplete: true,
        },
        SlashCommandInfo {
            command: "/mode",
            description: "Switch mode: plan | build | chat",
            autocomplete: true,
        },
        SlashCommandInfo {
            command: "/model",
            description: "Switch AI model",
            autocomplete: true,
        },
        SlashCommandInfo {
            command: "/provider",
            description: "Switch AI provider",
            autocomplete: true,
        },
        SlashCommandInfo {
            command: "/theme",
            description: "Switch theme",
            autocomplete: true,
        },
        SlashCommandInfo {
            command: "/session",
            description: "Manage sessions: save | load | list",
            autocomplete: true,
        },
        SlashCommandInfo {
            command: "/config",
            description: "View or edit configuration",
            autocomplete: true,
        },
        SlashCommandInfo {
            command: "/status",
            description: "Show current status",
            autocomplete: true,
        },
        SlashCommandInfo {
            command: "/version",
            description: "Show version",
            autocomplete: true,
        },
        SlashCommandInfo {
            command: "/exit",
            description: "Exit Quantumn Code",
            autocomplete: true,
        },
        SlashCommandInfo {
            command: "/quit",
            description: "Exit Quantumn Code (alias for /exit)",
            autocomplete: true,
        },
    ]
}

/// Slash command information
pub struct SlashCommandInfo {
    pub command: &'static str,
    pub description: &'static str,
    pub autocomplete: bool,
}

/// Run the help command
pub async fn run(section: Option<String>) -> Result<()> {
    match section.as_deref() {
        Some("commands") => print_commands(),
        Some("providers") => print_providers(),
        Some("themes") => print_themes(),
        Some("shortcuts") => print_shortcuts(),
        Some("slash") => print_slash_commands(),
        Some("quick") => print_quick_start(),
        None => print_full_help(),
        _ => {
            println!("Unknown help section: {}", section.unwrap());
            println!("Available sections: commands, providers, themes, shortcuts, slash, quick");
        }
    }
    Ok(())
}

/// Print full help documentation
fn print_full_help() {
    print_banner();
    println!();
    print_quick_start();
    println!();
    print_commands();
    println!();
    print_providers();
    println!();
    print_themes();
    println!();
    print_shortcuts();
    println!();
    print_slash_commands();
    print_footer();
}

/// Print the Quantumn Code banner
fn print_banner() {
    println!();
    println!(r"   ____  _   _ _   _ _   _ ___  ____  ");
    println!(r"  |  _ \| | | | \ | | \ | |   \|_  / ");
    println!(r"  | |_) | |_| |  \| |  \| | |) |/ /  ");
    println!(r"  |  _ <|  _  | |\  | |\  |   // /   ");
    println!(r"  |_| \_\_| |_|_| \_|_| \_|__/___|   ");
    println!();
    println!("  An advanced AI-powered coding assistant CLI");
    println!("  Built in Rust for performance and reliability");
    println!();
}

/// Print footer
fn print_footer() {
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("  Documentation: https://github.com/Akatsuki2r/QuantumCode");
    println!("  Issues:        https://github.com/Akatsuki2r/QuantumCode/issues");
    println!();
    println!("  Made with Rust by NahanoMark");
    println!();
}

/// Print quick start guide
fn print_quick_start() {
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│  QUICK START                                                    │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!();
    println!("  1. Install:");
    println!("     cargo install quantumn");
    println!("     # or");
    println!("     npm install -g @quantumn/code");
    println!();
    println!("  2. Set up your AI provider:");
    println!("     export ANTHROPIC_API_KEY=your_key_here   # For Claude");
    println!("     # or");
    println!("     export OPENAI_API_KEY=your_key_here        # For OpenAI");
    println!("     # or use local models with Ollama:");
    println!("     ollama serve && ollama pull llama3.2");
    println!();
    println!("  3. Start chatting:");
    println!("     quantumn                                 # Interactive mode");
    println!("     quantumn chat \"Explain this code\"        # One-shot query");
    println!();
    println!("  4. Useful commands:");
    println!("     quantumn edit src/main.rs                 # Edit file with AI");
    println!("     quantumn commit                           # Generate commit");
    println!("     quantumn review src/lib.rs                # Code review");
    println!();
}

/// Print commands section
fn print_commands() {
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│  COMMANDS                                                       │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!();

    let commands = get_commands();
    let max_name_len = commands.iter().map(|c| c.name.len()).max().unwrap_or(12);

    for cmd in &commands {
        println!("  {:width$}  {}", cmd.name, cmd.short_desc, width = max_name_len);
        if !cmd.aliases.is_empty() {
            println!("  {:width$}  Aliases: {}", "", cmd.aliases.join(", "), width = max_name_len);
        }
    }

    println!();
    println!("  Use 'quantumn <command> --help' for detailed usage");
    println!("  Use 'quantumn help commands' for full command reference");
}

/// Print providers section
fn print_providers() {
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│  AI PROVIDERS                                                   │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!();

    for provider in get_providers() {
        println!("  {} ({})", provider.display_name, provider.name);
        println!("  {}", provider.description);
        println!("  Models:");
        for model in provider.models {
            println!("    • {}", model);
        }
        println!("  API Key: {}", provider.env_key);
        println!("  Setup: {}", provider.setup);
        println!();
    }
}

/// Print themes section
fn print_themes() {
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│  THEMES                                                         │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!();

    for theme in get_themes() {
        let default_marker = if theme.name == "oxidized" { " [default]" } else { "" };
        println!("  {}{} - {}", theme.name, default_marker, theme.description);
    }

    println!();
    println!("  Set theme: quantumn theme set <name>");
    println!("  List themes: quantumn theme list");
}

/// Print shortcuts section
fn print_shortcuts() {
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│  KEYBOARD SHORTCUTS                                             │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!();

    let shortcuts = get_shortcuts();
    let max_key_len = shortcuts.iter().map(|s| s.key.len()).max().unwrap_or(10);

    for shortcut in &shortcuts {
        println!("  {:width$}  {}", shortcut.key, shortcut.action, width = max_key_len);
    }
}

/// Print slash commands section
fn print_slash_commands() {
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│  SLASH COMMANDS (in interactive mode)                          │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!();

    let commands = get_slash_commands();
    let max_cmd_len = commands.iter().map(|c| c.command.len()).max().unwrap_or(10);

    for cmd in &commands {
        println!("  {:width$}  {}", cmd.command, cmd.description, width = max_cmd_len);
    }

    println!();
    println!("  Type / followed by command in interactive mode");
    println!("  Tab completion is available for all slash commands");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_commands() {
        let commands = get_commands();
        assert!(!commands.is_empty());
        assert!(commands.iter().any(|c| c.name == "chat"));
    }

    #[test]
    fn test_get_providers() {
        let providers = get_providers();
        assert!(!providers.is_empty());
        assert!(providers.iter().any(|p| p.name == "anthropic"));
    }

    #[test]
    fn test_get_themes() {
        let themes = get_themes();
        assert!(!themes.is_empty());
        assert!(themes.iter().any(|t| t.name == "oxidized"));
    }
}