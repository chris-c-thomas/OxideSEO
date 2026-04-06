//! Anthropic Messages API adapter.
//!
//! Supports Claude models via the `/v1/messages` endpoint.
//! JSON mode is achieved by prepending a JSON instruction to the system prompt.

use std::time::Instant;

use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::ai::provider::{CompletionRequest, CompletionResponse, LlmProvider, ResponseFormat};

/// Anthropic adapter implementing `LlmProvider`.
pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap_or_default();
        Self {
            client,
            api_key,
            model,
        }
    }
}

// ---------------------------------------------------------------------------
// API request / response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct MessagesRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct MessagesResponse {
    content: Vec<ContentBlock>,
    usage: AnthropicUsage,
    model: String,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Deserialize)]
struct AnthropicError {
    error: AnthropicErrorDetail,
}

#[derive(Deserialize)]
struct AnthropicErrorDetail {
    message: String,
}

// ---------------------------------------------------------------------------
// LlmProvider implementation
// ---------------------------------------------------------------------------

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "Anthropic"
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        // For JSON mode, prepend instruction to system prompt.
        let system = if request.system_prompt.is_empty()
            && request.response_format == ResponseFormat::Text
        {
            None
        } else {
            let mut sys = request.system_prompt;
            if request.response_format == ResponseFormat::Json {
                let json_instruction = "You must respond with valid JSON only. No markdown, no explanation, just JSON.";
                if sys.is_empty() {
                    sys = json_instruction.to_string();
                } else {
                    sys = format!("{json_instruction}\n\n{sys}");
                }
            }
            Some(sys)
        };

        let messages = vec![Message {
            role: "user".into(),
            content: request.user_prompt,
        }];

        let body = MessagesRequest {
            model: self.model.clone(),
            max_tokens: request.max_tokens,
            system,
            messages,
            temperature: request.temperature,
        };

        let start = Instant::now();
        let resp = retry_with_backoff(|| {
            self.client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&body)
                .send()
        })
        .await?;

        let status = resp.status();
        let resp_text = resp
            .text()
            .await
            .context("Failed to read Anthropic response body")?;
        let latency_ms = start.elapsed().as_millis() as u64;

        if !status.is_success() {
            let err_msg = serde_json::from_str::<AnthropicError>(&resp_text)
                .map(|e| e.error.message)
                .unwrap_or_else(|_| format!("HTTP {status}: {resp_text}"));
            bail!("Anthropic API error: {err_msg}");
        }

        let msg_resp: MessagesResponse =
            serde_json::from_str(&resp_text).context("Failed to parse Anthropic response")?;

        let text = msg_resp
            .content
            .first()
            .and_then(|c| c.text.clone())
            .unwrap_or_default();

        Ok(CompletionResponse {
            text,
            input_tokens: msg_resp.usage.input_tokens,
            output_tokens: msg_resp.usage.output_tokens,
            model: msg_resp.model,
            latency_ms,
        })
    }

    async fn health_check(&self) -> Result<bool> {
        let req = CompletionRequest {
            system_prompt: String::new(),
            user_prompt: "Say OK".into(),
            max_tokens: 5,
            temperature: 0.0,
            response_format: ResponseFormat::Text,
        };
        match self.complete(req).await {
            Ok(_) => Ok(true),
            Err(e) => {
                tracing::warn!(error = %e, "Anthropic health check failed");
                Ok(false)
            }
        }
    }

    fn cost_estimate(&self) -> (f64, f64) {
        // Approximate costs per 1K tokens (USD) as of early 2026.
        match self.model.as_str() {
            m if m.contains("haiku") => (0.0008, 0.004),
            m if m.contains("sonnet") => (0.003, 0.015),
            m if m.contains("opus") => (0.015, 0.075),
            _ => (0.003, 0.015), // default to sonnet-tier pricing
        }
    }
}

// ---------------------------------------------------------------------------
// Retry with exponential backoff on 429/529
// ---------------------------------------------------------------------------

async fn retry_with_backoff<F, Fut>(mut make_request: F) -> Result<reqwest::Response>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = reqwest::Result<reqwest::Response>>,
{
    let delays = [
        std::time::Duration::from_secs(1),
        std::time::Duration::from_secs(2),
        std::time::Duration::from_secs(4),
    ];

    for (attempt, delay) in delays.iter().enumerate() {
        let resp = make_request()
            .await
            .context("HTTP request to Anthropic failed")?;

        let status = resp.status().as_u16();
        // Anthropic uses 429 for rate limits and 529 for overloaded.
        if status != 429 && status != 529 {
            return Ok(resp);
        }

        let wait = resp
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .map(std::time::Duration::from_secs)
            .unwrap_or(*delay);

        tracing::warn!(
            attempt = attempt + 1,
            status,
            wait_secs = wait.as_secs(),
            "Anthropic rate limited, retrying"
        );
        tokio::time::sleep(wait).await;
    }

    make_request()
        .await
        .context("HTTP request to Anthropic failed after retries")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_estimate_sonnet() {
        let provider = AnthropicProvider::new("test-key".into(), "claude-sonnet-4-20250514".into());
        let (input, output) = provider.cost_estimate();
        assert!(input > 0.0);
        assert!(output > input);
    }

    #[test]
    fn test_cost_estimate_haiku() {
        let provider = AnthropicProvider::new("test-key".into(), "claude-haiku-4-5".into());
        let (input, _output) = provider.cost_estimate();
        assert!(input < 0.001);
    }
}
