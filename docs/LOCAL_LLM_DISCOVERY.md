# Local LLM Discovery

## Overview

Quantum Code automatically discovers locally installed LLM models from Ollama, LM Studio, and llama.cpp on startup. No manual configuration required.

**Status**: 85% Implemented

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                  Local Model Discovery                       │
│                                                              │
│  Startup → discover_all_models()                             │
│              │                                               │
│              ├── discover_ollama_models()                    │
│              │   └── Runs: ollama list                       │
│              │                                               │
│              ├── discover_lm_studio_models()                 │
│              │   └── Scans: ~/.lmstudio/models/              │
│              │                                               │
│              └── discover_llama_cpp_models()                 │
│                  └── Scans: common paths                     │
│                                                              │
│  Result: LocalModelConfig stored in memory                   │
└─────────────────────────────────────────────────────────────┘
```

## Discovery Process

**File**: `src/providers/local_discover.rs`

### Main Discovery Function

```rust
pub fn discover_all_models() -> LocalModelConfig {
    let mut config = LocalModelConfig::default();
    
    // Discover from each provider
    discover_ollama_models(&mut config);
    discover_lm_studio_models(&mut config);
    discover_llama_cpp_models(&mut config);
    
    // Set timestamp
    config.last_discovery = Some(chrono::Utc::now().to_rfc3339());
    
    config
}
```

---

## Ollama Discovery

**Status**: Complete

### How It Works

Runs `ollama list` and parses the output:

```bash
$ ollama list
NAME                    ID              SIZE      MODIFIED
llama3.2:latest         abc123          2.0GB     1 day ago
mistral:7b              def456          4.1GB     2 days ago
codellama:13b           ghi789          7.2GB     3 days ago
```

### Implementation

```rust
fn discover_ollama_models(config: &mut LocalModelConfig) {
    let output = Command::new("ollama").args(["list"]).output();
    
    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Parse output (skip header line)
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            
            if parts.len() >= 4 {
                let name = parts[0].to_string();
                let size = parts[2].to_string();
                let modified = parts[3..].join(" ");
                
                config.ollama.insert(
                    name.clone(),
                    OllamaModelInfo {
                        name,
                        size,
                        modified,
                    },
                );
            }
        }
    }
}
```

### Stored Information

```rust
pub struct OllamaModelInfo {
    pub name: String,      // e.g., "llama3.2:latest"
    pub size: String,      // e.g., "2.0GB"
    pub modified: String,  // e.g., "1 day ago"
}
```

---

## LM Studio Discovery

**Status**: Complete

### How It Works

Scans `~/.lmstudio/models/` for `.gguf` files:

```
~/.lmstudio/models/
├── meta-llama-3/
│   └── Llama-3-8B-Instruct.Q4_K_M.gguf
├── mistral-7b/
│   └── mistral-7b-instruct-v0.2.Q5_K_M.gguf
└── qwen-2.5/
    └── Qwen2.5-7B-Instruct.Q4_K_M.gguf
```

### Implementation

```rust
fn discover_lm_studio_models(config: &mut LocalModelConfig) {
    let home = std::env::var("HOME").unwrap_or_default();
    let models_dir = PathBuf::from(format!("{}/.lmstudio/models", home));
    
    if !models_dir.exists() {
        return;
    }
    
    if let Ok(entries) = std::fs::read_dir(&models_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            // Check for GGUF files in subdirectories
            if path.is_dir() {
                if let Ok(sub_entries) = std::fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let sub_path = sub_entry.path();
                        
                        if sub_path.extension().map(|e| e == "gguf").unwrap_or(false) {
                            let size = std::fs::metadata(&sub_path)
                                .map(|m| m.len())
                                .unwrap_or(0);
                            
                            let name = sub_path
                                .file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_else(|| "unknown".to_string());
                            
                            config.lm_studio.insert(
                                name.clone(),
                                LmStudioModelInfo {
                                    path: sub_path,
                                    size_bytes: size,
                                },
                            );
                        }
                    }
                }
            }
        }
    }
}
```

### Stored Information

```rust
pub struct LmStudioModelInfo {
    pub path: PathBuf,       // Full path to GGUF file
    pub size_bytes: u64,     // File size in bytes
}
```

---

## llama.cpp Discovery

**Status**: Complete

### How It Works

Scans common model locations for `.gguf` files:

**Search Paths**:
- `~/.llama.cpp/models`
- `~/models`
- `/usr/local/share/llama.cpp/models`

### Implementation

```rust
fn discover_llama_cpp_models(config: &mut LocalModelConfig) {
    let possible_paths = vec![
        PathBuf::from("~/.llama.cpp/models"),
        PathBuf::from("~/models"),
        PathBuf::from("/usr/local/share/llama.cpp/models"),
    ];
    
    for base_path in possible_paths {
        let expanded = expand_tilde(&base_path);
        if !expanded.exists() {
            continue;
        }
        
        if let Ok(entries) = std::fs::read_dir(&expanded) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if path.extension().map(|e| e == "gguf").unwrap_or(false) {
                    let size = std::fs::metadata(&path)
                        .map(|m| m.len())
                        .unwrap_or(0);
                    
                    let name = path
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    
                    config.llama_cpp.insert(
                        name.clone(),
                        LlamaCppModelInfo {
                            path,
                            size_bytes: size,
                        },
                    );
                }
            }
        }
    }
}
```

### Stored Information

```rust
pub struct LlamaCppModelInfo {
    pub path: PathBuf,       // Full path to GGUF file
    pub size_bytes: u64,     // File size in bytes
}
```

---

## Unified Model List

After discovery, all models can be retrieved as a flat list:

```rust
pub fn get_all_models(config: &LocalModelConfig) -> Vec<LocalModel> {
    let mut models = Vec::new();
    
    // Add Ollama models
    for (name, info) in &config.ollama {
        models.push(LocalModel {
            provider: "ollama".to_string(),
            name: name.clone(),
            path: format!("ollama://{}", name),
            size_bytes: parse_size(&info.size),
        });
    }
    
    // Add LM Studio models
    for (name, info) in &config.lm_studio {
        models.push(LocalModel {
            provider: "lm_studio".to_string(),
            name: name.clone(),
            path: info.path.to_string_lossy().to_string(),
            size_bytes: Some(info.size_bytes),
        });
    }
    
    // Add llama.cpp models
    for (name, info) in &config.llama_cpp {
        models.push(LocalModel {
            provider: "llama_cpp".to_string(),
            name: name.clone(),
            path: info.path.to_string_lossy().to_string(),
            size_bytes: Some(info.size_bytes),
        });
    }
    
    models
}
```

### Local Model Struct

```rust
pub struct LocalModel {
    pub provider: String,       // "ollama", "lm_studio", "llama_cpp"
    pub name: String,           // Model name
    pub path: String,           // Full path or identifier
    pub size_bytes: Option<u64>, // Size if available
}
```

---

## Size Parsing and Formatting

### Parse Size String to Bytes

```rust
fn parse_size(size_str: &str) -> Option<u64> {
    let size_str = size_str.trim();
    
    if size_str.ends_with("GB") || size_str.ends_with("G") {
        let numeric: f64 = size_str[..size_str.len() - 2].parse().ok()?;
        Some((numeric * 1024.0 * 1024.0 * 1024.0) as u64)
    } else if size_str.ends_with("MB") || size_str.ends_with("M") {
        let numeric: f64 = size_str[..size_str.len() - 2].parse().ok()?;
        Some((numeric * 1024.0 * 1024.0) as u64)
    } else if size_str.ends_with("KB") || size_str.ends_with("K") {
        let numeric: f64 = size_str[..size_str.len() - 2].parse().ok()?;
        Some((numeric * 1024.0) as u64)
    } else if size_str.ends_with("B") {
        let numeric: f64 = size_str[..size_str.len() - 1].parse().ok()?;
        Some(numeric as u64)
    } else {
        None
    }
}
```

### Format Bytes to Human Readable

```rust
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}
```

---

## Tilde Expansion

Paths with `~` are expanded to the home directory:

```rust
fn expand_tilde(path: &PathBuf) -> PathBuf {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(
                path.to_string_lossy().replacen("~", &home, 1)
            );
        }
    }
    path.clone()
}
```

---

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("4.7GB"), Some(5046586572));
        assert_eq!(parse_size("1.2MB"), Some(1258291));
        assert_eq!(parse_size("500KB"), Some(512000));
    }
    
    #[test]
    fn test_format_size() {
        assert_eq!(format_size(4_700_000_000), "4.4GB");
        assert_eq!(format_size(1_200_000), "1.1MB");
        assert_eq!(format_size(500_000), "488.3KB");
    }
}
```

---

## Usage in Dropdown Widget

**File**: `src/tui/widgets/dropdown.rs`

The dropdown widget uses discovered models:

```rust
fn default_providers() -> Vec<ProviderInfo> {
    vec![
        ProviderInfo::new(
            "ollama",
            "Ollama (Local)",
            false,  // requires_api_key
            "llama3.2",
            get_discovered_ollama_models(),  // Dynamic model list
            true,   // is_local
        ),
        // ... other providers
    ]
}
```

---

## Known Issues

### 1. Discovery Only on Startup

**Problem**: Models installed while app is running won't appear until restart.

**Fix Required**:
```rust
// Add refresh method
pub fn refresh_discovery() -> LocalModelConfig {
    discover_all_models()
}

// Call this when user runs `quantumn model list`
```

**Estimated Effort**: 2-4 hours

---

### 2. No Model Download

**Problem**: Can't download models from within the app.

**Fix Required**:
```rust
pub fn download_ollama_model(name: &str) -> Result<(), DownloadError> {
    Command::new("ollama")
        .args(["pull", name])
        .status()?;
    Ok(())
}
```

**Estimated Effort**: 2-4 hours

---

### 3. No Model Validation

**Problem**: No check if discovered models are actually loadable.

**Fix Required**:
```rust
pub fn validate_model(provider: &str, name: &str) -> Result<bool, ValidationError> {
    match provider {
        "ollama" => {
            // Try to run a test prompt
            let output = Command::new("ollama")
                .args(["run", name, "hello"])
                .output();
            Ok(output.is_ok())
        }
        // ... other providers
        _ => Err(ValidationError::UnknownProvider),
    }
}
```

**Estimated Effort**: 4-6 hours

---

## Configuration

Discovered models are stored in config:

**File**: `~/.config/quantumn-code/config.toml`

```toml
[local_models]
last_discovery = "2026-04-16T10:30:00Z"

[local_models.ollama]
llama3.2 = { name = "llama3.2:latest", size = "2.0GB", modified = "1 day ago" }
mistral = { name = "mistral:7b", size = "4.1GB", modified = "2 days ago" }

[local_models.lm_studio]
llama-3-8b = { path = "/home/user/.lmstudio/models/meta-llama-3/Llama-3-8B-Instruct.Q4_K_M.gguf", size_bytes = 4900000000 }

[local_models.llama_cpp]
mistral-7b.gguf = { path = "/home/user/models/mistral-7b.gguf", size_bytes = 4100000000 }
```

---

## Future Enhancements

### 1. Hot Reload

Watch for model changes and auto-refresh:
```rust
use notify::{Watcher, RecursiveWatcher};

pub fn watch_model_directories() -> Result<(), notify::Error> {
    let mut watcher = recommend_watcher(|res| {
        match res {
            Ok(event) => {
                if event.kind.is_create() || event.kind.is_remove() {
                    // Re-run discovery
                    let config = discover_all_models();
                    // Update UI
                }
            }
            Err(e) => eprintln!("Watch error: {:?}", e),
        }
    })?;
    
    watcher.watch(Path::new("~/.lmstudio/models"), RecursiveMode::Recursive)?;
    
    Ok(())
}
```

---

### 2. Model Recommendations

Suggest models based on hardware:
```rust
pub fn recommend_models(ram_gb: u32) -> Vec<&'static str> {
    if ram_gb >= 32 {
        vec!["llama3.1:70b", "codellama:34b"]
    } else if ram_gb >= 16 {
        vec!["llama3.2:8b", "mistral:7b", "qwen2.5-coder:7b"]
    } else {
        vec!["llama3.2:3b", "phi3:3.8b", "gemma2:2b"]
    }
}
```

---

### 3. Model Benchmarks

Run benchmarks on discovered models:
```rust
pub struct ModelBenchmark {
    pub tokens_per_second: f32,
    pub first_token_ms: u32,
    pub model_name: String,
}

pub fn benchmark_model(provider: &str, name: &str) -> Result<ModelBenchmark, BenchmarkError> {
    // Run standard prompt and measure performance
}
```

---

## Commands

### List Discovered Models

```bash
quantumn model list
```

Shows all discovered models from all providers.

---

### Show Provider-Specific Models

```bash
quantumn model ollama      # Show Ollama models
quantumn model lm_studio   # Show LM Studio models
quantumn model llama_cpp   # Show llama.cpp models
```

---

### Refresh Discovery

```bash
quantumn model refresh     # Re-run discovery
```

(TODO: Implement this command)
