//! TUI Application state and rendering

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::{App, Mode};
use crate::config::themes::Theme;

/// TUI Application
pub struct TuiApp {
    /// The main app state
    pub app: App,
    /// Theme colors converted for ratatui
    pub colors: crate::config::themes::RatatuiColors,
}

impl TuiApp {
    pub fn new(app: App) -> Self {
        let colors = app.theme.colors.to_ratatui().unwrap_or_else(|_| {
            crate::config::Theme::default_theme()
                .colors
                .to_ratatui()
                .unwrap()
        });
        Self { app, colors }
    }
}

/// Create the main layout
pub fn create_layout(frame: &Frame) -> Rect {
    frame.area()
}

/// Main render function
pub fn render(frame: &mut Frame, app: &App) {
    // Create theme colors
    let colors = match app.theme.colors.to_ratatui() {
        Ok(c) => c,
        Err(_) => {
            let default_theme = Theme::default_theme();
            default_theme.colors.to_ratatui().unwrap()
        }
    };

    // Create main layout with tabs
    let chunks = Layout::vertical([
        Constraint::Length(3), // Tab bar
        Constraint::Length(1), // Status bar
        Constraint::Min(1),    // Main content
        Constraint::Length(3), // Input
    ])
    .split(frame.area());

    // Render tab bar
    app.tab_bar.render_sleek(frame, chunks[0]);

    // Render based on active tab
    match app.tab_bar.active_index {
        1 => render_files_tab(frame, chunks[2], app, &colors),
        2 => render_kanban_tab(frame, chunks[2], app, &colors),
        3 => render_settings_tab(frame, chunks[2], app, &colors),
        _ => {
            // Always render chat underneath for context
            render_status_bar(frame, chunks[1], app, &colors);
            match app.mode {
                Mode::ProviderSelect => render_chat(frame, chunks[2], app, &colors),
                Mode::Normal => render_chat(frame, chunks[2], app, &colors),
                Mode::Help => render_help(frame, chunks[2], app, &colors),
                Mode::Editing => render_editor(frame, chunks[2], app, &colors),
                Mode::Review => render_review(frame, chunks[2], app, &colors),
                Mode::Command => render_command_palette(frame, chunks[2], app, &colors),
            }
        }
    }

    // Render input (not on settings tab)
    if app.tab_bar.active_index != 3 {
        render_input(frame, chunks[3], app, &colors);
    }

    // Render dropdown as a SINGLE centered modal overlay — only when active
    if matches!(app.mode, Mode::ProviderSelect) {
        render_dropdown_overlay(frame, app, &colors);
    }
}

/// Render the dropdown overlay for provider/model selection — single render only
fn render_dropdown_overlay(
    frame: &mut Frame,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
    // Dim / darken the background by drawing a translucent block
    let full = frame.area();
    let dim_block = Block::default()
        .style(Style::default().bg(colors.background));
    frame.render_widget(dim_block, full);

    // Centered modal — width 58 cols, height adapts to content
    let area = center_rect(58, 18, frame.area());

    // Outer modal shell — themed
    let modal_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.accent))
        .style(Style::default().bg(colors.background).fg(colors.foreground));
    frame.render_widget(modal_block, area);

    // Inner area with 1-cell padding
    let inner = Rect::new(
        area.x + 1,
        area.y + 1,
        area.width.saturating_sub(2),
        area.height.saturating_sub(2),
    );
    app.dropdown.render(frame, inner, colors);
}

/// Center a rect within another rect
fn center_rect(width: u16, height: u16, outer: Rect) -> Rect {
    let x = outer.x + (outer.width.saturating_sub(width)) / 2;
    let y = outer.y + (outer.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(outer.width), height.min(outer.height))
}

/// Render the status bar
fn render_status_bar(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
    let status = Paragraph::new(Line::from(vec![
        Span::styled(
            " Quantumn ",
            Style::default()
                .fg(colors.accent)
                .bg(colors.background)
                .bold(),
        ),
        Span::styled(
            format!(" {} ", app.session.model),
            Style::default().fg(colors.foreground).bg(colors.background),
        ),
        Span::styled(
            format!(" {} ", app.session.provider),
            Style::default().fg(colors.muted).bg(colors.background),
        ),
        Span::styled(
            format!(" {} tokens ", app.total_tokens()),
            Style::default().fg(colors.info).bg(colors.background),
        ),
        Span::styled(
            match app.mode {
                Mode::Normal => " NORMAL ",
                Mode::Editing => " EDIT ",
                Mode::Review => " REVIEW ",
                Mode::Help => " HELP ",
                Mode::Command => " COMMAND ",
                Mode::ProviderSelect => " SELECT ",
            },
            Style::default()
                .fg(colors.accent)
                .bg(colors.background)
                .bold(),
        ),
    ]))
    .style(Style::default().bg(colors.background));

    frame.render_widget(status, area);
}

/// Render the chat area
fn render_chat(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
    // Build list of messages
    let messages: Vec<Line> = app
        .session
        .messages
        .iter()
        .flat_map(|msg| {
            let role_style = match msg.role.as_str() {
                "user" => Style::default().fg(colors.accent).bold(),
                "assistant" => Style::default().fg(colors.success),
                _ => Style::default().fg(colors.muted),
            };

            let role_prefix = Span::styled(
                match msg.role.as_str() {
                    "user" => "You: ",
                    "assistant" => "AI: ",
                    _ => "System: ",
                },
                role_style,
            );

            // Wrap content into lines
            let lines: Vec<Line> = textwrap::wrap(&msg.content, area.width as usize)
                .into_iter()
                .map(|line| {
                    Line::from(Span::styled(
                        line.to_string(),
                        Style::default().fg(colors.foreground),
                    ))
                })
                .collect();

            let mut result = vec![Line::from(role_prefix)];
            result.extend(lines);
            result.push(Line::default()); // Empty line between messages

            result
        })
        .collect();

    let paragraph = Paragraph::new(messages)
        .style(Style::default().fg(colors.foreground).bg(colors.background))
        .wrap(Wrap { trim: false })
        .scroll((app.scroll_offset as u16, 0));

    frame.render_widget(paragraph, area);
}

/// Render the input area
fn render_input(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
    // Show provider/model in the input title
    let provider_text = format!("[{}:{}]", app.session.provider, app.session.model);

    // Slash command suggestion hint
    let suggestion = if app.input.starts_with('/') && app.input.len() > 1 {
        let partial = app.input[1..].to_lowercase();
        let commands = [
            "help", "clear", "quit", "exit", "provider", "model",
            "theme", "session", "config", "status", "version", "mode",
            "commit", "review", "test",
        ];
        commands
            .iter()
            .find(|c| c.starts_with(partial.as_str()) && **c != partial.as_str())
            .map(|c| format!("  Tab→ /{}", c))
            .unwrap_or_default()
    } else if app.input.is_empty() {
        "  type / for commands, p for providers".to_string()
    } else {
        String::new()
    };

    let title_line = Line::from(vec![
        Span::styled(format!(" {} ", provider_text), Style::default().fg(colors.accent)),
        Span::styled(suggestion, Style::default().fg(colors.muted)),
    ]);

    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(colors.foreground).bg(colors.background))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border))
                .title(title_line),
        );

    frame.render_widget(input, area);

    // Show cursor
    let cursor_x = (area.x + 1 + app.cursor_position as u16).min(area.x + area.width - 2);
    let cursor_y = area.y + 1;
    frame.set_cursor_position(Position {
        x: cursor_x,
        y: cursor_y,
    });
}

/// Render help screen
fn render_help(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
    let help_text = vec![
        Line::from(Span::styled(
            "Quantumn Code - Help",
            Style::default().fg(colors.accent).bold(),
        )),
        Line::default(),
        Line::from(Span::styled(
            "Keyboard Shortcuts:",
            Style::default().fg(colors.secondary).bold(),
        )),
        Line::from("  Enter       - Send message"),
        Line::from("  Ctrl+C      - Quit"),
        Line::from("  Esc         - Cancel/Exit"),
        Line::from("  Tab         - Autocomplete"),
        Line::from("  Ctrl+L      - Clear screen"),
        Line::from("  Ctrl+S      - Save session"),
        Line::from("  F1          - Toggle help"),
        Line::from("  F2          - Toggle file tree"),
        Line::from("  F3          - Toggle token count"),
        Line::from("  F4          - Change theme"),
        Line::from("  /           - Command palette"),
        Line::from("  ←→         - Switch tabs"),
        Line::from("  P           - Open provider selector"),
        Line::default(),
        Line::from(Span::styled(
            "Commands:",
            Style::default().fg(colors.secondary).bold(),
        )),
        Line::from("  /help       - Show help"),
        Line::from("  /clear      - Clear conversation"),
        Line::from("  /model      - Change model"),
        Line::from("  /theme      - Change theme"),
        Line::from("  /commit     - Generate commit"),
        Line::from("  /review     - Review code"),
        Line::from("  /test       - Run tests"),
        Line::from("  /quit       - Exit"),
        Line::default(),
        Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(colors.muted),
        )),
    ];

    let paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(colors.foreground).bg(colors.background))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border))
                .title(" Help ")
                .title_style(Style::default().fg(colors.accent)),
        );

    frame.render_widget(paragraph, area);
}

/// Render editor mode
fn render_editor(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
    let paragraph = Paragraph::new("Editor mode - Coming soon")
        .style(Style::default().fg(colors.foreground).bg(colors.background))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border))
                .title(" Editor ")
                .title_style(Style::default().fg(colors.accent)),
        );

    frame.render_widget(paragraph, area);
}

/// Render review mode
fn render_review(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
    let paragraph = Paragraph::new("Review mode - Coming soon")
        .style(Style::default().fg(colors.foreground).bg(colors.background))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border))
                .title(" Review ")
                .title_style(Style::default().fg(colors.accent)),
        );

    frame.render_widget(paragraph, area);
}

/// Render command palette
fn render_command_palette(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
    let paragraph = Paragraph::new("Command palette - Coming soon")
        .style(Style::default().fg(colors.foreground).bg(colors.background))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border))
                .title(" Commands ")
                .title_style(Style::default().fg(colors.accent)),
        );

    frame.render_widget(paragraph, area);
}

// render_provider_select removed — dropdown is now rendered exclusively
// through render_dropdown_overlay as a single centered modal.

/// Render files tab
fn render_files_tab(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
    let files: Vec<Line> = app
        .session
        .files
        .values()
        .map(|f| {
            Line::from(Span::styled(
                format!("📄 {}", f.path),
                Style::default().fg(colors.foreground),
            ))
        })
        .collect();

    let files_text = if files.is_empty() {
        vec![Line::from(Span::styled(
            "No files in context. Use /add <file> to add files.",
            Style::default().fg(colors.muted),
        ))]
    } else {
        files
    };

    let paragraph = Paragraph::new(files_text)
        .style(Style::default().fg(colors.foreground).bg(colors.background))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border))
                .title(" Files ")
                .title_style(Style::default().fg(colors.accent)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Render kanban tab
fn render_kanban_tab(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(" Kanban Board ", Style::default().fg(colors.accent).bold()),
        Span::styled(
            " - Use arrow keys to navigate, Enter to select ",
            Style::default().fg(colors.muted),
        ),
    ]))
    .style(Style::default().bg(colors.background));

    let header_area = Rect::new(area.x, area.y, area.width, 1);
    frame.render_widget(header, header_area);

    // Kanban board
    let board_area = Rect::new(area.x, area.y + 1, area.width, area.height - 1);
    app.kanban.render(frame, board_area);
}

/// Render settings tab
fn render_settings_tab(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
    let settings_text = vec![
        Line::from(Span::styled(
            "Settings",
            Style::default().fg(colors.accent).bold(),
        )),
        Line::default(),
        Line::from(format!("Provider: {}", app.session.provider)),
        Line::from(format!("Model: {}", app.session.model)),
        Line::from(format!("Theme: {}", app.settings.ui.theme)),
        Line::default(),
        Line::from(Span::styled("API Keys:", Style::default().bold())),
        Line::from(format!(
            "  Anthropic: {}",
            if *app.api_keys.get("anthropic").unwrap_or(&false) {
                "✓ Set"
            } else {
                "✗ Not set"
            }
        )),
        Line::from(format!(
            "  OpenAI: {}",
            if *app.api_keys.get("openai").unwrap_or(&false) {
                "✓ Set"
            } else {
                "✗ Not set"
            }
        )),
        Line::default(),
        Line::from(Span::styled(
            "Press P to change provider/model",
            Style::default().fg(colors.muted),
        )),
    ];

    let paragraph = Paragraph::new(settings_text)
        .style(Style::default().fg(colors.foreground).bg(colors.background))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border))
                .title(" Settings ")
                .title_style(Style::default().fg(colors.accent)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
