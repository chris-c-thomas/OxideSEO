//! Performance rules: page size and response time checks.

use crate::crawler::ParsedPage;
use crate::rules::rule::{CrawlContext, Issue, SeoRule};
use crate::{RuleCategory, Severity};

pub struct LargePage;

impl SeoRule for LargePage {
    fn id(&self) -> &str { "perf.large_page" }
    fn name(&self) -> &str { "Large Page Size" }
    fn category(&self) -> RuleCategory { RuleCategory::Performance }
    fn default_severity(&self) -> Severity { Severity::Warning }

    fn evaluate(&self, _page: &ParsedPage, _ctx: &CrawlContext) -> Vec<Issue> {
        // TODO(phase-3): Check body_size from FetchResult (not available on ParsedPage).
        // This rule will be evaluated in the pipeline where FetchResult is accessible.
        vec![]
    }
}

pub struct SlowResponse;

impl SeoRule for SlowResponse {
    fn id(&self) -> &str { "perf.slow_response" }
    fn name(&self) -> &str { "Slow Response Time" }
    fn category(&self) -> RuleCategory { RuleCategory::Performance }
    fn default_severity(&self) -> Severity { Severity::Warning }

    fn evaluate(&self, _page: &ParsedPage, _ctx: &CrawlContext) -> Vec<Issue> {
        // TODO(phase-3): Check response_time_ms from FetchResult.
        vec![]
    }
}
