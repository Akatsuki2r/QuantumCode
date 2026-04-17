# TUI Widgets

## Overview

Quantum Code uses custom TUI widgets built on top of `ratatui` for its interactive terminal interface. This document covers the dropdown selector, tabs, and other UI components.

**Status**: 80% Implemented

## Widget Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      TUI Widgets                             │
│                                                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  Dropdown   │  │    Tabs     │  │   Chat Interface    │  │
│  │  Selector   │  │   Widget    │  │                     │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
│                                                              │
│  Built on: ratatui 0.30, crossterm 0.29                      │
└─────────────────────────────────────────────────────────────┘
```

---

## Dropdown Selector

**File**: `src/tui/widgets/dropdown.rs`

### Purpose

Interactive dropdown for selecting AI providers and models with API key prompting.

### Features

1. **Provider Selection**: Choose between Anthropic, OpenAI, Ollama, LM Studio, llama.cpp
2. **Model Selection**: View available models for selected provider
3. **API Key Prompt**: Modal when cloud provider selected without API key
4. **Keyboard Navigation**: Arrow keys, Enter, Esc

---

### State Machine

```rust
pub enum DropdownState {
    Closed,           // Collapsed, showing current selection
    Provider_Selected, // Showing provider list
    Model_Selected,   // Showing model list
    ApiKeyInput,      // Showing API key prompt
}
```

### State Transitions

```
Closed → Provider_Selected (Enter or 'p')
Provider_Selected → Model_Selected (Enter on provider)
Provider_Selected → Closed (Esc)
Model_Selected → Provider_Selected (Esc or Left)
Model_Selected → ApiKeyInput (Enter, if API key needed)
ApiKeyInput → Closed (Enter or Esc)
```

---

### Provider Info Structure

```rust
pub struct ProviderInfo {
    pub name: String,           // Internal name (e.g., "anthropic")
    pub display_name: String,   // Display name (e.g., "Anthropic (Cloud)")
    pub requires_api_key: bool, // Whether API key is required
    pub default_model: String,  // Default model for this provider
    pub models: Vec<String>,    // Available models
    pub is_local: bool,         // Local vs cloud provider
}
```

### Default Providers

```rust
fn default_providers() -> Vec<ProviderInfo> {
    vec![
        ProviderInfo::new(
            "anthropic",
            "Anthropic (Cloud)",
            true,   // requires_api_key
            "claude-sonnet-4-20250514",
            vec![
                "claude-opus-4-20250514",
                "claude-sonnet-4-20250514",
                "claude-haiku-4-20250514",
                "claude-3-5-sonnet-20241022",
                "claude-3-5-haiku-20241022",
            ],
            false,  // is_local
        ),
        ProviderInfo::new(
            "openai",
            "OpenAI (Cloud)",
            true,
            "gpt-4o",
            vec![
                "gpt-4o",
                "gpt-4o-mini",
                "gpt-4-turbo",
                "o1",
                "o1-mini",
            ],
            false,
        ),
        ProviderInfo::new(
            "ollama",
            "Ollama (Local)",
            false,  // No API key needed
            "llama3.2",
            vec![
                "llama3.2",
                "llama3.1",
                "llama3",
                "mistral",
                "codellama",
                "deepseek-coder",
                "qwen2.5-coder",
                "phi3",
                "gemma2",
            ],
            true,   // is_local
        ),
        // ... LM Studio, llama.cpp
    ]
}
```

---

### Dropdown Struct

```rust
pub struct DropdownSelector {
    pub providers: Vec<ProviderInfo>,
    pub state: DropdownState,
    pub provider_index: usize,
    pub model_index: usize,
    pub selected_provider: Option<String>,
    pub selected_model: Option<String>,
    pub api_key_input: String,
    pub api_key_env_var: String,
    pub pending_provider: Option<String>,
    pub pending_model: Option<String>,
}
```

---

### Rendering

#### Collapsed State

```rust
fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
    match self.state {
        DropdownState::Closed => {
            // Show current selection
            let display = match (&self.selected_provider, &self.selected_model) {
                (Some(p), Some(m)) => format!("{}: {}", p, m),
                _ => "Select provider...".to_string(),
            };

            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Provider ");

            let paragraph = Paragraph::new(display.as_str()).block(block);
            frame.render_widget(paragraph, area);
        }
        // ... other states
    }
}
```

#### Provider List

```
┌─────────────────────────────────────┐
│  Select Provider (↑↓ Enter Esc)     │
├─────────────────────────────────────┤
│  > [C] Anthropic (Cloud) *          │
│    [C] OpenAI (Cloud) *             │
│    [L] Ollama (Local)               │
│    [L] LM Studio (Local)            │
│    [L] llama.cpp (Local)            │
└─────────────────────────────────────┘

[C] = Cloud, [L] = Local
* = API key required
```

#### Model List

```
┌─────────────────────────────────────┐
│  Models for Anthropic (← Back)      │
├─────────────────────────────────────┤
│  > claude-opus-4-20250514           │
│    claude-sonnet-4-20250514         │
│    claude-haiku-4-20250514          │
│    claude-3-5-sonnet-20241022       │
│    claude-3-5-haiku-20241022        │
└─────────────────────────────────────┘
```

#### API Key Prompt

```
┌─────────────────────────────────────┐
│  ⚠ API Key Required                 │
├─────────────────────────────────────┤
│                                     │
│  Anthropic requires an API key.     │
│                                     │
│  Set the environment variable:      │
│                                     │
│    export ANTHROPIC_API_KEY=<key>   │
│                                     │
│  Press Enter when set, Esc to cancel│
└─────────────────────────────────────┘
```

---

### Keyboard Handling

```rust
pub fn handle_key(&mut self, key: KeyEvent) -> Option<DropdownAction> {
    use crossterm::event::{KeyCode, KeyModifiers};

    match self.state {
        DropdownState::Closed => match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Enter)
            | (KeyModifiers::NONE, KeyCode::Char('p')) => {
                self.state = DropdownState::Provider_Selected;
                Some(DropdownAction::OpenProviders)
            }
            _ => None,
        },
        DropdownState::Provider_Selected => match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Up) => {
                if self.provider_index > 0 {
                    self.provider_index -= 1;
                }
                Some(DropdownAction::Navigate)
            }
            (KeyModifiers::NONE, KeyCode::Down) => {
                if self.provider_index < self.providers.len() - 1 {
                    self.provider_index += 1;
                }
                Some(DropdownAction::Navigate)
            }
            (KeyModifiers::NONE, KeyCode::Enter) => {
                self.select_provider(self.provider_index);
                Some(DropdownAction::Provider_Selected)
            }
            (KeyModifiers::NONE, KeyCode::Esc) => {
                self.state = DropdownState::Closed;
                Some(DropdownAction::Close)
            }
            _ => None,
        },
        // ... Model_Selected, ApiKeyInput states
    }
}
```

---

### Dropdown Actions

```rust
pub enum DropdownAction {
    OpenProviders,
    Provider_Selected,
    NeedsApiKey,
    Navigate,
    Confirmed(String, String),  // (provider, model)
    Close,
    BackToProviders,
}
```

---

### API Key Checking

```rust
pub fn check_api_key_set(&self) -> bool {
    if let Some(provider) = self.get_current_provider() {
        if !provider.requires_api_key {
            return true;  // Local providers don't need API keys
        }
        let env_var = match provider.name.as_str() {
            "anthropic" => std::env::var("ANTHROPIC_API_KEY").is_ok(),
            "openai" => std::env::var("OPENAI_API_KEY").is_ok(),
            _ => true,
        };
        return env_var;
    }
    false
}

pub fn get_api_key_env_name(&self) -> Option<&str> {
    let provider = self.get_current_provider()?;
    if !provider.requires_api_key {
        return None;
    }
    match provider.name.as_str() {
        "anthropic" => Some("ANTHROPIC_API_KEY"),
        "openai" => Some("OPENAI_API_KEY"),
        _ => None,
    }
}
```

---

## Tabs Widget

**File**: `src/tui/widgets/tabs.rs`

### Purpose

Tab navigation for switching between different views (Chat, Plan, Build, Review, Debug).

### Implementation

```rust
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Tabs},
};

pub struct ModeTabs {
    pub modes: Vec<String>,
    pub selected_index: usize,
}

impl ModeTabs {
    pub fn new() -> Self {
        Self {
            modes: vec![
                "Chat".to_string(),
                "Plan".to_string(),
                "Build".to_string(),
                "Review".to_string(),
                "Debug".to_string(),
            ],
            selected_index: 0,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let titles: Vec<Line> = self.modes
            .iter()
            .map(|m| Line::from(m.as_str()))
            .collect();

        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title(" Mode "))
            .select(self.selected_index)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .bold()
                    .underline()
            );

        frame.render_widget(tabs, area);
    }

    pub fn next(&mut self) {
        self.selected_index = (self.selected_index + 1) % self.modes.len();
    }

    pub fn previous(&mut self) {
        self.selected_index = if self.selected_index == 0 {
            self.modes.len() - 1
        } else {
            self.selected_index - 1
        };
    }

    pub fn selected_mode(&self) -> &str {
        &self.modes[self.selected_index]
    }
}
```

### Visual

```
┌─────────────────────────────────────┐
│  Mode                               │
├─────────────────────────────────────┤
│ Chat │ Plan │ Build │ Review │ Debug│
│ ════ │      │       │        │      │
└─────────────────────────────────────┘
```

---

## Chat Interface

**Status**: Partially Implemented

### Purpose

Main chat view for interacting with the AI.

### Components

1. **Message List**: Scrollable list of messages
2. **Input Area**: Text input for new messages
3. **Status Bar**: Current mode, model, token count

### Implementation (Partial)

```rust
pub struct ChatView {
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub scroll_offset: usize,
}

pub struct ChatMessage {
    pub role: MessageRole,  // User, Assistant, System
    pub content: String,
    pub timestamp: DateTime<Utc>,
}
```

### TODO

- [ ] Syntax highlighting for code blocks
- [ ] Markdown rendering
- [ ] Streaming response display
- [ ] Auto-scroll on new messages
- [ ] Input history (up/down arrows)

---

## File Tree Widget

**Status**: Not Implemented

### Planned Features

- Directory tree view
- File icons by type
- Selection and opening files
- Git status indicators

### Planned API

```rust
pub struct FileTree {
    pub root: PathBuf,
    pub expanded: HashSet<PathBuf>,
    pub selected: Option<PathBuf>,
}

impl FileTree {
    pub fn new(root: PathBuf) -> Self;
    pub fn toggle_expand(&mut self, path: PathBuf);
    pub fn select_next(&mut self);
    pub fn select_previous(&mut self);
    pub fn render(&self, frame: &mut Frame, area: Rect);
}
```

---

## Diff Viewer Widget

**Status**: Not Implemented

### Planned Features

- Side-by-side diff view
- Inline diff view
- Syntax highlighting
- Hunk navigation

### Planned API

```rust
pub struct DiffViewer {
    pub old_content: String,
    pub new_content: String,
    pub view_mode: DiffViewMode,  // SideBySide, Inline
}

pub enum DiffViewMode {
    SideBySide,
    Inline,
}
```

---

## Status Bar Widget

**Status**: Partially Implemented

### Purpose

Display current status, mode, model, and token count.

### Implementation

```rust
pub struct StatusBar {
    pub mode: String,
    pub model: String,
    pub token_count: usize,
    pub provider: String,
}

impl StatusBar {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let status = format!(
            " {} | {} | {} | {} tokens ",
            self.mode,
            self.provider,
            self.model,
            self.token_count
        );

        let bar = Paragraph::new(status)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray));

        frame.render_widget(bar, area);
    }
}
```

### Visual

```
┌─────────────────────────────────────────────────────────┐
│ Build | Anthropic | claude-sonnet-4 | 1,234 tokens     │
└─────────────────────────────────────────────────────────┘
```

---

## Widget Composition

### Main TUI Layout

```rust
pub fn render_main_tui(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Tabs
            Constraint::Min(0),      // Chat area
            Constraint::Length(3),   // Input
            Constraint::Length(1),   // Status bar
        ])
        .split(frame.area());

    // Render tabs
    app.mode_tabs.render(frame, chunks[0]);

    // Render chat
    app.chat_view.render(frame, chunks[1]);

    // Render input
    app.input.render(frame, chunks[2]);

    // Render status bar
    app.status_bar.render(frame, chunks[3]);
}
```

### Layout Example

```
┌─────────────────────────────────────────┐
│ Chat │ Plan │ Build │ Review │ Debug   │ ← Tabs
├─────────────────────────────────────────┤
│                                         │
│  User: What is a mutex?                 │
│                                         │
│  Assistant: A mutex is...               │
│                                         │
│  [Chat messages...]                     │ ← Chat Area
│                                         │
│                                         │
│                                         │
├─────────────────────────────────────────┤
│ > What is a mutex?                      │ ← Input
├─────────────────────────────────────────┤
│ Chat | Anthropic | claude-sonnet | 500 │ ← Status
└─────────────────────────────────────────┘
```

---

## Testing

### Widget Tests (TODO)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dropdown_provider_selection() {
        let mut dropdown = DropdownSelector::new();
        dropdown.select_provider(0);  // Select Anthropic

        assert_eq!(dropdown.selected_provider, Some("anthropic".to_string()));
        assert_eq!(dropdown.state, DropdownState::Model_Selected);
    }

    #[test]
    fn test_tabs_navigation() {
        let mut tabs = ModeTabs::new();
        assert_eq!(tabs.selected_mode(), "Chat");

        tabs.next();
        assert_eq!(tabs.selected_mode(), "Plan");

        tabs.previous();
        assert_eq!(tabs.selected_mode(), "Chat");
    }
}
```

---

## Future Enhancements

### 1. Command Palette

Quick command access via `Ctrl+P`:

```rust
pub struct CommandPalette {
    pub commands: Vec<Command>,
    pub filter: String,
    pub selected_index: usize,
}
```

### 2. Notifications

Toast-style notifications:

```rust
pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,  // Info, Warning, Error
    pub created_at: Instant,
}
```

### 3. Progress Indicators

For long-running operations:

```rust
pub struct ProgressBar {
    pub current: usize,
    pub total: usize,
    pub label: String,
}
```

---

## Related Documentation

- [Mode System](./MODE_SYSTEM.md) - Mode definitions
- [Providers](./PROVIDERS.md) - Provider details
- [Architecture](./ARCHITECTURE.md) - Overall system design
