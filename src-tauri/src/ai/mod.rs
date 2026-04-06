//! AI integration: LLM provider trait and adapters (Phase 7).
//!
//! All AI features are opt-in and degrade gracefully when no provider
//! is configured. Users supply their own API keys (BYOK model).

pub mod adapters;
pub mod engine;
pub mod keystore;
pub mod prompts;
pub mod provider;
