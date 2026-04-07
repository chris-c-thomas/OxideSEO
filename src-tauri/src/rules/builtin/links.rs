//! Link rules: broken links, redirect chains, nofollow.
//!
//! Note: These rules operate on per-page data. Cross-page analysis
//! (orphan pages, broken external links) runs post-crawl in Phase 3.

use crate::crawler::ParsedPage;
use crate::rules::rule::{CrawlContext, Issue, SeoRule};
use crate::{RuleCategory, Severity};

pub struct BrokenInternal;

impl SeoRule for BrokenInternal {
    fn id(&self) -> &str {
        "links.broken_internal"
    }
    fn name(&self) -> &str {
        "Broken Internal Link"
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Links
    }
    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn evaluate(&self, _page: &ParsedPage, _ctx: &CrawlContext) -> Vec<Issue> {
        // Per-page stub — cross-page detection in PostCrawlAnalyzer.
        vec![]
    }
}

pub struct RedirectChain;

impl SeoRule for RedirectChain {
    fn id(&self) -> &str {
        "links.redirect_chain"
    }
    fn name(&self) -> &str {
        "Redirect Chain"
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Links
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn evaluate(&self, _page: &ParsedPage, _ctx: &CrawlContext) -> Vec<Issue> {
        // Per-page stub — cross-page detection in PostCrawlAnalyzer.
        vec![]
    }
}

pub struct NofollowInternal;

impl SeoRule for NofollowInternal {
    fn id(&self) -> &str {
        "links.nofollow_internal"
    }
    fn name(&self) -> &str {
        "Internal Nofollow Link"
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Links
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn evaluate(&self, page: &ParsedPage, _ctx: &CrawlContext) -> Vec<Issue> {
        let nofollow_internals: Vec<_> = page
            .links
            .iter()
            .filter(|l| l.is_internal && l.is_nofollow)
            .collect();

        if !nofollow_internals.is_empty() {
            vec![Issue {
                rule_id: self.id().into(),
                severity: self.default_severity(),
                category: self.category(),
                message: format!(
                    "{} internal link(s) have rel=\"nofollow\".",
                    nofollow_internals.len()
                ),
                detail: Some(serde_json::json!({
                    "links": nofollow_internals.iter().map(|l| &l.href).collect::<Vec<_>>()
                })),
            }]
        } else {
            vec![]
        }
    }
}
