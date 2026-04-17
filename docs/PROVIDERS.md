# Provider System

## Overview

Quantum Code supports multiple AI providers through a unified trait-based abstraction. This allows seamless switching between cloud providers (Anthropic, OpenAI) and local models (Ollama, LM Studio, llama.cpp).

## Provider Architecture

```
┌─────────────────────────────────────────┐
│           Provider Trait                 │
│  - chat()                               │
│  - stream()                             │
│  - get_models()                         │
│  - requires_api_key()                   │
└─────────────────┬───────────────────────┘
                  │
    ┌─────────────┼─────────────┬─────────────┬──────────────┐
    │             │             │             │              │
┌───▼───┐   ┌────▼────┐   ┌───▼────┐  ┌────▼─────┐  ┌────▼──────┐
│Anthropic│ │ OpenAI  │   │ Ollama  │  │LM Studio │  │llama.cpp  │
│Provider │ │Provider │   │Provider │  │Provider  │  │Provider   │
└─────────┘ └─────────┘   └─────────┘  └──────────┘  └───────────┘
```

## Provider Trait

**File**: `src/providers/provider_trait.rs`

```rust
pub trait Provider {
    /// Send a chat message and get a response
    fn chat(&self, messages: Vec<Message>) -> Result<String, ProviderError>;
    
    /// Stream a response
    fn stream(&self, messages: Vec<Message>) -> Result<impl Stream<Item = StreamChunk>, ProviderError>;
    
    /// Get available models
    fn get_models(&self) -> Result<Vec<String>, ProviderError>;
    
    /// Whether this provider requires an API key
    fn requires_api_key(&self) -> bool;
}
```

## Provider Implementations

### 1. Anthropic Provider

**File**: `src/providers/anthropic.rs`

**Type**: Cloud
**API Key Required**: Yes
**Environment Variable**: `ANTHROPIC_API_KEY`

**Models**:
- `claude-opus-4-20250514` - Most capable
- `claude-sonnet-4-20250514` - Balanced (default)
- `claude-haiku-4-20250514` - Fast, cheap

**Pricing**:
| Model | Input/1M | Output/1M |
|-------|----------|-----------|
| Opus | $15 | $75 |
| Sonnet | $3 | $15 |
| Haiku | $0.25 | $1.25 |

**Setup**:
```bash
export ANTHROPIC_API_KEY=sk-ant-...
quantumn model anthropic
```

### 2. OpenAI Provider

**File**: `src/providers/openai.rs`

**Type**: Cloud
**API Key Required**: Yes
**Environment Variable**: `OPENAI_API_KEY`

**Models**:
- `gpt-4o` - GPT-4 Omni (recommended)
- `gpt-4o-mini` - Fast, cheap
- `gpt-4-turbo` - GPT-4 Turbo
- `o1` - Advanced reasoning
- `o1-mini` - Reasoning, cheap

**Pricing**:
| Model | Input/1M | Output/1M |
|-------|----------|-----------|
| GPT-4o | $5 | $15 |
| GPT-4o-mini | $0.15 | $0.60 |

**Setup**:
```bash
export OPENAI_API_KEY=sk-...
quantumn model openai
```

### 3. Ollama Provider

**File**: `src/providers/ollama.rs`

**Type**: Local
**API Key Required**: No
**Server**: `http://localhost:11434`

**Models** (auto-discovered):
- `llama3.2` - General purpose
- `llama3.1` - Meta Llama
- `mistral` - Mistral AI
- `codellama` - Code-specialized
- `deepseek-coder` - DeepSeek coding
- `qwen2.5-coder` - Qwen coding
- `phi3` - Microsoft Phi
- `gemma2` - Google Gemma

**Setup**:
```bash
# Install Ollama
curl https://ollama.ai/install.sh | sh

# Start server
ollama serve

# Download models
ollama pull llama3.2
ollama pull mistral

# Use with Quantum Code
quantumn model ollama
```

### 4. LM Studio Provider

**File**: `src/providers/lm_studio.rs`

**Type**: Local
**API Key Required**: No
**Server**: `http://localhost:1234/v1`

**Models**: Auto-discovered from `~/.lmstudio/models/`

**Setup**:
```bash
# Download LM Studio
# https://lmstudio.ai

# Download a model through the app
# Start Local Inference Server

# Use with Quantum Code
quantumn model lm_studio
```

### 5. llama.cpp Provider

**File**: `src/providers/llama_cpp.rs`

**Type**: Local
**API Key Required**: No
**Server**: User-managed llama.cpp server

**Models**: GGUF format files from common locations

**Setup**:
```bash
# Build llama.cpp
git clone https://github.com/ggerganov/llama.cpp
cd llama.cpp && make

# Download GGUF model
# Configure in ~/.config/quantumn-code/config.toml

# Use with Quantum Code
quantumn model llama_cpp
```

## Local Model Discovery

**File**: `src/providers/local_discover.rs`

### Auto-Discovery Process

The `discover_all_models()` function runs on startup to find installed local models:

```rust
pub fn discover_all_models() -> LocalModelConfig {
    let mut config = LocalModelConfig::default();
    
    discover_ollama_models(&mut config);
    discover_lm_studio_models(&mut config);
    discover_llama_cpp_models(&mut config);
    
    config.last_discovery = Some(chrono::Utc::now().to_rfc3339());
    config
}
```

### Ollama Discovery

```bash
ollama list
```

Parses output:
```
NAME                    ID          SIZE      MODIFIED
llama3.2:latest         abc123      2.0GB     1 day ago
mistral:7b              def456      4.1GB     2 days ago
```

### LM Studio Discovery

Scans `~/.lmstudio/models/` for `.gguf` files.

### llama.cpp Discovery

Scans common locations:
- `~/.llama.cpp/models`
- `~/models`
- `/usr/local/share/llama.cpp/models`

## Provider Selection UI

**File**: `src/tui/widgets/dropdown.rs`

The dropdown widget provides a UI for selecting providers:

### Features

1. **Provider List**: Shows all available providers
   - `[L]` = Local (no API key)
   - `[C]` = Cloud (API key required)

2. **Model Selection**: After selecting a provider, shows available models

3. **API Key Prompt**: If a cloud provider is selected without an API key set:
   - Shows modal with environment variable to set
   - Waits for user confirmation

### Usage

```rust
let mut dropdown = DropdownSelector::new();

// Open dropdown
dropdown.state = DropdownState::ProviderSelected;

// Navigate
dropdown.handle_key(KeyEvent { code: KeyCode::Down, .. });

// Select provider
dropdown.handle_key(KeyEvent { code: KeyCode::Enter, .. });

// Get selection
let (provider, model) = dropdown.confirm_selection()?;
```

## Provider Comparison

| Provider | Type | Models | API Key | Latency | Cost |
|----------|------|--------|---------|---------|------|
| Anthropic | Cloud | 5+ | Yes | Moderate | $$$ |
| OpenAI | Cloud | 5+ | Yes | Moderate | $$ |
| Ollama | Local | Any | No | Fast | Free |
| LM Studio | Local | Any | No | Fast | Free |
| llama.cpp | Local | GGUF | No | Fastest | Free |

## Adding a New Provider

To add a new provider:

1. **Create provider file**: `src/providers/new_provider.rs`

2. **Implement Provider trait**:
```rust
pub struct NewProvider {
    // Provider-specific config
}

impl Provider for NewProvider {
    fn chat(&self, messages: Vec<Message>) -> Result<String, ProviderError> {
        // Implementation
    }
    
    fn stream(&self, messages: Vec<Message>) -> Result<impl Stream<Item = StreamChunk>, ProviderError> {
        // Implementation
    }
    
    fn get_models(&self) -> Result<Vec<String>, ProviderError> {
        // Implementation
    }
    
    fn requires_api_key(&self) -> bool {
        false // or true for cloud providers
    }
}
```

3. **Add to module**: `src/providers/mod.rs`
```rust
pub mod new_provider;
pub use new_provider::NewProvider;
```

4. **Add to dropdown**: `src/tui/widgets/dropdown.rs`
```rust
ProviderInfo::new(
    "new_provider",
    "New Provider (Local)",
    false, // requires_api_key
    "default-model",
    vec!["model1", "model2"],
    true, // is_local
)
```

## API Key Management

### Security Best Practices

1. **Never store API keys** - Always read from environment
2. **Use environment variables**:
   - `ANTHROPIC_API_KEY`
   - `OPENAI_API_KEY`
3. **Prompt user** if key is missing (dropdown widget)

### Checking API Keys

```rust
pub fn check_api_key_set(&self) -> bool {
    if let Some(provider) = self.get_current_provider() {
        if !provider.requires_api_key {
            return true; // Local providers don't need API keys
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
```

## Provider Failover (Future)

Currently, provider failover is NOT implemented. Future enhancement:

```rust
// Conceptual - NOT implemented
pub struct FailoverProvider {
    primary: Box<dyn Provider>,
    fallback: Box<dyn Provider>,
}

impl Provider for FailoverProvider {
    fn chat(&self, messages: Vec<Message>) -> Result<String, ProviderError> {
        match self.primary.chat(messages.clone()) {
            Ok(response) => Ok(response),
            Err(ProviderError::RateLimit) => self.fallback.chat(messages),
            Err(e) => Err(e),
        }
    }
}
```

## Testing

### Unit Tests

- `src/providers/local_discover.rs` - Size parsing/formatting tests

### Integration Tests (TODO)

- Provider connectivity tests
- Model listing tests
- Chat completion tests

## Known Issues

1. **No automatic failover** - If a provider fails, user must manually switch
2. **No provider health tracking** - No latency/error rate monitoring
3. **Local discovery on startup only** - No hot-reload when models are installed
