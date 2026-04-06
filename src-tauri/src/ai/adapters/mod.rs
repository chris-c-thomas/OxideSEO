//! LLM provider adapters and shared configuration types.
//!
//! Each adapter implements the `LlmProvider` trait for a specific API.
//! The `create_provider` factory builds the appropriate adapter from config.

pub mod anthropic;
pub mod ollama;
pub mod openai;

use std::fmt;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

use super::provider::LlmProvider;

/// Which LLM provider to use.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AiProviderType {
    OpenAi,
    Anthropic,
    Ollama,
}

impl fmt::Display for AiProviderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenAi => write!(f, "openai"),
            Self::Anthropic => write!(f, "anthropic"),
            Self::Ollama => write!(f, "ollama"),
        }
    }
}

/// Configuration for the active AI provider (non-secret fields only).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderConfig {
    pub provider_type: AiProviderType,
    pub model: String,
    /// Ollama endpoint URL (only used when `provider_type` is `Ollama`).
    pub ollama_endpoint: Option<String>,
    /// Maximum tokens to spend per crawl analysis session.
    pub max_tokens_per_crawl: u32,
    /// Whether the provider has a valid API key stored.
    pub is_configured: bool,
}

impl Default for AiProviderConfig {
    fn default() -> Self {
        Self {
            provider_type: AiProviderType::OpenAi,
            model: "gpt-4o".into(),
            ollama_endpoint: None,
            max_tokens_per_crawl: 100_000,
            is_configured: false,
        }
    }
}

/// Create a boxed `LlmProvider` from config and an optional API key.
///
/// For OpenAI and Anthropic, an API key is required.
/// For Ollama, no API key is needed.
pub fn create_provider(
    config: &AiProviderConfig,
    api_key: Option<&str>,
) -> Result<Box<dyn LlmProvider>> {
    match config.provider_type {
        AiProviderType::OpenAi => {
            let key = api_key.ok_or_else(|| anyhow::anyhow!("OpenAI API key is required"))?;
            Ok(Box::new(openai::OpenAiProvider::new(
                key.to_string(),
                config.model.clone(),
            )))
        }
        AiProviderType::Anthropic => {
            let key = api_key.ok_or_else(|| anyhow::anyhow!("Anthropic API key is required"))?;
            Ok(Box::new(anthropic::AnthropicProvider::new(
                key.to_string(),
                config.model.clone(),
            )))
        }
        AiProviderType::Ollama => {
            let endpoint = config
                .ollama_endpoint
                .as_deref()
                .unwrap_or("http://localhost:11434");
            if endpoint.is_empty() {
                bail!("Ollama endpoint cannot be empty");
            }
            Ok(Box::new(ollama::OllamaProvider::new(
                endpoint.to_string(),
                config.model.clone(),
            )))
        }
    }
}
