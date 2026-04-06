//! AI analysis IPC commands: provider configuration, API key management,
//! page analysis, batch analysis, and crawl summaries.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::ai::adapters::{AiProviderConfig, AiProviderType, create_provider};
use crate::ai::keystore;
use crate::storage::db::Database;
use crate::storage::models::{AiAnalysisRow, AiCrawlSummaryRow, AiUsageRow};
use crate::storage::queries;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

/// Filter for batch page analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchAnalysisFilter {
    pub only_with_issues: bool,
    pub only_missing_meta: bool,
    pub max_pages: u32,
}

/// Result summary from batch analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchAnalysisResult {
    pub pages_analyzed: u32,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cost_usd: f64,
    pub errors: u32,
}

// ---------------------------------------------------------------------------
// Provider configuration commands
// ---------------------------------------------------------------------------

/// Get AI provider configuration (without the API key).
#[tauri::command]
pub async fn get_ai_config(db: State<'_, Arc<Database>>) -> Result<AiProviderConfig, String> {
    db.with_conn(|conn| {
        let config = match queries::get_setting(conn, "ai_provider_config")? {
            Some(json) => serde_json::from_str(&json).unwrap_or_else(|e| {
                tracing::warn!(error = %e, "Invalid AI config JSON, using default");
                AiProviderConfig::default()
            }),
            None => AiProviderConfig::default(),
        };
        Ok(config)
    })
    .map(|mut config| {
        // Compute is_configured from keyring state.
        config.is_configured = keystore::has_api_key(config.provider_type).unwrap_or(false);
        config
    })
    .map_err(|e| format!("{e:#}"))
}

/// Save AI provider configuration (model, endpoint, budget).
#[tauri::command]
pub async fn set_ai_config(
    config: AiProviderConfig,
    db: State<'_, Arc<Database>>,
) -> Result<(), String> {
    db.with_conn(|conn| {
        let json = serde_json::to_string(&config)
            .map_err(|e| anyhow::anyhow!("Failed to serialize AI config: {e}"))?;
        queries::set_setting(conn, "ai_provider_config", &json)?;
        Ok(())
    })
    .map_err(|e| format!("{e:#}"))
}

// ---------------------------------------------------------------------------
// API key management commands
// ---------------------------------------------------------------------------

/// Store an API key in the OS keychain.
#[tauri::command]
pub async fn set_api_key(provider: AiProviderType, key: String) -> Result<(), String> {
    keystore::store_api_key(provider, &key).map_err(|e| format!("{e:#}"))
}

/// Delete an API key from the OS keychain.
#[tauri::command]
pub async fn delete_api_key(provider: AiProviderType) -> Result<(), String> {
    keystore::delete_api_key(provider).map_err(|e| format!("{e:#}"))
}

/// Check if a provider has a stored API key.
#[tauri::command]
pub async fn has_api_key(provider: AiProviderType) -> Result<bool, String> {
    keystore::has_api_key(provider).map_err(|e| format!("{e:#}"))
}

/// Test connectivity to the configured provider. Returns the model name on success.
#[tauri::command]
pub async fn test_ai_connection(db: State<'_, Arc<Database>>) -> Result<String, String> {
    let config = get_ai_config_internal(&db)?;
    let api_key = keystore::get_api_key(config.provider_type).map_err(|e| format!("{e:#}"))?;

    let provider = create_provider(&config, api_key.as_deref()).map_err(|e| format!("{e:#}"))?;

    let healthy = provider
        .health_check()
        .await
        .map_err(|e| format!("{e:#}"))?;

    if healthy {
        Ok(format!(
            "Connected to {} (model: {})",
            provider.name(),
            config.model
        ))
    } else {
        Err(format!("Failed to connect to {}", provider.name()))
    }
}

// ---------------------------------------------------------------------------
// Page analysis commands
// ---------------------------------------------------------------------------

/// Analyze a single page with the configured AI provider.
#[tauri::command]
pub async fn analyze_page(
    crawl_id: String,
    page_id: i64,
    analysis_types: Vec<String>,
    db: State<'_, Arc<Database>>,
) -> Result<Vec<AiAnalysisRow>, String> {
    let config = get_ai_config_internal(&db)?;
    let api_key = keystore::get_api_key(config.provider_type).map_err(|e| format!("{e:#}"))?;
    let provider = create_provider(&config, api_key.as_deref()).map_err(|e| format!("{e:#}"))?;

    let page = db
        .with_conn(|conn| {
            queries::select_page_by_id(conn, &crawl_id, page_id)?
                .ok_or_else(|| anyhow::anyhow!("Page not found: {page_id}"))
        })
        .map_err(|e| format!("{e:#}"))?;

    let engine = crate::ai::engine::AiAnalysisEngine::new(provider, db.inner().clone());
    engine
        .analyze_page(&crawl_id, &page, &analysis_types)
        .await
        .map_err(|e| format!("{e:#}"))
}

/// Batch analyze pages for a crawl.
#[tauri::command]
pub async fn batch_analyze_pages(
    crawl_id: String,
    filter: BatchAnalysisFilter,
    analysis_types: Vec<String>,
    app_handle: tauri::AppHandle,
    db: State<'_, Arc<Database>>,
) -> Result<BatchAnalysisResult, String> {
    let config = get_ai_config_internal(&db)?;
    let api_key = keystore::get_api_key(config.provider_type).map_err(|e| format!("{e:#}"))?;
    let provider = create_provider(&config, api_key.as_deref()).map_err(|e| format!("{e:#}"))?;

    let pages = db
        .with_conn(|conn| {
            queries::select_pages_for_ai_analysis(
                conn,
                &crawl_id,
                filter.only_with_issues,
                filter.only_missing_meta,
                filter.max_pages as i64,
            )
        })
        .map_err(|e| format!("{e:#}"))?;

    let engine = crate::ai::engine::AiAnalysisEngine::new(provider, db.inner().clone());
    engine
        .batch_analyze(
            &crawl_id,
            pages,
            &analysis_types,
            config.max_tokens_per_crawl,
            app_handle,
        )
        .await
        .map_err(|e| format!("{e:#}"))
}

// ---------------------------------------------------------------------------
// Crawl summary commands
// ---------------------------------------------------------------------------

/// Generate an AI summary for a completed crawl.
#[tauri::command]
pub async fn generate_crawl_summary(
    crawl_id: String,
    db: State<'_, Arc<Database>>,
) -> Result<AiCrawlSummaryRow, String> {
    let config = get_ai_config_internal(&db)?;
    let api_key = keystore::get_api_key(config.provider_type).map_err(|e| format!("{e:#}"))?;
    let provider = create_provider(&config, api_key.as_deref()).map_err(|e| format!("{e:#}"))?;

    let engine = crate::ai::engine::AiAnalysisEngine::new(provider, db.inner().clone());
    engine
        .generate_summary(&crawl_id)
        .await
        .map_err(|e| format!("{e:#}"))
}

/// Get cached AI analyses for a page.
#[tauri::command]
pub async fn get_page_analyses(
    crawl_id: String,
    page_id: i64,
    db: State<'_, Arc<Database>>,
) -> Result<Vec<AiAnalysisRow>, String> {
    db.with_conn(|conn| queries::select_ai_analyses_for_page(conn, &crawl_id, page_id))
        .map_err(|e| format!("{e:#}"))
}

/// Get AI usage/cost stats for a crawl.
#[tauri::command]
pub async fn get_ai_usage(
    crawl_id: String,
    db: State<'_, Arc<Database>>,
) -> Result<Vec<AiUsageRow>, String> {
    db.with_conn(|conn| queries::select_ai_usage(conn, &crawl_id))
        .map_err(|e| format!("{e:#}"))
}

/// Get cached AI crawl summary.
#[tauri::command]
pub async fn get_crawl_ai_summary(
    crawl_id: String,
    db: State<'_, Arc<Database>>,
) -> Result<Option<AiCrawlSummaryRow>, String> {
    db.with_conn(|conn| queries::select_ai_crawl_summary(conn, &crawl_id))
        .map_err(|e| format!("{e:#}"))
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn get_ai_config_internal(db: &Arc<Database>) -> Result<AiProviderConfig, String> {
    let mut config = db
        .with_conn(|conn| {
            Ok(match queries::get_setting(conn, "ai_provider_config")? {
                Some(json) => serde_json::from_str(&json).unwrap_or_else(|e| {
                    tracing::warn!(error = %e, "Invalid AI config JSON, using default");
                    AiProviderConfig::default()
                }),
                None => AiProviderConfig::default(),
            })
        })
        .map_err(|e| format!("{e:#}"))?;

    config.is_configured = keystore::has_api_key(config.provider_type).unwrap_or(false);
    Ok(config)
}
