# UI Context: Quantum Code TUI

## Framework
- **Ratatui**: Terminal UI framework for high-performance rendering.

## Layout & Navigation
- **Tab-Based**: Switching between Chat, Context, and Activity views.
- **Multi-Pane**: Support for chat interaction and side-panel diagnostics.
- **Command Palette**: Triggered via `Ctrl+P` or slash commands for mode/theme/model switching.

## Key Components
- **Dropdown Selector**: Integrated provider and model selection with API key status indicators.
- **Kanban Board**: Task tracking and project management widget.
- **History Navigation**: Scrollable output with input history buffers and auto-scroll logic.
- **Status Bar**: Real-time display of current mode, model, and token usage.

## Visual Identity (Themes)
- **Oxidized**: Default "Rust" theme (brown/orange on black).
- **Tokyo Night**: Purple/blue accents for dark environments.
- **Hacker**: Classic Matrix-style green.
- **Deep Black**: High-contrast minimal dark mode.

## User Interaction
- **Slash Commands**: `/mode`, `/clear`, `/session`, `/config`.
- **Hotkeys**: `Ctrl+S` (Save), `Ctrl+L` (Load), `Tab` (Pane switch).