//! AI analysis engine: orchestrates LLM calls with caching, rate limiting,
//! budget enforcement, and usage tracking.

use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::storage::db::Database;
use crate::storage::models::{AiAnalysisRow, AiCrawlSummaryRow, PageRow};
use crate::storage::queries;

use super::prompts;
use super::provider::{CompletionResponse, LlmProvider};

/// Delay between requests to avoid bursting (milliseconds).
const REQUEST_DELAY_MS: u64 = 200;

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

/// AI analysis progress event payload.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchProgress {
    pub completed: u32,
    pub total: u32,
    pub current_url: String,
    pub tokens_used: u64,
    pub budget_remaining: u64,
}

/// Orchestrates AI analysis across pages with caching and rate limiting.
pub struct AiAnalysisEngine {
    provider: Box<dyn LlmProvider>,
    db: Arc<Database>,
}

impl AiAnalysisEngine {
    pub fn new(provider: Box<dyn LlmProvider>, db: Arc<Database>) -> Self {
        Self { provider, db }
    }

    /// Analyze a single page. Checks cache first by content hash.
    pub async fn analyze_page(
        &self,
        crawl_id: &str,
        page: &PageRow,
        analysis_types: &[String],
    ) -> Result<Vec<AiAnalysisRow>> {
        let mut results = Vec::new();

        let body_text = page.body_text.as_deref().unwrap_or_default();
        if body_text.is_empty() {
            tracing::warn!(page_id = page.id, "No body text available for AI analysis");
            return Ok(results);
        }

        // Compute content hash for cache lookup.
        let content_hash = blake3::hash(body_text.as_bytes());
        let hash_bytes = content_hash.as_bytes().to_vec();

        for analysis_type in analysis_types {
            // Check cache first.
            let cached = self.db.with_conn(|conn| {
                queries::select_ai_analysis_by_hash(conn, crawl_id, &hash_bytes, analysis_type)
            })?;

            if let Some(cached_row) = cached {
                tracing::debug!(page_id = page.id, analysis_type, "Using cached AI analysis");
                results.push(cached_row);
                continue;
            }

            // Build prompt and call LLM.
            let request = match analysis_type.as_str() {
                "content_score" => {
                    prompts::content_quality_request(body_text, &page.url, page.title.as_deref())
                }
                "meta_desc" => prompts::meta_description_request(
                    body_text,
                    page.title.as_deref(),
                    page.meta_desc.as_deref(),
                ),
                "title_tag" => {
                    prompts::title_tag_request(body_text, page.title.as_deref(), &page.url)
                }
                other => {
                    tracing::warn!(analysis_type = other, "Unknown analysis type, skipping");
                    continue;
                }
            };

            let response = self
                .provider
                .complete(request)
                .await
                .context("LLM completion failed")?;

            let (input_cost, output_cost) = self.provider.cost_estimate();
            let cost = (response.input_tokens as f64 * input_cost
                + response.output_tokens as f64 * output_cost)
                / 1000.0;

            let row = AiAnalysisRow {
                id: 0,
                crawl_id: crawl_id.to_string(),
                page_id: page.id,
                analysis_type: analysis_type.clone(),
                provider: self.provider.name().to_string(),
                model: response.model.clone(),
                result_json: response.text.clone(),
                input_tokens: response.input_tokens,
                output_tokens: response.output_tokens,
                cost_usd: cost,
                latency_ms: response.latency_ms as u32,
                created_at: String::new(), // Set by SQL
            };

            // Store result and update usage.
            self.db.with_conn(|conn| {
                queries::insert_ai_analysis_with_hash(conn, &row, &hash_bytes)?;
                queries::upsert_ai_usage(
                    conn,
                    crawl_id,
                    &row.provider,
                    &row.model,
                    row.input_tokens,
                    row.output_tokens,
                    cost,
                )?;
                Ok(())
            })?;

            results.push(row);

            // Rate limiting delay.
            tokio::time::sleep(std::time::Duration::from_millis(REQUEST_DELAY_MS)).await;
        }

        Ok(results)
    }

    /// Batch analyze pages with rate limiting and budget enforcement.
    pub async fn batch_analyze(
        &self,
        crawl_id: &str,
        pages: Vec<PageRow>,
        analysis_types: &[String],
        budget_tokens: u32,
        app_handle: tauri::AppHandle,
    ) -> Result<BatchAnalysisResult> {
        use tauri::Emitter;

        let total = pages.len() as u32;
        let mut completed = 0u32;
        let mut total_input_tokens = 0u64;
        let mut total_output_tokens = 0u64;
        let mut total_cost = 0.0f64;
        let mut errors = 0u32;

        // Budget limit at 90% to leave room for summary.
        let budget_limit = (budget_tokens as f64 * 0.9) as u64;

        for page in &pages {
            // Check budget.
            let tokens_used = total_input_tokens + total_output_tokens;
            if tokens_used >= budget_limit {
                tracing::info!(
                    tokens_used,
                    budget_limit,
                    "Token budget exhausted, stopping batch analysis"
                );
                break;
            }

            // Emit progress event.
            if let Err(e) = app_handle.emit(
                "ai://progress",
                BatchProgress {
                    completed,
                    total,
                    current_url: page.url.clone(),
                    tokens_used,
                    budget_remaining: budget_limit.saturating_sub(tokens_used),
                },
            ) {
                tracing::warn!(error = %e, "Failed to emit AI progress event");
            }

            match self.analyze_page(crawl_id, page, analysis_types).await {
                Ok(results) => {
                    for r in &results {
                        total_input_tokens += r.input_tokens as u64;
                        total_output_tokens += r.output_tokens as u64;
                        total_cost += r.cost_usd;
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        page_id = page.id,
                        error = %e,
                        "AI analysis failed for page"
                    );
                    errors += 1;
                }
            }

            completed += 1;
        }

        // Final progress event.
        if let Err(e) = app_handle.emit(
            "ai://progress",
            BatchProgress {
                completed,
                total,
                current_url: String::new(),
                tokens_used: total_input_tokens + total_output_tokens,
                budget_remaining: budget_limit
                    .saturating_sub(total_input_tokens + total_output_tokens),
            },
        ) {
            tracing::warn!(error = %e, "Failed to emit final AI progress event");
        }

        Ok(BatchAnalysisResult {
            pages_analyzed: completed,
            total_input_tokens,
            total_output_tokens,
            total_cost_usd: total_cost,
            errors,
        })
    }

    /// Generate an AI summary for a completed crawl.
    pub async fn generate_summary(&self, crawl_id: &str) -> Result<AiCrawlSummaryRow> {
        // Check for cached summary first.
        if let Some(cached) = self
            .db
            .with_conn(|conn| queries::select_ai_crawl_summary(conn, crawl_id))?
        {
            return Ok(cached);
        }

        // Gather crawl statistics.
        let stats = self.db.with_conn(|conn| {
            let crawl = queries::select_crawl_by_id(conn, crawl_id)?
                .ok_or_else(|| anyhow::anyhow!("Crawl not found: {crawl_id}"))?;
            let (errors, warnings, info) = queries::count_issues_by_severity(conn, crawl_id)?;
            let total_pages = queries::count_pages_filtered(conn, crawl_id, None, None, None)?;

            Ok(serde_json::json!({
                "crawlId": crawl_id,
                "startUrl": crawl.start_url,
                "urlsCrawled": crawl.urls_crawled,
                "urlsErrored": crawl.urls_errored,
                "totalPages": total_pages,
                "issues": {
                    "errors": errors,
                    "warnings": warnings,
                    "info": info,
                    "total": errors + warnings + info,
                }
            }))
        })?;

        let stats_json = serde_json::to_string_pretty(&stats)?;
        let request = prompts::crawl_summary_request(&stats_json);
        let response: CompletionResponse = self
            .provider
            .complete(request)
            .await
            .context("Failed to generate crawl summary")?;

        let (input_cost, output_cost) = self.provider.cost_estimate();
        let cost = (response.input_tokens as f64 * input_cost
            + response.output_tokens as f64 * output_cost)
            / 1000.0;

        let row = AiCrawlSummaryRow {
            id: 0,
            crawl_id: crawl_id.to_string(),
            provider: self.provider.name().to_string(),
            model: response.model.clone(),
            summary_json: response.text,
            input_tokens: response.input_tokens,
            output_tokens: response.output_tokens,
            cost_usd: cost,
            created_at: String::new(),
        };

        self.db.with_conn(|conn| {
            queries::insert_ai_crawl_summary(conn, &row)?;
            queries::upsert_ai_usage(
                conn,
                crawl_id,
                &row.provider,
                &row.model,
                row.input_tokens,
                row.output_tokens,
                cost,
            )?;
            Ok(())
        })?;

        // Re-fetch to get the DB-generated ID and timestamp.
        self.db
            .with_conn(|conn| queries::select_ai_crawl_summary(conn, crawl_id))?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve saved crawl summary"))
    }
}
