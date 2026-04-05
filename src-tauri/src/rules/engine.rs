//! Rule registry: discovers, instantiates, configures, and executes rules.

use std::collections::HashMap;

use crate::Severity;
use crate::crawler::ParsedPage;
use crate::rules::rule::{CrawlContext, Issue, SeoRule};

/// The rule registry manages all enabled SEO rules and executes them
/// against parsed pages.
pub struct RuleRegistry {
    /// All registered rules, keyed by rule ID.
    rules: Vec<Box<dyn SeoRule>>,
    /// Per-rule enabled/disabled state. Missing = enabled by default.
    enabled: HashMap<String, bool>,
    /// Per-rule severity overrides.
    severity_overrides: HashMap<String, Severity>,
}

impl RuleRegistry {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            enabled: HashMap::new(),
            severity_overrides: HashMap::new(),
        }
    }

    /// Register all built-in rules.
    pub fn register_builtins(&mut self) {
        use crate::rules::builtin;

        // Meta rules
        self.register(Box::new(builtin::meta::TitleMissing));
        self.register(Box::new(builtin::meta::TitleLength::default()));
        self.register(Box::new(builtin::meta::DescMissing));
        self.register(Box::new(builtin::meta::DescLength::default()));
        self.register(Box::new(builtin::meta::CanonicalMissing));
        self.register(Box::new(builtin::meta::CanonicalMismatch));
        self.register(Box::new(builtin::meta::ViewportMissing));

        // Content rules
        self.register(Box::new(builtin::content::H1Missing));
        self.register(Box::new(builtin::content::H1Multiple));
        self.register(Box::new(builtin::content::HeadingHierarchy));
        self.register(Box::new(builtin::content::ThinContent::default()));

        // Link rules
        self.register(Box::new(builtin::links::BrokenInternal));
        self.register(Box::new(builtin::links::RedirectChain));
        self.register(Box::new(builtin::links::NofollowInternal));

        // Image rules
        self.register(Box::new(builtin::images::AltMissing));
        self.register(Box::new(builtin::images::AltEmpty));

        // Performance & Security rules
        self.register(Box::new(builtin::performance::LargePage::default()));
        self.register(Box::new(builtin::performance::SlowResponse::default()));
        self.register(Box::new(builtin::security::MixedContent));
        self.register(Box::new(builtin::security::HttpPage));

        tracing::info!(count = self.rules.len(), "Registered built-in rules");
    }

    /// Register a single rule.
    pub fn register(&mut self, rule: Box<dyn SeoRule>) {
        self.rules.push(rule);
    }

    /// Set enabled/disabled state for a rule.
    pub fn set_enabled(&mut self, rule_id: &str, enabled: bool) {
        self.enabled.insert(rule_id.to_string(), enabled);
    }

    /// Set severity override for a rule.
    pub fn set_severity(&mut self, rule_id: &str, severity: Severity) {
        self.severity_overrides
            .insert(rule_id.to_string(), severity);
    }

    /// Apply configuration overrides from a crawl profile.
    pub fn apply_config(&mut self, config: &serde_json::Value) {
        if let Some(rules) = config.get("rules").and_then(|r| r.as_object()) {
            for (rule_id, rule_config) in rules {
                if let Some(enabled) = rule_config.get("enabled").and_then(|e| e.as_bool()) {
                    self.set_enabled(rule_id, enabled);
                }
                if let Some(severity_str) = rule_config.get("severity").and_then(|s| s.as_str()) {
                    if let Ok(severity) =
                        serde_json::from_value(serde_json::Value::String(severity_str.to_string()))
                    {
                        self.set_severity(rule_id, severity);
                    }
                }
                if let Some(params) = rule_config.get("params") {
                    for rule in &mut self.rules {
                        if rule.id() == rule_id {
                            if let Err(e) = rule.configure(params) {
                                tracing::warn!(
                                    rule_id = %rule_id,
                                    error = %e,
                                    "Failed to configure rule"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    /// Evaluate all enabled rules against a parsed page.
    ///
    /// Returns a flat list of issues. Designed to run on the rayon thread pool.
    pub fn evaluate_page(&self, page: &ParsedPage, ctx: &CrawlContext) -> Vec<Issue> {
        let mut issues = Vec::new();

        for rule in &self.rules {
            let rule_id = rule.id();

            // Skip disabled rules.
            if let Some(false) = self.enabled.get(rule_id) {
                continue;
            }

            let mut rule_issues = rule.evaluate(page, ctx);

            // Apply severity overrides.
            if let Some(override_severity) = self.severity_overrides.get(rule_id) {
                for issue in &mut rule_issues {
                    issue.severity = *override_severity;
                }
            }

            issues.extend(rule_issues);
        }

        issues
    }

    /// Get the number of registered rules.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// List all rule IDs and names (for settings UI).
    pub fn list_rules(&self) -> Vec<(String, String, String, String)> {
        self.rules
            .iter()
            .map(|r| {
                (
                    r.id().to_string(),
                    r.name().to_string(),
                    format!("{:?}", r.category()),
                    format!("{:?}", r.default_severity()),
                )
            })
            .collect()
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}
