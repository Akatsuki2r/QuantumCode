# Quantum Code Architecture

## System Overview

Quantum Code is a local-first, AI-powered coding assistant CLI built in Rust. It provides intelligent code assistance through multiple AI backends while prioritizing speed, privacy, and developer experience.

### Core Principles

1. **Local-First**: Works offline with Ollama or llama.cpp - no cloud required
2. **Performance**: Built in Rust for fast startup and low memory usage
3. **Multi-Provider**: Switch seamlessly between Claude, OpenAI, Ollama, and llama.cpp
4. **Mode-Aware**: Plan mode, build mode, chat mode - each optimized for different workflows
5. **Privacy-Focused**: Your code stays on your machine

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        CLI Interface                             │
│                    (clap + Interactive TUI)                      │
└─────────────────────────┬───────────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────────┐
│                      Application Layer                           │
│  ┌─────────────┐ ┌──────────────┐ ┌─────────────┐ ┌──────────┐ │
│  │   Commands  │ │   Providers  │ │   Router    │ │   TUI    │ │
│  │   Module    │ │   System     │ │   Engine    │ │  Engine  │ │
│  └─────────────┘ └──────────────┘ └─────────────┘ └──────────┘ │
└─────────────────────────┬───────────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────────┐
│                      Core Services                               │
│  ┌─────────────┐ ┌──────────────┐ ┌─────────────┐ ┌──────────┐ │
│  │   Config    │ │   Session    │ │    RAG      │  Tools   │ │
│  │   Manager   │ │   Manager    │ │   Engine    │  System  │ │
│  └─────────────┘ └──────────────┘ └─────────────┘ └──────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Component Breakdown

### 1. CLI Interface (`src/cli.rs`, `src/main.rs`)

**Purpose**: Entry point and command-line argument parsing

**Key Components**:
- `clap` for argument parsing
- Subcommands: `chat`, `edit`, `commit`, `review`, `test`, `scaffold`, `session`, `config`, `model`, `theme`, `status`, `help`
- Shell completions generation

**File**: `src/main.rs` (~200 lines)
**File**: `src/cli.rs` (~300 lines)

### 2. Commands Module (`src/commands/`)

**Purpose**: Implementation of all CLI commands

**Submodules**:
| File | Purpose |
|------|---------|
| `chat.rs` | Interactive chat with AI |
| `edit.rs` | AI-assisted file editing |
| `commit.rs` | AI-generated git commits |
| `review.rs` | Code review generation |
| `test.rs` | Test running with AI analysis |
| `scaffold.rs` | Project scaffolding |
| `session.rs` | Session save/load/list |
| `config.rs` | Configuration management |
| `model.rs` | Model/provider selection |
| `theme.rs` | Theme switching |
| `status.rs` | System status display |
| `help.rs` | Help documentation |

### 3. Provider System (`src/providers/`)

**Purpose**: Abstraction layer for multiple AI providers

**Architecture**:
```
┌─────────────────────────────────────────┐
│           Provider Trait                 │
│  - chat()                               │
│  - stream()                             │
│  - get_models()                         │
└─────────────────┬───────────────────────┘
                  │
    ┌─────────────┼─────────────┬─────────────┬──────────────┐
    │             │             │             │              │
┌───▼───┐   ┌────▼────┐   ┌───▼────┐  ┌────▼─────┐  ┌────▼──────┐
│Anthropic│ │ OpenAI  │   │ Ollama  │  │LM Studio │  │llama.cpp  │
│Provider │ │Provider │   │Provider │  │Provider  │  │Provider   │
└─────────┘ └─────────┘   └─────────┘  └──────────┘  └───────────┘
```

**Files**:
- `src/providers/provider_trait.rs` - Base trait definition
- `src/providers/anthropic.rs` - Anthropic Claude API
- `src/providers/openai.rs` - OpenAI GPT API
- `src/providers/ollama.rs` - Ollama local models
- `src/providers/lm_studio.rs` - LM Studio integration
- `src/providers/llama_cpp.rs` - llama.cpp integration
- `src/providers/local_discover.rs` - Auto-discovery of local models
- `src/providers/mod.rs` - Module exports

### 4. Router Engine (`src/router/`)

**Purpose**: 7-layer policy engine for routing prompts to appropriate execution contexts

**7 Layers**:
1. **Intent Classification** - regex-based, < 1ms, 16 intent types
2. **Complexity Estimation** - keyword-weighted scoring, 5 levels
3. **Mode Selection** - 5 execution modes (chat/plan/build/review/debug)
4. **Model Tier Selection** - 4 capability tiers (local/fast/standard/capable)
5. **Tool Policy** - per-intent allowed/disallowed tools
6. **Context Budget** - token budget allocation
7. **Memory Policy** - relevance-based memory loading

**Files**:
- `src/router/types.rs` - Type definitions (455 lines)
- `src/router/mod.rs` - Main routing logic (212 lines)
- `src/router/analyzer.rs` - Intent classification + complexity scoring (393 lines)
- `src/router/mode.rs` - Mode selection and state machine (149 lines)
- `src/router/model.rs` - Model tier selection (139 lines)
- `src/router/tools.rs` - Tool policy determination (85 lines)
- `src/router/context.rs` - Context budget allocation (106 lines)
- `src/router/memory.rs` - Memory policy selection (85 lines)

### 5. TUI Engine (`src/tui/`)

**Purpose**: Terminal user interface for interactive mode

**Components**:
- `src/tui/app.rs` - Application state and main loop
- `src/tui/render.rs` - Rendering logic
- `src/tui/event.rs` - Event handling
- `src/tui/widgets/` - Reusable UI widgets
  - `dropdown.rs` - Provider/model selector with API key prompts
  - `tabs.rs` - Tab navigation
  - `mod.rs` - Widget exports

### 6. Configuration (`src/config/`)

**Purpose**: Configuration management

**Files**:
- `src/config/mod.rs` - Configuration loading/saving
- `src/config/settings.rs` - Settings structure
- `src/config/themes.rs` - Theme definitions

**Config Location**: `~/.config/quantumn-code/config.toml`

### 7. RAG Engine (`src/rag/`)

**Purpose**: Retrieval-Augmented Generation for context-aware responses

**Components**:
- `RagConfig` - RAG configuration
- `ContextChunk` - Retrieved context chunks
- `KeywordRetriever` - Keyword-based retrieval (placeholder for embeddings)
- `RagIndex` - In-memory document index
- `compact_prompts` - Prompt compaction utilities

**Status**: Partially implemented - keyword retrieval works, embedding-based search not yet implemented

### 8. Tools System (`src/tools/`)

**Purpose**: Tool definitions for AI agent actions

**Tools**:
- `Read` - Read file contents
- `Write` - Write/create files
- `Bash` - Execute shell commands
- `Grep` - Search file contents
- `Glob` - Find files by pattern

**Files**:
- `src/tools/mod.rs` - Tool trait and registry
- `src/tools/read_file.rs` - Read tool implementation
- `src/tools/write_file.rs` - Write tool implementation
- `src/tools/bash.rs` - Bash tool implementation
- `src/tools/grep.rs` - Grep tool implementation
- `src/tools/glob.rs` - Glob tool implementation

### 9. Agent System (`src/agent/`)

**Purpose**: Agent execution and tool orchestration

**Files**:
- `src/agent/mod.rs` - Agent module exports
- `src/agent/executor.rs` - Tool execution engine
- `src/agent/parser.rs` - Tool call parsing
- `src/agent/tools.rs` - Tool definitions

### 10. Supervisor (`src/supervisor/`)

**Purpose**: Model supervision and fallback handling

**Files**:
- `src/supervisor/mod.rs` - Supervisor exports
- `src/supervisor/model_supervisor.rs` - Model health monitoring

## Data Flow

### Typical Request Flow

```
1. User Input (TUI or CLI)
         │
         ▼
2. Router Analysis
   - Intent Classification
   - Complexity Scoring
   - Mode Selection
   - Model Tier Selection
   - Tool Policy
   - Context Budget
   - Memory Policy
         │
         ▼
3. Provider Selection
   - Check API key availability
   - Select model based on tier
         │
         ▼
4. Context Building
   - Load memory (if applicable)
   - RAG retrieval (if enabled)
   - Apply prompt compaction
         │
         ▼
5. AI Request
   - Format request for provider
   - Send to API / local model
   - Stream response
         │
         ▼
6. Tool Execution (if needed)
   - Parse tool calls
   - Execute tools (with policy checks)
   - Feed results back to AI
         │
         ▼
7. Response Display
   - Render in TUI or CLI output
   - Save to session history
```

## File Structure

```
QuantumCode/
├── Cargo.toml              # Rust dependencies and package metadata
├── src/
│   ├── main.rs             # Entry point (~200 lines)
│   ├── cli.rs              # CLI argument parsing (~300 lines)
│   ├── app.rs              # Application state
│   ├── commands/           # All CLI commands (~12 files)
│   │   ├── chat.rs
│   │   ├── edit.rs
│   │   ├── commit.rs
│   │   ├── review.rs
│   │   ├── test.rs
│   │   ├── scaffold.rs
│   │   ├── session.rs
│   │   ├── config.rs
│   │   ├── model.rs
│   │   ├── theme.rs
│   │   ├── status.rs
│   │   ├── help.rs
│   │   └── mod.rs
│   ├── providers/          # AI provider implementations (~8 files)
│   │   ├── provider_trait.rs
│   │   ├── anthropic.rs
│   │   ├── openai.rs
│   │   ├── ollama.rs
│   │   ├── lm_studio.rs
│   │   ├── llama_cpp.rs
│   │   ├── local_discover.rs
│   │   └── mod.rs
│   ├── router/             # 7-layer routing engine (~8 files)
│   │   ├── types.rs
│   │   ├── mod.rs
│   │   ├── analyzer.rs
│   │   ├── mode.rs
│   │   ├── model.rs
│   │   ├── tools.rs
│   │   ├── context.rs
│   │   ├── memory.rs
│   │   └── tools.rs
│   ├── rag/                # RAG engine (~1 file)
│   │   └── mod.rs
│   ├── tools/              # Tool implementations (~6 files)
│   │   ├── mod.rs
│   │   ├── read_file.rs
│   │   ├── write_file.rs
│   │   ├── bash.rs
│   │   ├── grep.rs
│   │   └── glob.rs
│   ├── agent/              # Agent execution (~3 files)
│   │   ├── mod.rs
│   │   ├── executor.rs
│   │   └── parser.rs
│   ├── supervisor/         # Model supervision (~2 files)
│   │   ├── mod.rs
│   │   └── model_supervisor.rs
│   ├── config/             # Configuration (~3 files)
│   │   ├── mod.rs
│   │   ├── settings.rs
│   │   └── themes.rs
│   ├── tui/                # Terminal UI (~5 files + widgets)
│   │   ├── app.rs
│   │   ├── render.rs
│   │   ├── event.rs
│   │   ├── mod.rs
│   │   └── widgets/
│   │       ├── dropdown.rs
│   │       ├── tabs.rs
│   │       └── mod.rs
│   ├── utils/              # Utilities
│   │   ├── mod.rs
│   │   └── syntax.rs
│   └── prompts/            # Prompt templates
│       ├── mod.rs
│       ├── modes.rs
│       └── system.rs
├── themes/                  # Theme configuration files
│   ├── default.toml
│   ├── oxidized.toml
│   ├── tokyo_night.toml
│   ├── hacker.toml
│   └── deep_black.toml
├── docs/                    # Documentation (this folder)
├── research_*.md            # Research documents
└── README.md                # Main README
```

## Key Design Decisions

### 1. Rust for Performance

- Fast startup (< 100ms target)
- Low memory footprint
- Type safety and compile-time guarantees
- No garbage collection pauses

### 2. Provider Abstraction

- Single trait for all providers
- Easy to add new providers
- Consistent API across cloud and local models

### 3. Router as Policy Engine

- Pure functions, no side effects
- Testable without mocking
- Decisions are explainable (reasoning field)
- 7 layers allow fine-grained control

### 4. Mode System

- Different modes for different workflows
- Mode affects tool access, prompt shaping, model selection
- State machine prevents invalid transitions

### 5. Local-First Philosophy

- Works offline with local models
- Cloud providers are optional
- Auto-discovery of local models
- Privacy-focused (code stays on machine)

## Dependencies

### Core Dependencies

| Dependency | Purpose | Version |
|------------|---------|---------|
| `clap` | CLI parsing | 4.x |
| `ratatui` | TUI framework | 0.30 |
| `tokio` | Async runtime | 1.x |
| `reqwest` | HTTP client | 0.12 |
| `serde` | Serialization | 1.x |
| `regex` | Pattern matching | 1.x |
| `git2` | Git operations | 0.19 |
| `syntect` | Syntax highlighting | 5.x |

### Key Features from Dependencies

- `clap` with `derive`, `color`, `suggestions`, `env`
- `ratatui` for terminal UI
- `tokio` with `full` features for async
- `reqwest` with `json`, `rustls-tls`, `stream`
- `serde` with `derive` for custom types
- `chrono` with `serde` for timestamps

## Testing Strategy

### Unit Tests

- Router module has comprehensive unit tests
- Provider trait implementations tested
- Tool implementations tested
- Configuration loading tested

### Integration Tests

- CLI command testing via `assert_cmd`
- End-to-end routing tests
- Provider integration tests (require API keys or local models)

### Test Files

- Tests in `src/router/mod.rs` (7 tests)
- Tests in `src/router/analyzer.rs` (15+ tests)
- Tests in `src/router/mode.rs` (8 tests)
- Tests in `src/router/model.rs` (8 tests)
- Tests in `src/router/context.rs` (8 tests)
- Tests in `src/router/memory.rs` (6 tests)
- Tests in `src/rag/mod.rs` (5 tests)
- Tests in `src/providers/local_discover.rs` (2 tests)

## Performance Considerations

### Optimizations Implemented

1. **Regex Compilation**: `lazy_static!` for regex sets (compile once)
2. **Pure Functions**: Router is stateless, no allocations
3. **Local-First**: Default to free local models
4. **Prompt Compaction**: Remove filler words, truncate when needed

### Known Choke Points

1. **RAG Embedding**: Currently uses keyword matching, not true embeddings
2. **Token Estimation**: Rough estimate (4 chars/token), not accurate
3. **Context Loading**: No intelligent relevance scoring yet
4. **Provider Failover**: No automatic fallback between providers

See [CHOKE_POINTS.md](./CHOKE_POINTS.md) for detailed analysis.

## Security Considerations

### Current Security Measures

1. **Confirmation for Destructive Operations**: Write/Bash require confirmation in certain modes
2. **API Key Handling**: Keys read from environment variables, not stored
3. **Read-Only Modes**: Plan/Review modes cannot modify files

### Security TODOs

1. **Sandboxing**: No sandboxing of executed commands
2. **Rate Limiting**: No rate limiting on tool execution
3. **Input Validation**: Limited validation of AI-generated tool calls

See [CHOKE_POINTS.md](./CHOKE_POINTS.md) for security-related items.
