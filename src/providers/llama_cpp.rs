//! llama.cpp (local LLM) provider

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::pin::Pin;

use super::provider_trait::{Message, Provider, ProviderError, Role, StreamChunk};
use crate::config::settings::LlamaCppConfig;
use crate::providers::ollama::OllamaProvider;

/// llama.cpp API client.
///
/// This provider connects to a running llama.cpp OpenAI-compatible server.
/// Model paths are resolved from settings for validation/discovery, but requests
/// are sent to the server using the configured model name.
pub struct LlamaCppProvider {
    base_url: String,
    model: String,
    client: reqwest::Client,
    config: LlamaCppConfig,
    resolved_model_path: Option<PathBuf>,
}

/// OpenAI-compatible request format used by llama.cpp server.
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_prompt: Option<bool>,
}

/// OpenAI-compatible message format.
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

/// OpenAI-compatible response format.
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

/// OpenAI-compatible stream response format.
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
    /// Create a new llama.cpp provider from settings.
    pub fn new(config: LlamaCppConfig) -> Self {
        let base_url = format!("http://localhost:{}", config.default_port);
        let default_model = config
            .model_paths
            .keys()
            .next()
            .cloned()
            .unwrap_or_else(|| "llama3.2".to_string());

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

    /// Create with a specific model.
    pub fn with_model(model: String, config: LlamaCppConfig) -> Self {
        let mut provider = Self::new(config);
        provider.model = model;
        provider.resolved_model_path = provider.resolve_model_path(&provider.model);
        provider
    }

    /// Compatibility constructor for callers that want a borrowed settings config.
    pub fn from_config(config: &LlamaCppConfig) -> Self {
        Self::new(config.clone())
    }

    /// Resolve a model name to a GGUF file path when possible.
    fn resolve_model_path(&self, model_name: &str) -> Option<PathBuf> {
        if let Some(path_str) = self.config.model_paths.get(model_name) {
            let path = PathBuf::from(path_str);
            if path.exists() {
                tracing::debug!(
                    "LlamaCpp: Resolved model '{}' from config: {:?}",
                    model_name,
                    path
                );
                return Some(path);
            }

            tracing::warn!(
                "LlamaCpp: Configured model path for '{}' does not exist: {:?}",
                model_name,
                path
            );
        }

        let path_as_name = PathBuf::from(model_name);
        if path_as_name.exists() && path_as_name.extension().map_or(false, |ext| ext == "gguf") {
            tracing::debug!(
                "LlamaCpp: Model name '{}' is a direct GGUF path: {:?}",
                model_name,
                path_as_name
            );
            return Some(path_as_name);
        }

        tracing::warn!(
            "LlamaCpp: Could not resolve model path for '{}'",
            model_name
        );
        None
    }

    /// Convert internal messages to OpenAI format.
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

    /// Check if llama.cpp server is running.
    pub async fn is_running(&self) -> bool {
        self.client
            .get(format!("{}/health", self.base_url))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}

impl Default for LlamaCppProvider {
    fn default() -> Self {
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
        let _ = self.resolved_model_path.as_ref().ok_or_else(|| {
            ProviderError::ConfigError(format!(
                "Model path for '{}' not resolved. Cannot send to llama.cpp server.",
                self.model
            ))
        })?;

        let mut all_messages = Vec::new();
        if let Some(sys) = system {
            all_messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: sys.to_string(),
            });
        }
        all_messages.extend(self.convert_messages(messages));

        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: all_messages,
            stream: false,
            cache_prompt: Some(true),
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
        if let Err(e) = self.resolved_model_path.as_ref().ok_or_else(|| {
            ProviderError::ConfigError(format!(
                "Model path for '{}' not resolved. Cannot stream from llama.cpp server.",
                self.model
            ))
        }) {
            return Box::pin(futures::stream::once(async move { Err(e) }));
        }

        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: self.convert_messages(messages),
            stream: true,
            cache_prompt: Some(true),
        };

        let client = self.client.clone();
        let url = format!("{}/v1/chat/completions", self.base_url);

        let stream = async_stream::try_stream! {
            let response = client
                .post(url)
                .json(&request)
                .send()
                .await
                .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

            if response.status().is_success() {
                let mut bytes_stream = response.bytes_stream();
                while let Some(chunk_result) = bytes_stream.next().await {
                    let bytes = chunk_result.map_err(|e| ProviderError::ApiError(e.to_string()))?;
                    let text = String::from_utf8_lossy(&bytes);

                    for line in text.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            if data == "[DONE]" {
                                yield StreamChunk {
                                    content: String::new(),
                                    done: true,
                                    tokens: None,
                                };
                                return;
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
            } else {
                let error_text = response.text().await.unwrap_or_default();
                Err(ProviderError::ApiError(error_text))?;
            }
        };

        Box::pin(stream)
    }

    fn count_tokens(&self, text: &str) -> usize {
        text.len() / 4
    }

    fn cost_per_million(&self) -> (f64, f64) {
        (0.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_provider_has_fallback_model() {
        let provider = LlamaCppProvider::default();
        assert_eq!(provider.current_model(), "llama3.2");
    }

    #[test]
    fn test_config_models_are_listed() {
        let mut config = LlamaCppConfig::default();
        config
            .model_paths
            .insert("test-model".to_string(), "/tmp/test-model.gguf".to_string());

        let provider = LlamaCppProvider::new(config);
        assert!(provider.models().contains(&"test-model".to_string()));
    }
}
