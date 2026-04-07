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

// Re-export from engine to maintain public API.
pub use crate::ai::engine::BatchAnalysisResult;

/// Pre-flight cost estimate for batch analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchCostEstimate {
    pub eligible_pages: u32,
    pub estimated_input_tokens: u64,
    pub estimated_output_tokens: u64,
    pub estimated_cost_usd: f64,
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
///
/// Times out after 15 seconds to avoid hanging on unreachable endpoints.
#[tauri::command]
pub async fn test_ai_connection(db: State<'_, Arc<Database>>) -> Result<String, String> {
    let config = get_ai_config_internal(&db)?;
    let api_key = keystore::get_api_key(config.provider_type).map_err(|e| format!("{e:#}"))?;

    let provider = create_provider(&config, api_key.as_deref()).map_err(|e| format!("{e:#}"))?;

    match tokio::time::timeout(std::time::Duration::from_secs(15), provider.health_check()).await {
        Ok(Ok(_)) => Ok(format!(
            "Connected to {} (model: {})",
            provider.name(),
            config.model
        )),
        Ok(Err(e)) => Err(format!("Connection failed: {e:#}")),
        Err(_) => Err("Connection timed out after 15 seconds".to_string()),
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
// Cost estimation
// ---------------------------------------------------------------------------

/// Estimate the cost of a batch analysis before running it.
#[tauri::command]
pub async fn estimate_batch_cost(
    crawl_id: String,
    filter: BatchAnalysisFilter,
    analysis_types: Vec<String>,
    db: State<'_, Arc<Database>>,
) -> Result<BatchCostEstimate, String> {
    let config = get_ai_config_internal(&db)?;
    let api_key = keystore::get_api_key(config.provider_type).map_err(|e| format!("{e:#}"))?;
    let provider = create_provider(&config, api_key.as_deref()).map_err(|e| format!("{e:#}"))?;

    let page_count = db
        .with_conn(|conn| {
            queries::count_pages_for_ai_analysis(
                conn,
                &crawl_id,
                filter.only_with_issues,
                filter.only_missing_meta,
                filter.max_pages as i64,
            )
        })
        .map_err(|e| format!("{e:#}"))?;

    let num_types = analysis_types.len() as u64;
    // Rough estimate: ~2000 input tokens + ~500 output tokens per analysis type per page.
    let est_input = page_count as u64 * num_types * 2000;
    let est_output = page_count as u64 * num_types * 500;
    let (input_cost, output_cost) = provider.cost_estimate();
    let est_cost = (est_input as f64 * input_cost + est_output as f64 * output_cost) / 1000.0;

    Ok(BatchCostEstimate {
        eligible_pages: page_count as u32,
        estimated_input_tokens: est_input,
        estimated_output_tokens: est_output,
        estimated_cost_usd: est_cost,
    })
}

// ---------------------------------------------------------------------------
// Crawl summary commands
// ---------------------------------------------------------------------------

/// Generate an AI summary for a completed crawl.
///
/// When `force` is true, regenerates even if a cached summary exists.
#[tauri::command]
pub async fn generate_crawl_summary(
    crawl_id: String,
    force: bool,
    db: State<'_, Arc<Database>>,
) -> Result<AiCrawlSummaryRow, String> {
    let config = get_ai_config_internal(&db)?;
    let api_key = keystore::get_api_key(config.provider_type).map_err(|e| format!("{e:#}"))?;
    let provider = create_provider(&config, api_key.as_deref()).map_err(|e| format!("{e:#}"))?;

    let engine = crate::ai::engine::AiAnalysisEngine::new(provider, db.inner().clone());
    engine
        .generate_summary(&crawl_id, force)
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
// Ollama model discovery
// ---------------------------------------------------------------------------

/// List models installed on an Ollama instance.
#[tauri::command]
pub async fn list_ollama_models(endpoint: String) -> Result<Vec<String>, String> {
    crate::ai::adapters::ollama::list_ollama_models(&endpoint)
        .await
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
