//! LLM provider adapters.
//!
//! Each adapter implements the `LlmProvider` trait for a specific API.
//! These are implemented in Phase 7.

// pub mod anthropic;
// pub mod ollama;
// pub mod openai;

// TODO(phase-7): Implement provider adapters:
//
// OpenAI adapter:
//   - Chat Completions endpoint
//   - Supports GPT-4o and similar models
//   - JSON mode via response_format parameter
//
// Anthropic adapter:
//   - Messages endpoint
//   - Supports Claude models
//   - JSON mode via system prompt instruction
//
// Ollama adapter:
//   - Local API at localhost:11434
//   - Supports any pulled model
//   - No cost tracking needed (local inference)
//
// All adapters:
//   - Handle rate limit errors (429) with exponential backoff
//   - Handle context length errors gracefully (truncate input, retry)
//   - Construct HTTP request with reqwest, parse response
