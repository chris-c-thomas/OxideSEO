//! Ollama local inference adapter.
//!
//! Connects to a local Ollama instance at a configurable endpoint
//! (default `http://localhost:11434`). No API key required, no cost tracking.

use std::time::Instant;

use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::ai::provider::{CompletionRequest, CompletionResponse, LlmProvider, ResponseFormat};

/// Ollama adapter implementing `LlmProvider`.
pub struct OllamaProvider {
    client: reqwest::Client,
    endpoint: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(endpoint: String, model: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120)) // Local models can be slow
            .build()
            .unwrap_or_default();
        Self {
            client,
            endpoint,
            model,
        }
    }
}

// ---------------------------------------------------------------------------
// API request / response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: u32,
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    message: OllamaResponseMessage,
    model: String,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    eval_count: Option<u32>,
}

#[derive(Deserialize)]
struct OllamaResponseMessage {
    content: String,
}

#[derive(Deserialize)]
struct OllamaErrorResponse {
    error: String,
}

// ---------------------------------------------------------------------------
// LlmProvider implementation
// ---------------------------------------------------------------------------

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn name(&self) -> &str {
        "Ollama"
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let mut messages = Vec::with_capacity(2);
        if !request.system_prompt.is_empty() {
            messages.push(OllamaMessage {
                role: "system".into(),
                content: request.system_prompt,
            });
        }
        messages.push(OllamaMessage {
            role: "user".into(),
            content: request.user_prompt,
        });

        let format = match request.response_format {
            ResponseFormat::Json => Some("json".to_string()),
            ResponseFormat::Text => None,
        };

        let body = OllamaChatRequest {
            model: self.model.clone(),
            messages,
            stream: false,
            format,
            options: OllamaOptions {
                temperature: request.temperature,
                num_predict: request.max_tokens,
            },
        };

        let url = format!("{}/api/chat", self.endpoint.trim_end_matches('/'));
        let start = Instant::now();

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("HTTP request to Ollama failed")?;

        let status = resp.status();
        let resp_text = resp
            .text()
            .await
            .context("Failed to read Ollama response body")?;
        let latency_ms = start.elapsed().as_millis() as u64;

        if !status.is_success() {
            let err_msg = serde_json::from_str::<OllamaErrorResponse>(&resp_text)
                .map(|e| e.error)
                .unwrap_or_else(|_| format!("HTTP {status}: {resp_text}"));
            bail!("Ollama API error: {err_msg}");
        }

        let chat_resp: OllamaChatResponse =
            serde_json::from_str(&resp_text).context("Failed to parse Ollama response")?;

        Ok(CompletionResponse {
            text: chat_resp.message.content,
            input_tokens: chat_resp.prompt_eval_count.unwrap_or(0),
            output_tokens: chat_resp.eval_count.unwrap_or(0),
            model: chat_resp.model,
            latency_ms,
        })
    }

    async fn health_check(&self) -> Result<bool> {
        // Check if the Ollama server is reachable by listing models.
        let url = format!("{}/api/tags", self.endpoint.trim_end_matches('/'));
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to Ollama")?;
        if resp.status().is_success() {
            Ok(true)
        } else {
            bail!("Ollama returned HTTP {}", resp.status());
        }
    }

    fn cost_estimate(&self) -> (f64, f64) {
        // Local inference — no cost.
        (0.0, 0.0)
    }
}

// ---------------------------------------------------------------------------
// Ollama model discovery
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModelInfo>,
}

#[derive(Debug, Deserialize)]
struct OllamaModelInfo {
    name: String,
}

/// List models installed on an Ollama instance.
pub async fn list_ollama_models(endpoint: &str) -> Result<Vec<String>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let url = format!("{}/api/tags", endpoint.trim_end_matches('/'));
    let resp = client
        .get(&url)
        .send()
        .await
        .context("Failed to connect to Ollama")?;
    if !resp.status().is_success() {
        bail!("Ollama returned HTTP {}", resp.status());
    }
    let tags: OllamaTagsResponse = resp
        .json()
        .await
        .context("Failed to parse Ollama model list")?;
    Ok(tags.models.into_iter().map(|m| m.name).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_estimate_is_zero() {
        let provider = OllamaProvider::new("http://localhost:11434".into(), "llama3".into());
        let (input, output) = provider.cost_estimate();
        assert_eq!(input, 0.0);
        assert_eq!(output, 0.0);
    }

    #[test]
    fn test_endpoint_url_construction() {
        let provider = OllamaProvider::new("http://localhost:11434/".into(), "llama3".into());
        // Verify trailing slash is handled
        let url = format!("{}/api/chat", provider.endpoint.trim_end_matches('/'));
        assert_eq!(url, "http://localhost:11434/api/chat");
    }
}
