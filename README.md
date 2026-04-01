# Quantumn Code

<div align="center">

![Quantumn Code](https://img.shields.io/badge/Quantumn-Code-purple?style=for-the-badge)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange?style=for-the-badge&logo=rust)
![License](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)

**An advanced AI-powered coding assistant CLI built in Rust**

[Features](#features) • [Installation](#installation) • [Usage](#usage) • [Themes](#themes)

</div>

---

## Features

- **Multi-Provider AI Support**: Works with Anthropic Claude, OpenAI, and Ollama (local LLMs)
- **15+ CLI Commands**: chat, edit, commit, review, test, scaffold, and more
- **Beautiful Themes**: Tokyo Night, Hacker (Matrix-style), Deep Black, and custom themes
- **Interactive TUI**: Split-pane terminal interface for comfortable coding sessions
- **Session Management**: Save and resume conversations
- **Git Integration**: AI-generated commit messages and PR descriptions
- **Project Scaffolding**: Create new projects from templates (Rust, Python, Node, etc.)
- **Syntax Highlighting**: Powered by syntect for beautiful code display
- **Cross-Platform**: Works on Linux, macOS, and Windows

## Installation

### Option 1: Download Binary (Recommended)

Download the latest release for your platform:

| Platform | Download |
|----------|----------|
| Linux x64 | `quantumn-x86_64-unknown-linux-gnu.tar.gz` |
| Linux ARM64 | `quantumn-aarch64-unknown-linux-gnu.tar.gz` |
| macOS Intel | `quantumn-x86_64-apple-darwin.tar.gz` |
| macOS Apple Silicon | `quantumn-aarch64-apple-darwin.tar.gz` |
| Windows | `quantumn-x86_64-pc-windows-gnu.zip` |

```bash
# Linux/macOS quick install
curl -sL https://github.com/Akatsuki2r/QuantumCode/releases/latest/download/quantumn-$(uname -m)-unknown-linux-gnu.tar.gz | tar xz
sudo mv quantumn /usr/local/bin/
```

### Option 2: Cargo (Rust)

```bash
cargo install quantumn
```

### Option 3: npm

```bash
npm install -g @quantumn/code
```

### Option 4: Homebrew (macOS)

```bash
brew tap Akatsuki2r/quantumn-code
brew install quantumn-code
```

### Option 5: AUR (Arch Linux)

```bash
yay -S quantumn-code
```

### Option 6: Build from Source

```bash
git clone https://github.com/Akatsuki2r/QuantumCode.git
cd QuantumCode
cargo build --release
sudo cp target/release/quantumn /usr/local/bin/
```

## Usage

### Interactive Mode

```bash
quantumn
```

This launches the interactive TUI where you can:
- Chat with AI about your code
- Edit files with AI assistance
- Run git commands with AI-generated messages
- Switch between themes and providers

### Commands

```bash
# Chat with AI
quantumn chat "Explain this function"

# Edit a file
quantumn edit src/main.rs "Add error handling"

# Generate commit message
quantumn commit

# Review code
quantumn review src/lib.rs

# Run tests with AI analysis
quantumn test

# Scaffold a new project
quantumn scaffold rust my-app
quantumn scaffold python my-script
quantumn scaffold node my-api

# Manage sessions
quantumn session save feature-x
quantumn session load feature-x
quantumn session list

# Switch themes
quantumn theme set hacker
quantumn theme list

# Manage AI providers
quantumn model list
quantumn model use claude-sonnet
quantumn model config
```

### Keyboard Shortcuts (TUI)

| Key | Action |
|-----|--------|
| `Ctrl+S` | Save session |
| `Ctrl+L` | Load session |
| `Ctrl+T` | Cycle themes |
| `Ctrl+P` | Switch provider |
| `Ctrl+Q` | Quit |
| `Tab` | Switch panes |
| `Enter` | Send message |

## Themes

### Default Theme
The Claude-inspired theme with warm colors.

### Tokyo Night
Purple and blue accents inspired by the popular VSCode theme.

### Hacker Theme
Matrix-style green on black for that terminal aesthetic.

### Deep Black
High contrast theme for focused coding sessions.

```bash
# Set theme via CLI
quantumn theme set tokyo_night
quantumn theme set hacker
quantumn theme set deep_black
```

## AI Providers

### Anthropic Claude

```bash
export ANTHROPIC_API_KEY=your_key_here
quantumn model use claude-sonnet
```

### OpenAI

```bash
export OPENAI_API_KEY=your_key_here
quantumn model use gpt-4
```

### Ollama (Local)

```bash
# Start Ollama server
ollama serve

# Use local model
quantumn model use ollama://llama2
```

## Project Structure

```
QuantumnCode/
├── Cargo.toml           # Rust dependencies
├── src/
│   ├── main.rs          # Entry point
│   ├── cli.rs           # CLI argument parsing
│   ├── app.rs           # Application state
│   ├── commands/        # All CLI commands
│   ├── providers/       # AI provider implementations
│   ├── tui/             # Terminal UI components
│   ├── tools/           # File/grep/bash tools
│   └── utils/           # Utilities
└── themes/              # Theme configuration files
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by [Claude Code](https://code.claude.com)
- Built with [Ratatui](https://ratatui.rs) for the TUI
- Syntax highlighting by [syntect](https://github.com/trishume/syntect)

---

<div align="center">

Made with ❤️ by [NahanoMark](https://github.com/Akatsuki2r)

</div>