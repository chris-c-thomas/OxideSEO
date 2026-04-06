//! OS-native credential storage for LLM provider API keys.
//!
//! Uses the `keyring` crate to store keys in macOS Keychain,
//! Windows Credential Manager, or Linux Secret Service.
//! Falls back to a warning if the keyring is unavailable.

use anyhow::{Context, Result};

use super::adapters::AiProviderType;

const SERVICE_NAME: &str = "com.oxideseo.desktop";

/// Get the keyring entry name for a provider.
fn key_name(provider: AiProviderType) -> &'static str {
    match provider {
        AiProviderType::OpenAi => "openai_api_key",
        AiProviderType::Anthropic => "anthropic_api_key",
        AiProviderType::Ollama => "ollama_api_key", // not used, but consistent
    }
}

/// Store an API key in the OS keychain.
pub fn store_api_key(provider: AiProviderType, key: &str) -> Result<()> {
    let entry = keyring::Entry::new(SERVICE_NAME, key_name(provider))
        .context("Failed to create keyring entry")?;
    entry
        .set_password(key)
        .context("Failed to store API key in keyring")?;
    tracing::info!(provider = %provider, "API key stored in OS keychain");
    Ok(())
}

/// Retrieve an API key from the OS keychain.
///
/// Returns `Ok(None)` if no key is stored for this provider.
pub fn get_api_key(provider: AiProviderType) -> Result<Option<String>> {
    let entry = keyring::Entry::new(SERVICE_NAME, key_name(provider))
        .context("Failed to create keyring entry")?;
    match entry.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => {
            tracing::warn!(
                provider = %provider,
                error = %e,
                "Failed to read API key from keyring"
            );
            Err(anyhow::anyhow!("Failed to read API key from keyring: {e}"))
        }
    }
}

/// Delete an API key from the OS keychain.
pub fn delete_api_key(provider: AiProviderType) -> Result<()> {
    let entry = keyring::Entry::new(SERVICE_NAME, key_name(provider))
        .context("Failed to create keyring entry")?;
    match entry.delete_credential() {
        Ok(()) => {
            tracing::info!(provider = %provider, "API key deleted from OS keychain");
            Ok(())
        }
        Err(keyring::Error::NoEntry) => {
            // Already deleted — not an error.
            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!(
            "Failed to delete API key from keyring: {e}"
        )),
    }
}

/// Check whether an API key is stored for this provider.
pub fn has_api_key(provider: AiProviderType) -> Result<bool> {
    // Ollama never needs a key.
    if provider == AiProviderType::Ollama {
        return Ok(true);
    }
    Ok(get_api_key(provider)?.is_some())
}
