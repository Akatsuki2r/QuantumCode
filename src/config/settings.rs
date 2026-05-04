//! User settings configuration

use color_eyre::eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Default provider options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderType {
    Anthropic,
    OpenAI,
    Ollama,
    LlamaCpp,
    LmStudio,
}

/// llama.cpp configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlamaCppConfig {
    /// Enable llama.cpp provider
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Default port for llama.cpp server
    #[serde(default = "default_llama_cpp_port")]
    pub default_port: u16,
    /// Model name to GGUF file path mapping
    #[serde(default)]
    pub model_paths: HashMap<String, String>,
    /// Fallback to Ollama if llama.cpp fails
    #[serde(default = "default_true")]
    pub fallback_to_ollama: bool,
    /// Auto-start llama.cpp server when needed
    #[serde(default)]
    pub auto_start: bool,
    /// Enable speculative decoding when Quantumn auto-starts llama-server
    #[serde(default)]
    pub speculative_decoding: bool,
    /// Draft GGUF model path used by llama.cpp speculative decoding
    #[serde(default)]
    pub draft_model_path: Option<String>,
    /// Maximum draft tokens proposed per verification step
    #[serde(default = "default_draft_max")]
    pub draft_max: u16,
    /// Minimum draft tokens before accepting speculative batches
    #[serde(default = "default_draft_min")]
    pub draft_min: u16,
    /// Minimum probability threshold for draft tokens
    #[serde(default = "default_draft_p_min")]
    pub draft_p_min: f32,
}

fn default_draft_max() -> u16 {
    16
}

fn default_llama_cpp_port() -> u16 {
    8080
}

fn default_true() -> bool {
    true
}

fn default_draft_min() -> u16 {
    5
}

fn default_draft_p_min() -> f32 {
    0.9
}

impl Default for LlamaCppConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_port: 8080,
            model_paths: HashMap::new(),
            fallback_to_ollama: true,
            auto_start: false, // Disabled by default since it requires llama-server binary
            speculative_decoding: false,
            draft_model_path: None,
            draft_max: default_draft_max(),
            draft_min: default_draft_min(),
            draft_p_min: default_draft_p_min(),
        }
    }
}

/// LM Studio configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LmStudioConfig {
    /// Enable LM Studio provider
    pub enabled: bool,
    /// Base URL for LM Studio server
    pub base_url: String,
    /// API token (optional, from LM Studio developer settings)
    pub api_token: Option<String>,
    /// Model name to model ID mapping (for custom model names)
    pub model_paths: HashMap<String, String>,
}

impl Default for LmStudioConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            base_url: "http://localhost:1234".to_string(),
            api_token: None,
            model_paths: HashMap::new(),
        }
    }
}

/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Provider name
    pub provider: String,
    /// Default model
    pub default_model: String,
    /// API key (stored securely, not in plain text)
    pub api_key_env: String,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            provider: "anthropic".to_string(),
            default_model: "claude-sonnet-4-20250514".to_string(),
            api_key_env: "ANTHROPIC_API_KEY".to_string(),
        }
    }
}

/// Git configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    /// Auto-commit message format
    pub commit_format: String,
    /// Include co-authors
    pub include_coauthors: bool,
    /// Use conventional commits
    pub conventional_commits: bool,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            commit_format: "{type}: {description}".to_string(),
            include_coauthors: true,
            conventional_commits: true,
        }
    }
}

/// Editor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    /// Tab width
    pub tab_width: usize,
    /// Use spaces instead of tabs
    pub use_spaces: bool,
    /// Show line numbers
    pub line_numbers: bool,
    /// Auto-save interval (seconds)
    pub auto_save: Option<u64>,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_width: 4,
            use_spaces: true,
            line_numbers: true,
            auto_save: Some(30),
        }
    }
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    /// Default theme
    pub theme: String,
    /// Show file tree
    pub show_file_tree: bool,
    /// Show token count
    pub show_token_count: bool,
    /// Show cost tracker
    pub show_cost: bool,
    /// Animation speed (0 = off, 1-10)
    pub animation_speed: u8,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            theme: "oxidized".to_string(),
            show_file_tree: true,
            show_token_count: true,
            show_cost: true,
            animation_speed: 5,
        }
    }
}

/// Main settings structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Model configuration
    pub model: ModelConfig,
    /// Git configuration
    pub git: GitConfig,
    /// Editor configuration
    pub editor: EditorConfig,
    /// UI configuration
    pub ui: UIConfig,
    /// Custom keybindings
    #[serde(default)]
    pub keybindings: std::collections::HashMap<String, String>,
    /// llama.cpp configuration
    #[serde(default)]
    pub llama_cpp: LlamaCppConfig,
    /// LM Studio configuration
    #[serde(default)]
    pub lm_studio: LmStudioConfig,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            model: ModelConfig::default(),
            git: GitConfig::default(),
            editor: EditorConfig::default(),
            ui: UIConfig::default(),
            keybindings: std::collections::HashMap::new(),
            llama_cpp: LlamaCppConfig::default(),
            lm_studio: LmStudioConfig::default(),
        }
    }
}

impl Settings {
    /// Get the configuration file path
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = directories::ProjectDirs::from("com", "quantumn", "code")
            .map(|dirs| dirs.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".quantumn"));

        Ok(config_dir.join("config.toml"))
    }

    /// Load settings from file
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if path.exists() {
            let content = fs::read_to_string(&path).wrap_err("Failed to read config file")?;
            let settings: Settings =
                toml::from_str(&content).wrap_err("Failed to parse config file")?;
            Ok(settings)
        } else {
            let settings = Self::default();
            settings.save()?;
            Ok(settings)
        }
    }

    /// Save settings to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).wrap_err("Failed to create config directory")?;
        }

        let content = toml::to_string_pretty(self).wrap_err("Failed to serialize settings")?;

        fs::write(&path, content).wrap_err("Failed to write config file")?;

        Ok(())
    }

    /// Get a setting by key path (e.g., "ui.theme")
    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "model.provider" => Some(self.model.provider.clone()),
            "model.default_model" => Some(self.model.default_model.clone()),
            "model.api_key_env" => Some(self.model.api_key_env.clone()),
            "git.commit_format" => Some(self.git.commit_format.clone()),
            "git.include_coauthors" => Some(self.git.include_coauthors.to_string()),
            "git.conventional_commits" => Some(self.git.conventional_commits.to_string()),
            "editor.tab_width" => Some(self.editor.tab_width.to_string()),
            "editor.use_spaces" => Some(self.editor.use_spaces.to_string()),
            "editor.line_numbers" => Some(self.editor.line_numbers.to_string()),
            "editor.auto_save" => Some(
                self.editor
                    .auto_save
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
            ),
            "ui.theme" => Some(self.ui.theme.clone()),
            "ui.show_file_tree" => Some(self.ui.show_file_tree.to_string()),
            "ui.show_token_count" => Some(self.ui.show_token_count.to_string()),
            "ui.show_cost" => Some(self.ui.show_cost.to_string()),
            "ui.animation_speed" => Some(self.ui.animation_speed.to_string()),
            "llama_cpp.enabled" => Some(self.llama_cpp.enabled.to_string()),
            "llama_cpp.default_port" => Some(self.llama_cpp.default_port.to_string()),
            "llama_cpp.fallback_to_ollama" => Some(self.llama_cpp.fallback_to_ollama.to_string()),
            "llama_cpp.auto_start" => Some(self.llama_cpp.auto_start.to_string()),
            "llama_cpp.speculative_decoding" => {
                Some(self.llama_cpp.speculative_decoding.to_string())
            }
            "llama_cpp.draft_model_path" => self.llama_cpp.draft_model_path.clone(),
            "llama_cpp.draft_max" => Some(self.llama_cpp.draft_max.to_string()),
            "llama_cpp.draft_min" => Some(self.llama_cpp.draft_min.to_string()),
            "llama_cpp.draft_p_min" => Some(self.llama_cpp.draft_p_min.to_string()),
            "lm_studio.enabled" => Some(self.lm_studio.enabled.to_string()),
            "lm_studio.base_url" => Some(self.lm_studio.base_url.clone()),
            _ => None,
        }
    }

    /// Set a setting by key path
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "model.provider" => self.model.provider = value.to_string(),
            "model.default_model" => self.model.default_model = value.to_string(),
            "model.api_key_env" => self.model.api_key_env = value.to_string(),
            "git.commit_format" => self.git.commit_format = value.to_string(),
            "git.include_coauthors" => {
                self.git.include_coauthors = value.parse().wrap_err("Invalid boolean value")?
            }
            "git.conventional_commits" => {
                self.git.conventional_commits = value.parse().wrap_err("Invalid boolean value")?
            }
            "editor.tab_width" => {
                self.editor.tab_width = value.parse().wrap_err("Invalid number")?
            }
            "editor.use_spaces" => {
                self.editor.use_spaces = value.parse().wrap_err("Invalid boolean value")?
            }
            "editor.line_numbers" => {
                self.editor.line_numbers = value.parse().wrap_err("Invalid boolean value")?
            }
            "editor.auto_save" => {
                self.editor.auto_save = Some(value.parse().wrap_err("Invalid number")?)
            }
            "ui.theme" => self.ui.theme = value.to_string(),
            "ui.show_file_tree" => {
                self.ui.show_file_tree = value.parse().wrap_err("Invalid boolean value")?
            }
            "ui.show_token_count" => {
                self.ui.show_token_count = value.parse().wrap_err("Invalid boolean value")?
            }
            "ui.show_cost" => {
                self.ui.show_cost = value.parse().wrap_err("Invalid boolean value")?
            }
            "ui.animation_speed" => {
                self.ui.animation_speed = value.parse().wrap_err("Invalid number")?
            }
            "llama_cpp.enabled" => {
                self.llama_cpp.enabled = value.parse().wrap_err("Invalid boolean value")?
            }
            "llama_cpp.default_port" => {
                self.llama_cpp.default_port = value.parse().wrap_err("Invalid port number")?
            }
            "llama_cpp.fallback_to_ollama" => {
                self.llama_cpp.fallback_to_ollama =
                    value.parse().wrap_err("Invalid boolean value")?
            }
            "llama_cpp.auto_start" => {
                self.llama_cpp.auto_start = value.parse().wrap_err("Invalid boolean value")?
            }
            "llama_cpp.speculative_decoding" => {
                self.llama_cpp.speculative_decoding =
                    value.parse().wrap_err("Invalid boolean value")?
            }
            "llama_cpp.draft_model_path" => {
                self.llama_cpp.draft_model_path = if value.trim().is_empty() {
                    None
                } else {
                    Some(value.to_string())
                }
            }
            "llama_cpp.draft_max" => {
                self.llama_cpp.draft_max = value.parse().wrap_err("Invalid number")?
            }
            "llama_cpp.draft_min" => {
                self.llama_cpp.draft_min = value.parse().wrap_err("Invalid number")?
            }
            "llama_cpp.draft_p_min" => {
                self.llama_cpp.draft_p_min = value.parse().wrap_err("Invalid number")?
            }
            "lm_studio.enabled" => {
                self.lm_studio.enabled = value.parse().wrap_err("Invalid boolean value")?
            }
            "lm_studio.base_url" => self.lm_studio.base_url = value.to_string(),
            _ => return Err(color_eyre::eyre::eyre!("Unknown setting: {}", key)),
        }
        Ok(())
    }
}
