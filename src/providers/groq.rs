//! Groq (high-performance cloud) provider

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use super::provider_trait::{Message, Provider, ProviderError, Role, StreamChunk};

/// Groq API client
pub struct GroqProvider {
    base_url: String,
    model: String,
    api_key: String,
    client: reqwest::Client,
}

/// OpenAI-compatible request format used by Groq
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
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

impl GroqProvider {
    /// Create a new Groq provider
    pub fn new() -> Self {
        let api_key = std::env::var("GROQ_API_KEY").unwrap_or_default();
        let base_url = std::env::var("GROQ_BASE_URL").unwrap_or_else(|_| "https://api.groq.com/openai/v1".to_string());
        let model = std::env::var("GROQ_MODEL").unwrap_or_else(|_| "llama-3.3-70b-versatile".to_string());
        Self {
            base_url,
            model,
            api_key,
            client: reqwest::Client::new(),
        }
    }

    /// Create with specific model
    pub fn with_model(model: String) -> Self {
        let mut provider = Self::new();
        provider.model = model;
        provider
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
}

impl Default for GroqProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for GroqProvider {
    fn name(&self) -> &str {
        "Groq"
    }

    fn models(&self) -> Vec<String> {
        vec![
            "llama-3.3-70b-versatile".to_string(),
            "llama3-70b-8192".to_string(),
            "mixtral-8x7b-32768".to_string(),
            "gemma2-9b-it".to_string(),
        ]
    }

    fn current_model(&self) -> &str {
        &self.model
    }

    fn set_model(&mut self, model: String) {
        self.model = model;
    }

    fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn send(&self, messages: Vec<Message>) -> Result<String, ProviderError> {
        self.send_with_system(messages, None).await
    }

    async fn send_with_system(
        &self,
        messages: Vec<Message>,
        system: Option<&str>,
    ) -> Result<String, ProviderError> {
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
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
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

        Ok(result.choices.first().map(|c| c.message.content.clone()).unwrap_or_default())
    }

    async fn send_stream(
        &self,
        messages: Vec<Message>,
    ) -> Pin<Box<dyn Stream<Item = Result<StreamChunk, ProviderError>> + Send>> {
        let client = self.client.clone();
        let url = format!("{}/chat/completions", self.base_url);
        let api_key = self.api_key.clone();
        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: self.convert_messages(messages),
            stream: true,
        };

        let stream = async_stream::try_stream! {
            let response = client
                .post(url)
                .header("Authorization", format!("Bearer {}", api_key))
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
                        if data == "[DONE]" { continue; }
                        
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
        (0.59, 0.79)
    }
}