//! OpenAI Chat Completions API adapter.
//!
//! Supports GPT-4o and similar models via the `/v1/chat/completions` endpoint.
//! JSON mode is enabled via the `response_format` parameter.

use std::time::Instant;

use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::ai::provider::{CompletionRequest, CompletionResponse, LlmProvider, ResponseFormat};

/// OpenAI adapter implementing `LlmProvider`.
pub struct OpenAiProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl OpenAiProvider {
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
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ChatResponseFormat>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatResponseFormat {
    r#type: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
    usage: ChatUsage,
    model: String,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Deserialize)]
struct ChatChoiceMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct ChatUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

#[derive(Deserialize)]
struct OpenAiError {
    error: OpenAiErrorDetail,
}

#[derive(Deserialize)]
struct OpenAiErrorDetail {
    message: String,
}

// ---------------------------------------------------------------------------
// LlmProvider implementation
// ---------------------------------------------------------------------------

#[async_trait]
impl LlmProvider for OpenAiProvider {
    fn name(&self) -> &str {
        "OpenAI"
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let mut messages = Vec::with_capacity(2);
        if !request.system_prompt.is_empty() {
            messages.push(ChatMessage {
                role: "system".into(),
                content: request.system_prompt,
            });
        }
        messages.push(ChatMessage {
            role: "user".into(),
            content: request.user_prompt,
        });

        let response_format = match request.response_format {
            ResponseFormat::Json => Some(ChatResponseFormat {
                r#type: "json_object".into(),
            }),
            ResponseFormat::Text => None,
        };

        let body = ChatRequest {
            model: self.model.clone(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            response_format,
        };

        let start = Instant::now();
        let resp = super::retry_with_backoff(
            || {
                self.client
                    .post("https://api.openai.com/v1/chat/completions")
                    .header("Authorization", format!("Bearer {}", self.api_key))
                    .json(&body)
                    .send()
            },
            &[429],
            "OpenAI",
        )
        .await?;

        let status = resp.status();
        let resp_text = resp
            .text()
            .await
            .context("Failed to read OpenAI response body")?;
        let latency_ms = start.elapsed().as_millis() as u64;

        if !status.is_success() {
            let err_msg = serde_json::from_str::<OpenAiError>(&resp_text)
                .map(|e| e.error.message)
                .unwrap_or_else(|_| format!("HTTP {status}: {resp_text}"));
            bail!("OpenAI API error: {err_msg}");
        }

        let chat_resp: ChatResponse =
            serde_json::from_str(&resp_text).context("Failed to parse OpenAI response")?;

        let text = chat_resp
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .filter(|t| !t.is_empty())
            .ok_or_else(|| anyhow::anyhow!("OpenAI returned empty response content"))?;

        Ok(CompletionResponse {
            text,
            input_tokens: chat_resp.usage.prompt_tokens,
            output_tokens: chat_resp.usage.completion_tokens,
            model: chat_resp.model,
            latency_ms,
        })
    }

    async fn health_check(&self) -> Result<bool> {
        // Use GET /v1/models instead of a completion — no tokens consumed.
        let resp = self
            .client
            .get("https://api.openai.com/v1/models")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .context("OpenAI health check failed")?;
        if resp.status().is_success() {
            Ok(true)
        } else {
            let status = resp.status();
            bail!("OpenAI API returned HTTP {status}");
        }
    }

    fn cost_estimate(&self) -> (f64, f64) {
        // Approximate costs per 1K tokens (USD) as of early 2026.
        // These are rough estimates — actual pricing may differ.
        match self.model.as_str() {
            m if m.starts_with("gpt-4o-mini") => (0.00015, 0.0006),
            m if m.starts_with("gpt-4o") => (0.0025, 0.01),
            m if m.starts_with("gpt-4") => (0.03, 0.06),
            _ => (0.005, 0.015), // conservative default
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_estimate_gpt4o() {
        let provider = OpenAiProvider::new("test-key".into(), "gpt-4o".into());
        let (input, output) = provider.cost_estimate();
        assert!(input > 0.0);
        assert!(output > 0.0);
        assert!(output > input);
    }

    #[test]
    fn test_cost_estimate_gpt4o_mini() {
        let provider = OpenAiProvider::new("test-key".into(), "gpt-4o-mini".into());
        let (input, output) = provider.cost_estimate();
        assert!(input < 0.001);
        assert!(output < 0.001);
    }
}
