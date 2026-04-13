//! AI provider system
//!
//! Supports multiple AI providers: Anthropic Claude, OpenAI, Ollama, llama.cpp, and LM Studio

pub mod anthropic;
pub mod llama_cpp;
pub mod lm_studio;
pub mod ollama;
pub mod openai;
pub mod provider_trait;

pub use anthropic::AnthropicProvider;
pub use llama_cpp::LlamaCppProvider;
pub use lm_studio::LmStudioProvider;
pub use ollama::OllamaProvider;
pub use openai::OpenAIProvider;
pub use provider_trait::{Message, Provider, ProviderError, Role, StreamChunk};
