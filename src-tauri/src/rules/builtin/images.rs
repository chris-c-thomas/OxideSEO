//! Image rules: alt attribute checks.

use crate::crawler::ParsedPage;
use crate::rules::rule::{CrawlContext, Issue, SeoRule};
use crate::{RuleCategory, Severity};

pub struct AltMissing;

impl SeoRule for AltMissing {
    fn id(&self) -> &str {
        "images.alt_missing"
    }
    fn name(&self) -> &str {
        "Image Missing Alt Attribute"
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Images
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn evaluate(&self, page: &ParsedPage, _ctx: &CrawlContext) -> Vec<Issue> {
        let missing: Vec<_> = page.images.iter().filter(|i| i.alt.is_none()).collect();
        if !missing.is_empty() {
            vec![Issue {
                rule_id: self.id().into(),
                severity: self.default_severity(),
                category: self.category(),
                message: format!("{} image(s) missing alt attribute.", missing.len()),
                detail: Some(
                    serde_json::json!({ "images": missing.iter().map(|i| &i.src).collect::<Vec<_>>() }),
                ),
            }]
        } else {
            vec![]
        }
    }
}

pub struct AltEmpty;

impl SeoRule for AltEmpty {
    fn id(&self) -> &str {
        "images.alt_empty"
    }
    fn name(&self) -> &str {
        "Image With Empty Alt"
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Images
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn evaluate(&self, page: &ParsedPage, _ctx: &CrawlContext) -> Vec<Issue> {
        let empty: Vec<_> = page
            .images
            .iter()
            .filter(|i| i.alt.as_deref() == Some(""))
            .collect();
        if !empty.is_empty() {
            vec![Issue {
                rule_id: self.id().into(),
                severity: self.default_severity(),
                category: self.category(),
                message: format!(
                    "{} image(s) have empty alt attribute (may be decorative).",
                    empty.len()
                ),
                detail: Some(
                    serde_json::json!({ "images": empty.iter().map(|i| &i.src).collect::<Vec<_>>() }),
                ),
            }]
        } else {
            vec![]
        }
    }
}
