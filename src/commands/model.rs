//! Model/provider commands

use color_eyre::eyre::{Result, WrapErr};
use futures::StreamExt;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const DEFAULT_DRAFT_REPO: &str = "Qwen/Qwen2.5-Coder-0.5B-Instruct-GGUF";
const DEFAULT_DRAFT_FILE: &str = "qwen2.5-0.5b-coder-instruct-q5_k_m.gguf";
const DEFAULT_DRAFT_URL: &str = "https://huggingface.co/Qwen/Qwen2.5-Coder-0.5B-Instruct-GGUF/resolve/main/qwen2.5-0.5b-coder-instruct-q5_k_m.gguf?download=true";

/// Run model command
pub async fn run(
    provider: Option<String>,
    list: bool,
    enable_speculative: bool,
    yes: bool,
) -> Result<()> {
    if enable_speculative {
        enable_speculative_decoding(yes).await?;
    } else if list {
        list_models(provider)?;
    } else if let Some(p) = provider {
        set_provider(&p)?;
    } else {
        show_current_provider()?;
    }

    Ok(())
}

/// Scan LM Studio's model directory for downloaded GGUF models
fn scan_lm_studio_models() -> Vec<(String, PathBuf)> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
    let models_dir = PathBuf::from(format!("{}/.lmstudio/models", home));
    let mut models = Vec::new();

    if models_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&models_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    if let Ok(sub) = std::fs::read_dir(entry.path()) {
                        for sub_entry in sub.flatten() {
                            if sub_entry
                                .path()
                                .extension()
                                .map(|e| e == "gguf")
                                .unwrap_or(false)
                            {
                                let name = sub_entry.file_name().to_string_lossy().to_string();
                                models.push((name, sub_entry.path()));
                            }
                        }
                    }
                }
            }
        }
    }
    models
}

/// List available models
fn list_models(provider: Option<String>) -> Result<()> {
    match provider.as_deref() {
        Some("anthropic") => list_anthropic_models(),
        Some("openai") => list_openai_models(),
        Some("ollama") => list_ollama_models(),
        Some("llama_cpp") => list_llama_cpp_models(),
        Some("lm_studio") => list_lm_studio_models(),
        None => {
            println!("╔════════════════════════════════════════════════════════════════╗");
            println!("║ CLOUD MODELS                                                   ║");
            println!("╠════════════════════════════════════════════════════════════════╣");
            println!("║ ANTHROPIC (Claude)                                            ║");
            list_anthropic_models();
            println!("\n║ OPENAI                                                        ║");
            list_openai_models();

            println!("\n╔════════════════════════════════════════════════════════════════╗");
            println!("║ DOWNLOADED LOCAL MODELS                                       ║");
            println!("╠════════════════════════════════════════════════════════════════╣");
            println!("║ LM STUDIO (~/.lmstudio/models/)                              ║");
            let local_models = scan_lm_studio_models();
            if !local_models.is_empty() {
                for (name, path) in &local_models {
                    let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                    let size_mb = size as f64 / (1024.0 * 1024.0);
                    let display_name = path
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| name.clone());
                    println!("║   {} ({:.1} MB)", display_name, size_mb);
                }
            } else {
                println!("║   No GGUF models found");
            }

            println!("\n║ OLLAMA                                                        ║");
            list_ollama_models();

            println!("\n║ LLAMA.CPP                                                    ║");
            list_llama_cpp_models();

            println!();
            println!("To set a provider: quantumn model <provider_name>");
        }
        Some(p) => {
            println!("Unknown provider: {}", p);
            println!("Available: anthropic, openai, ollama, llama_cpp, lm_studio");
        }
    }

    Ok(())
}

fn list_anthropic_models() {
    println!("  claude-opus-4-20250514   - Most capable (Opus 4)");
    println!("  claude-sonnet-4-20250514 - Balanced (Sonnet 4) [default]");
    println!("  claude-haiku-4-20250514  - Fast (Haiku 4)");
    println!("  claude-3-5-sonnet-20241022 - Legacy (Sonnet 3.5)");
    println!("  claude-3-5-haiku-20241022  - Legacy (Haiku 3.5)");
}

fn list_openai_models() {
    println!("  gpt-4o       - GPT-4 Omni (recommended)");
    println!("  gpt-4o-mini  - GPT-4 Omni Mini (fast, cheap)");
    println!("  gpt-4-turbo  - GPT-4 Turbo");
    println!("  o1           - O1 (advanced reasoning)");
    println!("  o1-mini      - O1 Mini");
}

fn list_ollama_models() {
    // Try to detect actual Ollama models
    let output = std::process::Command::new("ollama").args(["list"]).output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            println!("  Installed Ollama models:");
            // Skip header line
            for line in stdout.lines().skip(1) {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                // Parse NAME SIZE MODIFIED
                let parts: Vec<&str> = line.split_whitespace().collect();
                if !parts.is_empty() {
                    println!("  {}", parts[0]);
                }
            }
        }
        _ => {
            println!("  Ollama not running or no models installed.");
            println!("  Common models (not installed):");
            println!("  llama3.2       - Meta Llama 3.2");
            println!("  llama3.1       - Meta Llama 3.1");
            println!("  mistral        - Mistral");
            println!("  codellama      - Code Llama");
            println!("  deepseek-coder - DeepSeek Coder");
            println!("  qwen2.5-coder  - Qwen 2.5 Coder");
            println!();
            println!("  Install: ollama pull <model_name>");
        }
    }
    println!();
    println!("  Note: Run 'ollama list' to see installed models.");
    println!("  Or use 'quantumn agent --auto-detect' to auto-detect local LLMs");
}

fn list_llama_cpp_models() {
    println!("  Configured models from settings:");
    if let Ok(settings) = crate::config::Settings::load() {
        if settings.llama_cpp.model_paths.is_empty() {
            println!("  (none configured - add to [llama_cpp.model_paths] in config.toml)");
        } else {
            for (name, path) in &settings.llama_cpp.model_paths {
                println!("  {} -> {}", name, path);
            }
        }
    }
    println!();
    println!("  Common models:");
    println!("  llama3.2      - Meta Llama 3.2 (GGUF)");
    println!("  llama3.1      - Meta Llama 3.1 (GGUF)");
    println!("  mistral      - Mistral (GGUF)");
    println!("  qwen2.5      - Qwen 2.5 (GGUF)");
    println!("  deepseek-coder - DeepSeek Coder (GGUF)");
    println!();
    println!("  Requires llama-server binary and GGUF model files.");
    println!("  Configure model paths in config.toml under [llama_cpp.model_paths]");
    println!("  Optional speedup: quantumn model --enable-speculative");
}

fn list_lm_studio_models() {
    // Try to scan LM Studio models directory
    let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
    let models_dir = std::path::PathBuf::from(format!("{}/.lmstudio/models", home));
    let mut found_any = false;

    if models_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&models_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    if let Ok(sub) = std::fs::read_dir(entry.path()) {
                        for sub_entry in sub.flatten() {
                            if sub_entry
                                .path()
                                .extension()
                                .map(|e| e == "gguf")
                                .unwrap_or(false)
                            {
                                let name = sub_entry.file_name().to_string_lossy().to_string();
                                if let Ok(meta) = std::fs::metadata(&sub_entry.path()) {
                                    let size_mb = meta.len() as f64 / (1024.0 * 1024.0);
                                    println!("  {} ({:.1} MB)", name, size_mb);
                                    found_any = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if !found_any {
        println!("  No GGUF models found in ~/.lmstudio/models/");
    }
    println!();
    println!("  LM Studio manages models directly.");
    println!("  Start LM Studio server: lms server start");
    println!("  Or download models through LM Studio application.");
}

/// Set current provider
fn set_provider(provider: &str) -> Result<()> {
    let mut settings = crate::config::Settings::load()?;

    let (provider_name, default_model) = match provider {
        "anthropic" => ("anthropic", "claude-sonnet-4-20250514"),
        "openai" => ("openai", "gpt-4o"),
        "ollama" => ("ollama", "llama3.2"),
        "llama_cpp" => ("llama_cpp", "llama3.2"),
        "lm_studio" => ("lm_studio", "llama3.2"),
        _ => {
            println!("Unknown provider: {}", provider);
            println!("Available providers: anthropic, openai, ollama, llama_cpp, lm_studio");
            return Ok(());
        }
    };

    settings.model.provider = provider_name.to_string();
    settings.model.default_model = default_model.to_string();
    settings.save()?;

    println!("✓ Provider set to: {}", provider);
    println!("  Default model: {}", default_model);

    if provider == "llama_cpp" {
        println!();
        println!("Note: llama.cpp requires:");
        println!("  1. llama-server binary in PATH");
        println!("  2. GGUF model files configured in config.toml");
        println!("  3. Or use Ollama as fallback (enabled by default)");
    } else if provider == "lm_studio" {
        println!();
        println!("Note: LM Studio requires:");
        println!("  1. LM Studio application running");
        println!("  2. lms server start (or enable server in LM Studio GUI)");
        println!("  3. Models downloaded in LM Studio library");
    }

    Ok(())
}

/// Prompt the user, download the default draft model, and enable speculative decoding.
async fn enable_speculative_decoding(assume_yes: bool) -> Result<()> {
    println!("Quantumn local inference optimization: speculative decoding");
    println!();
    println!("This enables llama.cpp draft-model speculation for auto-started llama-server.");
    println!("It downloads a small code-oriented draft model:");
    println!("  {} / {}", DEFAULT_DRAFT_REPO, DEFAULT_DRAFT_FILE);
    println!();
    println!("Best fit: Qwen/Qwen-Coder main models. For Llama/Mistral main models, set");
    println!("llama_cpp.draft_model_path manually to a tiny model from the same tokenizer family.");
    println!("This downloads only the draft model; your main llama.cpp GGUF still needs to be configured.");
    println!();

    if !assume_yes {
        print!("Download the draft model and enable this optimization now? [y/N] ");
        io::stdout().flush().ok();

        let mut answer = String::new();
        io::stdin()
            .read_line(&mut answer)
            .wrap_err("Failed to read confirmation")?;

        let answer = answer.trim().to_ascii_lowercase();
        if answer != "y" && answer != "yes" {
            println!("Aborted. No config changes were made.");
            return Ok(());
        }
    }

    let mut settings = crate::config::Settings::load()?;
    let config_path = crate::config::Settings::config_path()?;
    let config_dir = config_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(".quantumn"));
    let draft_dir = config_dir.join("models").join("draft");
    std::fs::create_dir_all(&draft_dir).wrap_err("Failed to create draft model directory")?;

    let draft_path = draft_dir.join(DEFAULT_DRAFT_FILE);
    if draft_path.exists() {
        println!("✓ Draft model already exists: {}", draft_path.display());
    } else {
        download_file(DEFAULT_DRAFT_URL, &draft_path).await?;
    }

    settings.model.provider = "llama_cpp".to_string();
    settings.llama_cpp.enabled = true;
    settings.llama_cpp.auto_start = true;
    settings.llama_cpp.speculative_decoding = true;
    settings.llama_cpp.draft_model_path = Some(draft_path.to_string_lossy().to_string());
    settings.llama_cpp.draft_max = 16;
    settings.llama_cpp.draft_min = 0;
    settings.llama_cpp.draft_p_min = 0.75;
    settings.save()?;

    println!();
    println!("✓ Speculative decoding enabled for llama.cpp.");
    println!("  Draft model: {}", draft_path.display());
    println!("  Config: {}", config_path.display());
    println!("  Tune with: quantumn config set llama_cpp.draft_max 16");
    if settings.llama_cpp.model_paths.is_empty() {
        println!("  Next: add your main GGUF under [llama_cpp.model_paths] in config.toml.");
    }

    Ok(())
}

async fn download_file(url: &str, destination: &Path) -> Result<()> {
    let part_path = destination.with_extension("gguf.part");
    println!("Downloading draft model...");
    println!("  {}", url);
    println!("  -> {}", destination.display());

    let response = reqwest::get(url)
        .await
        .wrap_err("Failed to start draft model download")?
        .error_for_status()
        .wrap_err("Draft model download returned an error status")?;

    let total = response.content_length();
    let mut stream = response.bytes_stream();
    let mut file =
        std::fs::File::create(&part_path).wrap_err("Failed to create partial model file")?;
    let mut downloaded: u64 = 0;
    let mut last_reported_mb: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.wrap_err("Failed while downloading draft model")?;
        file.write_all(&chunk)
            .wrap_err("Failed to write draft model chunk")?;
        downloaded += chunk.len() as u64;

        let downloaded_mb = downloaded / (1024 * 1024);
        if downloaded_mb >= last_reported_mb + 64 {
            last_reported_mb = downloaded_mb;
            if let Some(total) = total {
                let total_mb = total / (1024 * 1024);
                println!("  {} / {} MB", downloaded_mb, total_mb);
            } else {
                println!("  {} MB", downloaded_mb);
            }
        }
    }

    file.flush().wrap_err("Failed to flush draft model file")?;
    std::fs::rename(&part_path, destination).wrap_err("Failed to finalize draft model download")?;
    println!("✓ Download complete");

    Ok(())
}

/// Show current provider and model
fn show_current_provider() -> Result<()> {
    let settings = crate::config::Settings::load()?;

    println!("Current provider: {}", settings.model.provider);
    println!("Current model: {}", settings.model.default_model);
    println!();

    // Check if API key is set
    let api_key_env = &settings.model.api_key_env;
    let has_key = std::env::var(api_key_env).is_ok();

    if has_key {
        println!("API key: ✓ Set ({})", api_key_env);
    } else {
        println!("API key: ✗ Not set");
        println!();
        println!("To set your API key:");
        println!("  export {}=your-api-key", api_key_env);
    }

    // Show llama.cpp config if relevant
    if settings.model.provider == "llama_cpp" {
        println!();
        println!("llama.cpp configuration:");
        println!("  Enabled: {}", settings.llama_cpp.enabled);
        println!("  Port: {}", settings.llama_cpp.default_port);
        println!(
            "  Fallback to Ollama: {}",
            settings.llama_cpp.fallback_to_ollama
        );
        println!("  Auto-start: {}", settings.llama_cpp.auto_start);
        println!(
            "  Speculative decoding: {}",
            settings.llama_cpp.speculative_decoding
        );
        if let Some(path) = &settings.llama_cpp.draft_model_path {
            println!("  Draft model: {}", path);
            println!(
                "  Draft params: max={} min={} p_min={}",
                settings.llama_cpp.draft_max,
                settings.llama_cpp.draft_min,
                settings.llama_cpp.draft_p_min
            );
        }
        if !settings.llama_cpp.model_paths.is_empty() {
            println!("  Model paths:");
            for (name, path) in &settings.llama_cpp.model_paths {
                println!("    {}: {}", name, path);
            }
        }
    }

    // Show LM Studio config if relevant
    if settings.model.provider == "lm_studio" {
        println!();
        println!("LM Studio configuration:");
        println!("  Enabled: {}", settings.lm_studio.enabled);
        println!("  Base URL: {}", settings.lm_studio.base_url);
        if settings.lm_studio.api_token.is_some() {
            println!("  API Token: ✓ Set");
        } else {
            println!("  API Token: Not set (optional)");
        }
        if !settings.lm_studio.model_paths.is_empty() {
            println!("  Model paths:");
            for (name, path) in &settings.lm_studio.model_paths {
                println!("    {}: {}", name, path);
            }
        }
    }

    Ok(())
}

/// Run provider command to show all providers
pub async fn run_provider(_list: bool) -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║ AVAILABLE AI PROVIDERS                                          ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║ ANTHROPIC (Cloud)                                             ║");
    println!("║   Provider: anthropic                                          ║");
    println!("║   Default: claude-sonnet-4-20250514                            ║");
    println!("║   Models: claude-opus-4, claude-sonnet-4, claude-haiku-4      ║");
    println!("║   Setup: export ANTHROPIC_API_KEY=your_key                      ║");
    println!();
    println!("║ OPENAI (Cloud)                                                 ║");
    println!("║   Provider: openai                                             ║");
    println!("║   Default: gpt-4o                                              ║");
    println!("║   Models: gpt-4o, gpt-4o-mini, gpt-4-turbo, o1, o1-mini        ║");
    println!("║   Setup: export OPENAI_API_KEY=your_key                        ║");
    println!();
    println!("║ OLLAMA (Local)                                                 ║");
    println!("║   Provider: ollama                                             ║");
    println!("║   Default: llama3.2                                            ║");
    println!("║   Setup: ollama serve && ollama pull llama3.2                  ║");
    println!();
    println!("║ LM STUDIO (Local)                                              ║");
    println!("║   Provider: lm_studio                                           ║");
    println!("║   Default: llama3.2                                             ║");
    println!("║   Setup: lms server start OR LM Studio GUI                    ║");
    println!();
    println!("║ LLAMA.CPP (Local)                                              ║");
    println!("║   Provider: llama_cpp                                          ║");
    println!("║   Default: llama3.2                                             ║");
    println!("║   Setup: llama-server binary + GGUF model files                ║");
    println!();
    println!("To switch: quantumn model <provider_name>");

    Ok(())
}
