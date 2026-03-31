//! Core trait and types for the SEO rule engine.

use serde::{Deserialize, Serialize};

use crate::crawler::ParsedPage;
use crate::{RuleCategory, Severity};

/// An SEO issue detected by a rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    /// The rule that generated this issue (e.g., "meta.title_missing").
    pub rule_id: String,
    /// Severity level.
    pub severity: Severity,
    /// Category for UI grouping.
    pub category: RuleCategory,
    /// Human-readable issue description.
    pub message: String,
    /// Rule-specific structured detail (optional JSON).
    pub detail: Option<serde_json::Value>,
}

/// Context available to rules during evaluation.
///
/// Provides access to crawl configuration and cross-page data lookups
/// for rules that need broader context than a single page.
pub struct CrawlContext {
    /// The crawl's root domain.
    pub root_domain: String,
    /// Whether cross-page analysis data is available (post-crawl only).
    pub cross_page_available: bool,
    // TODO(phase-3): Add methods for cross-page lookups:
    // - fn find_pages_with_title(&self, title: &str) -> Vec<i64>
    // - fn inbound_link_count(&self, page_id: i64) -> u32
}

/// Core trait for SEO audit rules.
///
/// Every rule implements this trait, enabling uniform registration,
/// execution, configuration, and testing.
///
/// # Stability Contract
/// - Stable from v1.0. New optional methods use default implementations.
/// - Implementors must be `Send + Sync` for rayon parallel evaluation.
pub trait SeoRule: Send + Sync {
    /// Unique identifier (e.g., "meta.title_missing").
    fn id(&self) -> &str;

    /// Human-readable name.
    fn name(&self) -> &str;

    /// Category for UI grouping.
    fn category(&self) -> RuleCategory;

    /// Default severity. Can be overridden by user config.
    fn default_severity(&self) -> Severity;

    /// Evaluate this rule against a parsed page.
    ///
    /// Returns zero or more issues found. Called on the rayon thread pool.
    fn evaluate(&self, page: &ParsedPage, ctx: &CrawlContext) -> Vec<Issue>;

    /// JSON Schema for rule-specific configurable parameters (optional).
    fn config_schema(&self) -> Option<serde_json::Value> {
        None
    }

    /// Apply configuration parameters to this rule instance.
    fn configure(&mut self, _params: &serde_json::Value) -> anyhow::Result<()> {
        Ok(())
    }
}
