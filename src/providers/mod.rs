//! AI provider system
//!
//! Supports multiple AI providers: Anthropic Claude, OpenAI, Ollama, and llama.cpp

pub mod provider_trait;
pub mod anthropic;
pub mod openai;
pub mod ollama;
pub mod llama_cpp;

pub use provider_trait::{Provider, ProviderError, Message, Role, StreamChunk};
pub use anthropic::AnthropicProvider;
pub use openai::OpenAIProvider;
pub use ollama::OllamaProvider;
pub use llama_cpp::LlamaCppProvider;