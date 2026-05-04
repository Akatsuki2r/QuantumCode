//! llama.cpp (local LLM) provider

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::pin::Pin;

use super::provider_trait::{Message, Provider, ProviderError, Role, StreamChunk};
use crate::config::settings::LlamaCppConfig;
use crate::providers::ollama::OllamaProvider;

/// llama.cpp API client
pub struct LlamaCppProvider {
    base_url: String,
    model: String, // This will be the user-friendly name, not necessarily a path
    client: reqwest::Client,
    config: LlamaCppConfig, // Store the config to access model_paths
    resolved_model_path: Option<PathBuf>, // Cache the resolved GGUF path
}

/// OpenAI-compatible request format used by llama.cpp server
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String, // The actual GGUF file name/path might be used here, or a server-internal ID
    messages: Vec<OpenAIMessage>,
    stream: bool,
}

/// OpenAI-compatible message format
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

/// OpenAI-compatible response format
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

/// OpenAI-compatible stream response format
#[derive(Debug, Deserialize)]
struct OpenAIStreamResponse {
    choices: Vec<OpenAIStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamChoice {
    delta: OpenAIStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamDelta {
    content: Option<String>,
}

impl LlamaCppProvider {
    /// Create a new LlamaCpp provider
    pub fn new(config: LlamaCppConfig) -> Self {
        let base_url = format!("http://localhost:{}", config.default_port);
        let default_model = config
            .model_paths
            .keys()
            .next()
            .cloned()
            .unwrap_or_else(|| "llama3.2".to_string()); // Fallback if no models configured

        let mut provider = Self {
            base_url,
            model: default_model,
            client: reqwest::Client::new(),
            config,
            resolved_model_path: None,
        };
        provider.resolved_model_path = provider.resolve_model_path(&provider.model);
        provider
    }

    /// Create with specific model
    pub fn with_model(model: String, config: LlamaCppConfig) -> Self {
        let mut provider = Self::new(config);
        provider.model = model;
        provider.resolved_model_path = provider.resolve_model_path(&provider.model);
        provider
    }

    /// Resolve the model name to a GGUF file path
    fn resolve_model_path(&self, model_name: &str) -> Option<PathBuf> {
        // 1. Check explicit configuration in settings
        if let Some(path_str) = self.config.model_paths.get(model_name) {
            let path = PathBuf::from(path_str);
            if path.exists() {
                tracing::debug!(
                    "LlamaCpp: Resolved model '{}' from config: {:?}",
                    model_name,
                    path
                );
                return Some(path);
            } else {
                tracing::warn!(
                    "LlamaCpp: Configured model path for '{}' does not exist: {:?}",
                    model_name,
                    path
                );
            }
        }

        // 2. Try to resolve as an Ollama model blob
        if let Some(ollama_path) = OllamaProvider::get_model_blob_path(model_name) {
            tracing::debug!(
                "LlamaCpp: Resolved model '{}' as Ollama blob: {:?}",
                model_name,
                ollama_path
            );
            return Some(ollama_path);
        }

        // 3. Check if the model name itself is a direct path to a GGUF file
        let path_as_name = PathBuf::from(model_name);
        if path_as_name.exists() && path_as_name.extension().map_or(false, |ext| ext == "gguf") {
            tracing::debug!(
                "LlamaCpp: Model name '{}' is a direct GGUF path: {:?}",
                model_name,
                path_as_name
            );
            return Some(path_as_name);
        }

        tracing::warn!("LlamaCpp: Could not resolve model path for '{}'", model_name);
        None
    }

    /// Convert internal messages to OpenAI format
    fn convert_messages(&self, messages: Vec<Message>) -> Vec<OpenAIMessage> {
        messages
            .into_iter()
            .map(|m| OpenAIMessage {
                role: match m.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                },
                content: m.content,
            })
            .collect()
    }

    /// Check if llama.cpp server is running
    pub async fn is_running(&self) -> bool {
        self.client
            .get(format!("{}/health", self.base_url)) // Assuming a health endpoint
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}

impl Default for LlamaCppProvider {
    fn default() -> Self {
        // This default is problematic as it needs LlamaCppConfig.
        // For now, create a default config. In a real app, this should be passed.
        Self::new(LlamaCppConfig::default())
    }
}

#[async_trait]
impl Provider for LlamaCppProvider {
    fn name(&self) -> &str {
        "llama.cpp"
    }

    fn models(&self) -> Vec<String> {
        let mut models = self.config.model_paths.keys().cloned().collect::<Vec<_>>();

        // Add dynamically discovered Ollama models that can be resolved
        let (ollama_names, _, _) = OllamaProvider::detect_models_comprehensive();
        for name in ollama_names {
            if self.resolve_model_path(&name).is_some() && !models.contains(&name) {
                models.push(name);
            }
        }
        models.sort();
        models
    }

    fn current_model(&self) -> &str {
        &self.model
    }

    fn set_model(&mut self, model: String) {
        self.model = model;
        self.resolved_model_path = self.resolve_model_path(&self.model);
    }

    fn is_configured(&self) -> bool {
        // Llama.cpp needs a model path to be resolved to be considered configured
        self.resolved_model_path.is_some()
    }

    async fn send(&self, messages: Vec<Message>) -> Result<String, ProviderError> {
        self.send_with_system(messages, None).await
    }

    async fn send_with_system(
        &self,
        messages: Vec<Message>,
        system: Option<&str>,
    ) -> Result<String, ProviderError> {
        // Ensure a model path is resolved before attempting to send
        let _ = self.resolved_model_path.as_ref().ok_or_else(|| {
            ProviderError::ConfigError(format!(
                "Model path for '{}' not resolved. Cannot send to llama.cpp server.",
                self.model
            ))
        })?;

        // The llama.cpp server's OpenAI-compatible API usually expects a model ID,
        // not a file path. We use the model name as the ID.
        let model_id_for_api = self.model.clone();

        let mut all_messages = Vec::new();
        if let Some(sys) = system {
            all_messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: sys.to_string(),
            });
        }
        all_messages.extend(self.convert_messages(messages));

        let request = OpenAIRequest {
            model: model_id_for_api,
            messages: all_messages,
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError(error_text));
        }

        let result: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        Ok(result
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default())
    }

    async fn send_stream(
        &self,
        messages: Vec<Message>,
    ) -> Pin<Box<dyn Stream<Item = Result<StreamChunk, ProviderError>> + Send>> {
        let client = self.client.clone();
        let url = format!("{}/v1/chat/completions", self.base_url);

        // Ensure a model path is resolved before attempting to send
        let _ = self.resolved_model_path.as_ref().ok_or_else(|| {
            ProviderError::ConfigError(format!(
                "Model path for '{}' not resolved. Cannot stream from llama.cpp server.",
                self.model
            ))
        });

        let model_id_for_api = self.model.clone(); // See comment in send_with_system

        let request = OpenAIRequest {
            model: model_id_for_api,
            messages: self.convert_messages(messages),
            stream: true,
        };

        let stream = async_stream::try_stream! {
            let response = client
                .post(url)
                .json(&request)
                .send()
                .await
                .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

            let mut bytes_stream = response.bytes_stream();
            while let Some(chunk_result) = bytes_stream.next().await {
                let bytes = chunk_result.map_err(|e| ProviderError::ApiError(e.to_string()))?;
                let text = String::from_utf8_lossy(&bytes);

                for line in text.lines() {
                    if line.starts_with("data: ") {
                        let data = &line[6..];
                        if data == "[DONE]" {
                            continue;
                        }

                        if let Ok(chunk) = serde_json::from_str::<OpenAIStreamResponse>(data) {
                            if let Some(choice) = chunk.choices.first() {
                                if let Some(content) = &choice.delta.content {
                                    yield StreamChunk {
                                        content: content.clone(),
                                        done: choice.finish_reason.is_some(),
                                        tokens: None,
                                    };
                                }
                            }
                        }
                    }
                }
            }
        };

        Box::pin(stream)
    }

    fn count_tokens(&self, text: &str) -> usize {
        text.len() / 4
    }

    fn cost_per_million(&self) -> (f64, f64) {
        // Llama.cpp is free (local)
        (0.0, 0.0)
    }
}
