//! Security rules: HTTPS and mixed content checks.

use crate::crawler::ParsedPage;
use crate::rules::rule::{CrawlContext, Issue, SeoRule};
use crate::{RuleCategory, Severity};

pub struct MixedContent;

impl SeoRule for MixedContent {
    fn id(&self) -> &str { "security.mixed_content" }
    fn name(&self) -> &str { "Mixed Content" }
    fn category(&self) -> RuleCategory { RuleCategory::Security }
    fn default_severity(&self) -> Severity { Severity::Error }

    fn evaluate(&self, page: &ParsedPage, _ctx: &CrawlContext) -> Vec<Issue> {
        if !page.url.starts_with("https://") { return vec![]; }

        let http_resources: Vec<String> = page.scripts.iter()
            .chain(page.stylesheets.iter())
            .chain(page.images.iter().map(|i| &i.src))
            .filter(|url| url.starts_with("http://"))
            .cloned()
            .collect();

        if !http_resources.is_empty() {
            vec![Issue {
                rule_id: self.id().into(), severity: self.default_severity(), category: self.category(),
                message: format!("HTTPS page loads {} HTTP resource(s).", http_resources.len()),
                detail: Some(serde_json::json!({ "http_resources": http_resources })),
            }]
        } else { vec![] }
    }
}

pub struct HttpPage;

impl SeoRule for HttpPage {
    fn id(&self) -> &str { "security.http_page" }
    fn name(&self) -> &str { "Page Served Over HTTP" }
    fn category(&self) -> RuleCategory { RuleCategory::Security }
    fn default_severity(&self) -> Severity { Severity::Warning }

    fn evaluate(&self, page: &ParsedPage, _ctx: &CrawlContext) -> Vec<Issue> {
        if page.url.starts_with("http://") {
            vec![Issue {
                rule_id: self.id().into(), severity: self.default_severity(), category: self.category(),
                message: "Page is served over HTTP instead of HTTPS.".into(),
                detail: None,
            }]
        } else { vec![] }
    }
}
