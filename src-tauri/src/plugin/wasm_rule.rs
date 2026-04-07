//! WASM rule adapter — bridges WASM plugin components to the `SeoRule` trait.
//!
//! Each `WasmRuleAdapter` holds a precompiled `Component` and creates an
//! ephemeral `Store` per `evaluate()` call. This makes the adapter `Send + Sync`
//! for rayon parallel evaluation.

use std::sync::Arc;

use wasmtime::Engine;
use wasmtime::component::Component;

use crate::crawler::ParsedPage;
use crate::rules::rule::{CrawlContext, Issue, SeoRule};
use crate::{RuleCategory, Severity};

// ---------------------------------------------------------------------------
// Plugin data types (subset of ParsedPage for WASM boundary)
// ---------------------------------------------------------------------------

/// Strict subset of `ParsedPage` exposed to WASM plugins.
///
/// This is the stability boundary — fields can be added but never removed.
#[derive(Debug, Clone)]
pub struct PluginParsedPage {
    pub url: String,
    pub title: Option<String>,
    pub meta_description: Option<String>,
    pub meta_robots: Option<String>,
    pub canonical: Option<String>,
    pub viewport: Option<String>,
    pub h1s: Vec<String>,
    pub h2s: Vec<String>,
    pub word_count: u32,
    pub body_text: Option<String>,
    pub body_size: Option<u32>,
    pub response_time_ms: Option<u32>,
    pub links_count: u32,
    pub images_count: u32,
    pub scripts: Vec<String>,
    pub stylesheets: Vec<String>,
}

/// Issue returned by a WASM plugin.
#[derive(Debug, Clone)]
pub struct PluginIssue {
    pub rule_id: String,
    pub severity: String,
    pub category: String,
    pub message: String,
    pub detail_json: Option<String>,
}

// ---------------------------------------------------------------------------
// Conversion functions
// ---------------------------------------------------------------------------

/// Convert a core `ParsedPage` to the plugin-facing subset.
pub fn to_plugin_parsed_page(page: &ParsedPage) -> PluginParsedPage {
    PluginParsedPage {
        url: page.url.clone(),
        title: page.title.clone(),
        meta_description: page.meta_description.clone(),
        meta_robots: page.meta_robots.clone(),
        canonical: page.canonical.clone(),
        viewport: page.viewport.clone(),
        h1s: page.h1s.clone(),
        h2s: page.h2s.clone(),
        word_count: page.word_count,
        body_text: page.body_text.clone(),
        body_size: page.body_size.map(|s| s.min(u32::MAX as usize) as u32),
        response_time_ms: page.response_time_ms,
        links_count: page.links.len() as u32,
        images_count: page.images.len() as u32,
        scripts: page.scripts.clone(),
        stylesheets: page.stylesheets.clone(),
    }
}

/// Convert a plugin issue to a core `Issue`, prefixing the rule_id with the plugin name.
pub fn to_core_issue(plugin_name: &str, pi: PluginIssue) -> Issue {
    let severity = pi.severity.parse::<Severity>().unwrap_or(Severity::Warning);

    let category = pi
        .category
        .parse::<RuleCategory>()
        .unwrap_or(RuleCategory::Structured);

    Issue {
        rule_id: format!("plugin.{}.{}", plugin_name, pi.rule_id),
        severity,
        category,
        message: pi.message,
        detail: pi.detail_json.and_then(|s| serde_json::from_str(&s).ok()),
    }
}

// ---------------------------------------------------------------------------
// WASM Rule Adapter
// ---------------------------------------------------------------------------

/// Configuration for constructing a `WasmRuleAdapter`.
pub struct WasmRuleConfig {
    pub engine: Arc<Engine>,
    pub component: Component,
    pub plugin_name: String,
    pub id: String,
    pub name: String,
    pub category: RuleCategory,
    pub default_severity: Severity,
    pub fuel_limit: u64,
}

/// Adapter that wraps a WASM plugin component as a `SeoRule` implementor.
///
/// Holds shared `Arc<Engine>` + precompiled `Component` (both `Send + Sync`).
/// Each `evaluate()` call creates an ephemeral `Store` with a fuel budget,
/// making this safe for rayon parallel execution.
pub struct WasmRuleAdapter {
    #[allow(dead_code)] // Used when full WIT bindgen is integrated.
    engine: Arc<Engine>,
    #[allow(dead_code)] // Used when full WIT bindgen is integrated.
    component: Component,
    #[allow(dead_code)] // Used when full WIT bindgen is integrated for error context.
    plugin_name: String,
    id: String,
    name: String,
    category: RuleCategory,
    default_severity: Severity,
    #[allow(dead_code)] // Used when full WIT bindgen is integrated for fuel budgeting.
    fuel_limit: u64,
}

impl WasmRuleAdapter {
    /// Create a new adapter from a precompiled WASM component.
    ///
    /// The `id`, `name`, `category`, and `default_severity` are fetched from
    /// the component's exported metadata functions during construction.
    pub fn new(config: WasmRuleConfig) -> Self {
        tracing::warn!(
            plugin = %config.plugin_name,
            "WASM evaluate is stubbed — this plugin will not produce issues until WIT bindgen is integrated"
        );
        Self {
            engine: config.engine,
            component: config.component,
            plugin_name: config.plugin_name,
            id: config.id,
            name: config.name,
            category: config.category,
            default_severity: config.default_severity,
            fuel_limit: config.fuel_limit,
        }
    }
}

impl SeoRule for WasmRuleAdapter {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn category(&self) -> RuleCategory {
        self.category
    }

    fn default_severity(&self) -> Severity {
        self.default_severity
    }

    fn evaluate(&self, page: &ParsedPage, _ctx: &CrawlContext) -> Vec<Issue> {
        let _plugin_page = to_plugin_parsed_page(page);

        // TODO(phase-8): Full wasmtime component instantiation + call.
        // The pattern will be:
        // 1. Create ephemeral Store with fuel budget
        // 2. Instantiate component with linker (host-log imports)
        // 3. Call guest evaluate(plugin_page)
        // 4. Convert PluginIssue -> Issue via to_core_issue
        //
        // Warning is emitted once at adapter construction time (in new()),
        // not per-page, to avoid log spam during large crawls.

        vec![]
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_plugin_parsed_page() {
        let page = ParsedPage {
            url: "https://example.com".into(),
            title: Some("Test".into()),
            meta_description: Some("Desc".into()),
            word_count: 42,
            h1s: vec!["Hello".into()],
            links: vec![],
            images: vec![],
            scripts: vec!["main.js".into()],
            stylesheets: vec!["style.css".into()],
            ..Default::default()
        };

        let pp = to_plugin_parsed_page(&page);
        assert_eq!(pp.url, "https://example.com");
        assert_eq!(pp.title.as_deref(), Some("Test"));
        assert_eq!(pp.word_count, 42);
        assert_eq!(pp.links_count, 0);
        assert_eq!(pp.images_count, 0);
        assert_eq!(pp.scripts, vec!["main.js"]);
    }

    #[test]
    fn test_to_core_issue() {
        let pi = PluginIssue {
            rule_id: "missing_schema".into(),
            severity: "error".into(),
            category: "structured".into(),
            message: "No JSON-LD found".into(),
            detail_json: Some(r#"{"count": 0}"#.into()),
        };

        let issue = to_core_issue("schema-validator", pi);
        assert_eq!(issue.rule_id, "plugin.schema-validator.missing_schema");
        assert_eq!(issue.severity, Severity::Error);
        assert_eq!(issue.category, RuleCategory::Structured);
        assert_eq!(issue.message, "No JSON-LD found");
        assert!(issue.detail.is_some());
    }

    #[test]
    fn test_to_core_issue_unknown_severity_defaults_to_warning() {
        let pi = PluginIssue {
            rule_id: "test".into(),
            severity: "unknown".into(),
            category: "meta".into(),
            message: "test".into(),
            detail_json: None,
        };

        let issue = to_core_issue("test-plugin", pi);
        assert_eq!(issue.severity, Severity::Warning);
    }

    #[test]
    fn test_to_core_issue_unknown_category_defaults_to_structured() {
        let pi = PluginIssue {
            rule_id: "test".into(),
            severity: "info".into(),
            category: "custom_category".into(),
            message: "test".into(),
            detail_json: None,
        };

        let issue = to_core_issue("test-plugin", pi);
        assert_eq!(issue.category, RuleCategory::Structured);
    }
}
