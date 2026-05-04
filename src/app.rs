//! Application state management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;

use crate::config::settings::Settings;
use crate::config::themes::Theme;
use crate::providers::Provider;
use crate::rag::{RagConfig, RagIndex};
use crate::router::RouterConfig;
use crate::tui::widgets::{DropdownSelector, KanbanBoard, TabBar};
use ratatui::widgets::ListState;

/// Current mode of the application
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    /// Default chat interaction mode
    Chat,
    /// Command palette or slash command interaction
    Command,
    /// Focused on a specific task or full-screen overlay (e.g., help, focus work)
    Focus,
}

/// A single message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role (user, assistant, system)
    pub role: String,
    /// Message content
    pub content: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Token count (if available)
    pub tokens: Option<usize>,
}

/// A file in the context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContext {
    /// File path
    pub path: String,
    /// File content
    pub content: String,
    /// Whether file is staged for context
    pub staged: bool,
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session ID
    pub id: String,
    /// Session name
    pub name: Option<String>,
    /// Creation time
    pub created: DateTime<Utc>,
    /// Last updated
    pub updated: DateTime<Utc>,
    /// Conversation messages
    pub messages: Vec<Message>,
    /// Files in context
    pub files: HashMap<String, FileContext>,
    /// Current provider
    pub provider: String,
    /// Current model
    pub model: String,
}

/// Main application state
pub struct App {
    /// Application settings
    pub settings: Settings,
    /// Current theme
    pub theme: Theme,
    /// Current session
    pub session: Session,
    /// Current mode
    pub mode: Mode,
    /// Whether the app should quit
    pub should_quit: bool,
    /// Current input buffer
    pub input: String,
    /// Cursor position in input
    pub cursor_position: usize,
    /// Scroll position in output
    pub scroll_offset: usize,
    /// Status message
    pub status: Option<String>,
    /// Available providers
    pub providers: Vec<Box<dyn Provider>>,
    /// API key status
    pub api_keys: HashMap<String, bool>,
    /// Dropdown selector for providers/models
    pub dropdown: DropdownSelector,
    /// Router configuration for automatic model selection
    pub router_config: RouterConfig,
    /// Whether automatic model switching via router is enabled
    pub router_enabled: bool,
    /// Input buffer for the command palette
    pub command_palette_input: String,
    /// Cursor position in command palette input
    pub command_palette_cursor_position: usize,
    /// Whether the command palette is active
    pub command_palette_active: bool,
    /// Last routing duration for diagnostics
    pub last_routing_duration: Option<std::time::Duration>,
    /// History of user inputs for the chat
    pub input_history: Vec<String>,
    /// Current position in history navigation
    pub history_index: Option<usize>,
    /// Whether to automatically scroll to the bottom
    pub auto_scroll: bool,
    /// RAG index for project context
    pub rag_index: RagIndex,
    /// Current git branch
    pub git_branch: Option<String>,
    /// Last time the git branch was checked
    pub last_git_check: Instant,
    /// Glob patterns for RAG indexing
    pub rag_include_patterns: Vec<String>,
}

impl App {
    /// Create a new application instance
    pub fn new(settings: Settings, theme: Theme) -> Self {
        let session_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        Self {
            settings,
            theme,
            session: Session {
                id: session_id,
                name: None,
                created: now,
                updated: now,
                messages: Vec::new(),
                files: HashMap::new(),
                provider: "anthropic".to_string(),
                model: "claude-sonnet-4-20250514".to_string(),
            },
            mode: Mode::Chat,
            should_quit: false,
            input: String::new(),
            cursor_position: 0,
            scroll_offset: 0,
            status: None,
            providers: Vec::new(),
            api_keys: HashMap::new(),
            dropdown: DropdownSelector::new(),
            router_config: RouterConfig::default(),
            router_enabled: true,
            command_palette_input: String::new(),
            command_palette_cursor_position: 0,
            command_palette_active: false,
            last_routing_duration: None,
            input_history: Vec::new(),
            history_index: None,
            auto_scroll: true,
            rag_index: RagIndex::new(RagConfig::default()),
            git_branch: Self::get_git_branch(),
            last_git_check: Instant::now(),
            rag_include_patterns: vec!["src/**/*.rs".to_string()],
        }
        .initialize()
    }

    /// Initialize application state (e.g., startup indexing)
    fn initialize(mut self) -> Self {
        self.debug_log("System: Initializing RAG index...");
        self.index_project_files();
        self
    }

    /// Update git branch if enough time has passed (30s throttle)
    pub fn update_git_status(&mut self) {
        if self.last_git_check.elapsed().as_secs() > 30 {
            self.git_branch = Self::get_git_branch();
            self.last_git_check = Instant::now();
        }
    }

    /// Force a scan of local models (Ollama/LM Studio) and update state
    pub fn refresh_local_models(&mut self) {
        let (names, _details, is_running) =
            crate::providers::ollama::OllamaProvider::detect_models_comprehensive();
        if is_running {
            tracing::debug!(target: "debug_console", "Discovered {} local models", names.len());
        }
    }

    /// Open the dropdown and synchronize it with the current session state
    pub fn open_dropdown(&mut self) {
        let provider = self.session.provider.clone();
        let model = self.session.model.clone();
        self.dropdown.open();
        self.dropdown.select(&provider, &model);
    }

    fn get_git_branch() -> Option<String> {
        std::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if branch.is_empty() {
                        None
                    } else {
                        Some(branch)
                    }
                } else {
                    None
                }
            })
    }

    /// Add a debug log entry
    pub fn debug_log(&mut self, message: &str) {
        tracing::debug!(target: "debug_console", "{}", message);
    }

    /// Toggle command palette visibility
    pub fn toggle_command_palette(&mut self) {
        self.command_palette_active = !self.command_palette_active;
        if self.command_palette_active {
            self.mode = Mode::Command;
            self.command_palette_input.clear();
            self.command_palette_cursor_position = 0;
        } else {
            self.mode = Mode::Chat;
        }
    }

    /// Search the RAG index for relevant context
    /// Uses the provided token budget to limit the number of retrieved chunks.
    pub fn search_context(&self, query: &str, token_budget: Option<usize>) -> crate::rag::RagResult {
        self.rag_index.search(query, token_budget)
    }

    /// Add a file to the RAG index
    pub fn index_file(&mut self, path: String, content: String) {
        self.rag_index.add_document(path, content);
    }

    /// Index all Rust source files in the current project
    pub fn index_project_files(&mut self) {
        use crate::tools::glob::find_files;
        use std::path::Path;

        let base = Path::new(".");
        let mut paths_to_index = HashSet::new();

        for pattern in &self.rag_include_patterns {
            if let Ok(matches) = find_files(pattern, base) {
                for path in matches {
                    paths_to_index.insert(path);
                }
            }
        }

        for path in paths_to_index {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let path_str = path.to_string_lossy().to_string();
                self.index_file(path_str, content);
            }
        }

        tracing::debug!(
            target: "rag",
            "Indexed {} files into RAG",
            self.rag_index.document_count()
        );
    }

    /// Route a prompt through the router and automatically select model
    /// Returns (provider, model) pair based on router decision
    pub fn route_prompt(&mut self, prompt: &str) -> (String, String) {
        use crate::router::{model::get_model_for_tier_with_local, route};

        if !self.router_enabled {
            // Router disabled, use current selection
            tracing::debug!(
                target: "router",
                "Router disabled, using current selection: provider={}, model={}",
                self.session.provider,
                self.session.model
            );
            return (self.session.provider.clone(), self.session.model.clone());
        }

        // Get routing decision
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "/".to_string());

        tracing::debug!(
            target: "router",
            "Routing prompt: length={}, cwd={}",
            prompt.len(),
            cwd
        );

        let decision = route(prompt, &cwd, &self.router_config);

        tracing::info!(
            target: "router",
            "Routing decision: intent={}, complexity={}, mode={}, tier={}, confidence={:.2}",
            decision.intent.as_str(),
            decision.complexity.as_str(),
            decision.mode.as_str(),
            decision.model_tier.as_str(),
            decision.confidence
        );

        // Enforce context budget from routing decision
        let max_tokens = decision.context_budget.tokens();
        self.enforce_context_budget(max_tokens);

        // Map model tier to actual provider/model
        // Local tier uses discovered Ollama models, others use cloud providers
        let model = get_model_for_tier_with_local(decision.model_tier);

        // Determine provider based on tier
        let provider = match decision.model_tier {
            crate::router::ModelTier::Local => {
                // Check if we have local models available
                if crate::router::model::has_local_models() {
                    "ollama".to_string()
                } else {
                    // Fall back to opencode if no local models
                    "opencode".to_string()
                }
            }
            crate::router::ModelTier::OpenCode => "opencode".to_string(),
            crate::router::ModelTier::Fast => "anthropic".to_string(),
            crate::router::ModelTier::Standard => "anthropic".to_string(),
            crate::router::ModelTier::Capable => "anthropic".to_string(),
        };

        tracing::info!(
            target: "router",
            "Selected: provider={}, model={}",
            provider,
            model
        );

        (provider, model)
    }

    /// Add a message to the conversation
    pub fn add_message(&mut self, role: &str, content: &str) {
        self.session.messages.push(Message {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
            tokens: None,
        });
        self.auto_scroll = true;
        self.session.updated = Utc::now();
    }

    /// Get the last message
    pub fn last_message(&self) -> Option<&Message> {
        self.session.messages.last()
    }

    /// Clear the conversation
    pub fn clear_conversation(&mut self) {
        self.session.messages.clear();
        self.session.files.clear();
        self.scroll_offset = 0;
    }

    /// Add a file to context
    pub fn add_file(&mut self, path: &str, content: &str) {
        self.session.files.insert(
            path.to_string(),
            FileContext {
                path: path.to_string(),
                content: content.to_string(),
                staged: true,
            },
        );
    }

    /// Remove a file from context
    pub fn remove_file(&mut self, path: &str) {
        self.session.files.remove(path);
    }

    /// Toggle file staged status
    pub fn toggle_file(&mut self, path: &str) {
        if let Some(file) = self.session.files.get_mut(path) {
            file.staged = !file.staged;
        }
    }

    /// Set the current mode
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    /// Quit the application
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Set status message
    pub fn set_status(&mut self, status: Option<String>) {
        self.status = status;
    }

    /// Get status message
    pub fn get_status(&self) -> Option<&String> {
        self.status.as_ref()
    }

    /// Get total tokens used in session
    pub fn total_tokens(&self) -> usize {
        self.session.messages.iter().filter_map(|m| m.tokens).sum()
    }

    /// Estimate tokens for a string (rough approximation)
    pub fn estimate_tokens(text: &str) -> usize {
        // Rough approximation: ~4 characters per token
        text.len() / 4
    }

    /// Enforce a token budget on message history by removing oldest messages
    pub fn enforce_context_budget(&mut self, max_tokens: usize) {
        if self.session.messages.is_empty() {
            return;
        }

        let mut current_total = 0;
        let mut to_keep = Vec::new();

        // Work backwards to keep the most recent messages
        for msg in self.session.messages.iter().rev() {
            // Use existing token count or estimate if missing
            let tokens = msg.tokens.unwrap_or_else(|| Self::estimate_tokens(&msg.content));
            if current_total + tokens <= max_tokens {
                current_total += tokens;
                to_keep.push(msg.clone());
            } else {
                break;
            }
        }

        if to_keep.len() < self.session.messages.len() {
            to_keep.reverse(); // Restore original order
            let removed = self.session.messages.len() - to_keep.len();
            tracing::info!(target: "app", "Trimmed {} messages to stay within {} token budget", removed, max_tokens);
            self.session.messages = to_keep;
        }
    }

    /// Save the current session to disk
    pub fn save_session(&self) -> std::io::Result<()> {
        self.session.save()
    }

    /// Load a session from disk and set it as the current session
    pub fn load_session(&mut self, id: &str) -> std::io::Result<()> {
        let session = Session::load(id)?;
        self.session = session;
        self.scroll_offset = 0;
        self.auto_scroll = true;
        self.status = Some(format!("Loaded session: {}", id));
        Ok(())
    }
}

impl Session {
    /// Create a new session
    pub fn new() -> Self {
        let session_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        Self {
            id: session_id,
            name: None,
            created: now,
            updated: now,
            messages: Vec::new(),
            files: HashMap::new(),
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
        }
    }

    /// Get the path where a session should be stored
    fn get_storage_path(id: &str) -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".config")
            .join("quantumn-code")
            .join("sessions")
            .join(format!("{}.json", id))
    }

    /// Save the session to disk as a JSON file
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::get_storage_path(&self.id);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(&path, json)?;
        tracing::info!("Session saved successfully to {:?}", path);
        Ok(())
    }

    /// Load a session from disk by its ID
    pub fn load(id: &str) -> std::io::Result<Self> {
        let path = Self::get_storage_path(id);
        let json = std::fs::read_to_string(path)?;
        let session: Session = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(session)
    }

    /// List all saved sessions from the storage directory
    pub fn list() -> Vec<Session> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let dir = PathBuf::from(home).join(".config/quantumn-code/sessions");
        let mut sessions = Vec::new();

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(session) = serde_json::from_str::<Session>(&content) {
                        sessions.push(session);
                    }
                }
            }
        }
        // Sort by most recently updated first
        sessions.sort_by(|a, b| b.updated.cmp(&a.updated));
        sessions
    }

    /// Create a named session
    pub fn with_name(name: String) -> Self {
        let mut session = Self::new();
        session.name = Some(name);
        session
    }
}
