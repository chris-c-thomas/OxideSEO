//! LLM provider abstraction for AI-powered analysis features.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Request payload for LLM completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub system_prompt: String,
    pub user_prompt: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub response_format: ResponseFormat,
}

/// Response format hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseFormat {
    Text,
    Json,
}

/// Response payload from LLM completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub text: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub model: String,
    pub latency_ms: u64,
}

/// Provider-agnostic LLM interface.
///
/// Implemented by OpenAI, Anthropic, and Ollama adapters.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Provider display name.
    fn name(&self) -> &str;

    /// Send a completion request and receive a response.
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;

    /// Check if the provider is configured and reachable.
    async fn health_check(&self) -> Result<bool>;

    /// Estimated cost per 1K tokens: (input_cost, output_cost).
    fn cost_estimate(&self) -> (f64, f64);
}

/// Compute USD cost from token counts and per-1K-token rates.
pub fn compute_cost(input_tokens: u64, output_tokens: u64, cost_per_1k: (f64, f64)) -> f64 {
    let (input_rate, output_rate) = cost_per_1k;
    (input_tokens as f64 * input_rate + output_tokens as f64 * output_rate) / 1000.0
}
