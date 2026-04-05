//! Meta rules: title tag, meta description, canonical, viewport.

use crate::crawler::ParsedPage;
use crate::rules::rule::{CrawlContext, Issue, SeoRule};
use crate::{RuleCategory, Severity};

// ---------------------------------------------------------------------------
// meta.title_missing
// ---------------------------------------------------------------------------

pub struct TitleMissing;

impl SeoRule for TitleMissing {
    fn id(&self) -> &str {
        "meta.title_missing"
    }
    fn name(&self) -> &str {
        "Missing Title Tag"
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Meta
    }
    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn evaluate(&self, page: &ParsedPage, _ctx: &CrawlContext) -> Vec<Issue> {
        if page.title.is_none() || page.title.as_deref() == Some("") {
            vec![Issue {
                rule_id: self.id().to_string(),
                severity: self.default_severity(),
                category: self.category(),
                message: "Page is missing a <title> tag.".to_string(),
                detail: None,
            }]
        } else {
            vec![]
        }
    }
}

// ---------------------------------------------------------------------------
// meta.title_length
// ---------------------------------------------------------------------------

pub struct TitleLength {
    pub min: usize,
    pub max: usize,
}

impl Default for TitleLength {
    fn default() -> Self {
        Self { min: 30, max: 60 }
    }
}

impl SeoRule for TitleLength {
    fn id(&self) -> &str {
        "meta.title_length"
    }
    fn name(&self) -> &str {
        "Title Tag Length"
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Meta
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn evaluate(&self, page: &ParsedPage, _ctx: &CrawlContext) -> Vec<Issue> {
        if let Some(title) = &page.title {
            let len = title.chars().count();
            if len < self.min {
                vec![Issue {
                    rule_id: self.id().to_string(),
                    severity: self.default_severity(),
                    category: self.category(),
                    message: format!(
                        "Title is too short ({} chars, minimum {} recommended).",
                        len, self.min
                    ),
                    detail: Some(
                        serde_json::json!({ "length": len, "min": self.min, "max": self.max }),
                    ),
                }]
            } else if len > self.max {
                vec![Issue {
                    rule_id: self.id().to_string(),
                    severity: self.default_severity(),
                    category: self.category(),
                    message: format!(
                        "Title is too long ({} chars, maximum {} recommended).",
                        len, self.max
                    ),
                    detail: Some(
                        serde_json::json!({ "length": len, "min": self.min, "max": self.max }),
                    ),
                }]
            } else {
                vec![]
            }
        } else {
            vec![] // Handled by TitleMissing rule.
        }
    }

    fn config_schema(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "min": { "type": "integer", "default": 30, "minimum": 1 },
                "max": { "type": "integer", "default": 60, "minimum": 1 }
            }
        }))
    }

    fn configure(&mut self, params: &serde_json::Value) -> anyhow::Result<()> {
        if let Some(val) = params.get("min") {
            self.min = val
                .as_u64()
                .ok_or_else(|| anyhow::anyhow!("min must be a positive integer"))?
                as usize;
        }
        if let Some(val) = params.get("max") {
            self.max = val
                .as_u64()
                .ok_or_else(|| anyhow::anyhow!("max must be a positive integer"))?
                as usize;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// meta.desc_missing
// ---------------------------------------------------------------------------

pub struct DescMissing;

impl SeoRule for DescMissing {
    fn id(&self) -> &str {
        "meta.desc_missing"
    }
    fn name(&self) -> &str {
        "Missing Meta Description"
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Meta
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn evaluate(&self, page: &ParsedPage, _ctx: &CrawlContext) -> Vec<Issue> {
        if page.meta_description.is_none() || page.meta_description.as_deref() == Some("") {
            vec![Issue {
                rule_id: self.id().to_string(),
                severity: self.default_severity(),
                category: self.category(),
                message: "Page is missing a meta description.".to_string(),
                detail: None,
            }]
        } else {
            vec![]
        }
    }
}

// ---------------------------------------------------------------------------
// meta.desc_length
// ---------------------------------------------------------------------------

pub struct DescLength {
    pub min: usize,
    pub max: usize,
}

impl Default for DescLength {
    fn default() -> Self {
        Self { min: 50, max: 160 }
    }
}

impl SeoRule for DescLength {
    fn id(&self) -> &str {
        "meta.desc_length"
    }
    fn name(&self) -> &str {
        "Meta Description Length"
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Meta
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn evaluate(&self, page: &ParsedPage, _ctx: &CrawlContext) -> Vec<Issue> {
        if let Some(desc) = &page.meta_description {
            let len = desc.chars().count();
            if len < self.min {
                vec![Issue {
                    rule_id: self.id().to_string(),
                    severity: self.default_severity(),
                    category: self.category(),
                    message: format!(
                        "Meta description is too short ({} chars, minimum {} recommended).",
                        len, self.min
                    ),
                    detail: Some(
                        serde_json::json!({ "length": len, "min": self.min, "max": self.max }),
                    ),
                }]
            } else if len > self.max {
                vec![Issue {
                    rule_id: self.id().to_string(),
                    severity: self.default_severity(),
                    category: self.category(),
                    message: format!(
                        "Meta description is too long ({} chars, maximum {} recommended).",
                        len, self.max
                    ),
                    detail: Some(
                        serde_json::json!({ "length": len, "min": self.min, "max": self.max }),
                    ),
                }]
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    fn configure(&mut self, params: &serde_json::Value) -> anyhow::Result<()> {
        if let Some(val) = params.get("min") {
            self.min = val
                .as_u64()
                .ok_or_else(|| anyhow::anyhow!("min must be a positive integer"))?
                as usize;
        }
        if let Some(val) = params.get("max") {
            self.max = val
                .as_u64()
                .ok_or_else(|| anyhow::anyhow!("max must be a positive integer"))?
                as usize;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// meta.canonical_missing
// ---------------------------------------------------------------------------

pub struct CanonicalMissing;

impl SeoRule for CanonicalMissing {
    fn id(&self) -> &str {
        "meta.canonical_missing"
    }
    fn name(&self) -> &str {
        "Missing Canonical Tag"
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Meta
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn evaluate(&self, page: &ParsedPage, _ctx: &CrawlContext) -> Vec<Issue> {
        // Only flag on indexable pages (no noindex directive).
        let is_noindex = page
            .meta_robots
            .as_deref()
            .map(|r| r.to_lowercase().contains("noindex"))
            .unwrap_or(false);

        if !is_noindex && page.canonical.is_none() {
            vec![Issue {
                rule_id: self.id().to_string(),
                severity: self.default_severity(),
                category: self.category(),
                message: "Indexable page is missing a canonical tag.".to_string(),
                detail: None,
            }]
        } else {
            vec![]
        }
    }
}

// ---------------------------------------------------------------------------
// meta.canonical_mismatch
// ---------------------------------------------------------------------------

pub struct CanonicalMismatch;

impl SeoRule for CanonicalMismatch {
    fn id(&self) -> &str {
        "meta.canonical_mismatch"
    }
    fn name(&self) -> &str {
        "Canonical URL Mismatch"
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Meta
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn evaluate(&self, page: &ParsedPage, _ctx: &CrawlContext) -> Vec<Issue> {
        if let Some(canonical) = &page.canonical {
            if canonical != &page.url {
                vec![Issue {
                    rule_id: self.id().to_string(),
                    severity: self.default_severity(),
                    category: self.category(),
                    message: format!("Canonical URL ({}) differs from page URL.", canonical),
                    detail: Some(
                        serde_json::json!({ "canonical": canonical, "page_url": &page.url }),
                    ),
                }]
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }
}

// ---------------------------------------------------------------------------
// meta.viewport_missing
// ---------------------------------------------------------------------------

pub struct ViewportMissing;

impl SeoRule for ViewportMissing {
    fn id(&self) -> &str {
        "meta.viewport_missing"
    }
    fn name(&self) -> &str {
        "Missing Viewport Meta Tag"
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Meta
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn evaluate(&self, page: &ParsedPage, _ctx: &CrawlContext) -> Vec<Issue> {
        if page.viewport.is_none() {
            vec![Issue {
                rule_id: self.id().to_string(),
                severity: self.default_severity(),
                category: self.category(),
                message: "Page is missing a viewport meta tag (affects mobile rendering)."
                    .to_string(),
                detail: None,
            }]
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx() -> CrawlContext {
        CrawlContext {
            root_domain: "example.com".into(),
            cross_page_available: false,
        }
    }

    #[test]
    fn test_title_missing_flags_empty() {
        let rule = TitleMissing;
        let page = ParsedPage {
            url: "https://example.com".into(),
            title: None,
            ..Default::default()
        };
        let issues = rule.evaluate(&page, &make_ctx());
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn test_title_missing_passes_with_title() {
        let rule = TitleMissing;
        let page = ParsedPage {
            url: "https://example.com".into(),
            title: Some("Hello".into()),
            ..Default::default()
        };
        let issues = rule.evaluate(&page, &make_ctx());
        assert!(issues.is_empty());
    }

    #[test]
    fn test_title_length_too_short() {
        let rule = TitleLength { min: 30, max: 60 };
        let page = ParsedPage {
            title: Some("Short".into()),
            ..Default::default()
        };
        let issues = rule.evaluate(&page, &make_ctx());
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("too short"));
    }

    #[test]
    fn test_title_length_ok() {
        let rule = TitleLength { min: 5, max: 60 };
        let page = ParsedPage {
            title: Some("A reasonably sized page title".into()),
            ..Default::default()
        };
        let issues = rule.evaluate(&page, &make_ctx());
        assert!(issues.is_empty());
    }
}
