//! Performance rules: page size and response time checks.

use crate::crawler::ParsedPage;
use crate::rules::rule::{CrawlContext, Issue, SeoRule};
use crate::{RuleCategory, Severity};

pub struct LargePage {
    pub max_bytes: usize,
}

impl Default for LargePage {
    fn default() -> Self {
        Self {
            max_bytes: 3_000_000, // 3 MB
        }
    }
}

impl SeoRule for LargePage {
    fn id(&self) -> &str {
        "perf.large_page"
    }
    fn name(&self) -> &str {
        "Large Page Size"
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Performance
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn evaluate(&self, page: &ParsedPage, _ctx: &CrawlContext) -> Vec<Issue> {
        if let Some(size) = page.body_size {
            if size > self.max_bytes {
                return vec![Issue {
                    rule_id: self.id().into(),
                    severity: self.default_severity(),
                    category: self.category(),
                    message: format!(
                        "Page size is {} bytes ({:.1} MB), exceeds {} byte threshold.",
                        size,
                        size as f64 / 1_000_000.0,
                        self.max_bytes
                    ),
                    detail: Some(serde_json::json!({
                        "body_size": size,
                        "threshold": self.max_bytes,
                    })),
                }];
            }
        }
        vec![]
    }

    fn config_schema(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "max_bytes": {
                    "type": "integer",
                    "default": 3_000_000,
                    "description": "Maximum page size in bytes"
                }
            }
        }))
    }

    fn configure(&mut self, params: &serde_json::Value) -> anyhow::Result<()> {
        if let Some(val) = params.get("max_bytes") {
            self.max_bytes = val
                .as_u64()
                .ok_or_else(|| anyhow::anyhow!("max_bytes must be a positive integer"))?
                as usize;
        }
        Ok(())
    }
}

pub struct SlowResponse {
    pub max_ms: u32,
}

impl Default for SlowResponse {
    fn default() -> Self {
        Self { max_ms: 3000 } // 3 seconds
    }
}

impl SeoRule for SlowResponse {
    fn id(&self) -> &str {
        "perf.slow_response"
    }
    fn name(&self) -> &str {
        "Slow Response Time"
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Performance
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn evaluate(&self, page: &ParsedPage, _ctx: &CrawlContext) -> Vec<Issue> {
        if let Some(ms) = page.response_time_ms {
            if ms > self.max_ms {
                return vec![Issue {
                    rule_id: self.id().into(),
                    severity: self.default_severity(),
                    category: self.category(),
                    message: format!(
                        "Response time is {}ms, exceeds {}ms threshold.",
                        ms, self.max_ms
                    ),
                    detail: Some(serde_json::json!({
                        "response_time_ms": ms,
                        "threshold_ms": self.max_ms,
                    })),
                }];
            }
        }
        vec![]
    }

    fn config_schema(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "max_ms": {
                    "type": "integer",
                    "default": 3000,
                    "description": "Maximum response time in milliseconds"
                }
            }
        }))
    }

    fn configure(&mut self, params: &serde_json::Value) -> anyhow::Result<()> {
        if let Some(val) = params.get("max_ms") {
            self.max_ms = val
                .as_u64()
                .ok_or_else(|| anyhow::anyhow!("max_ms must be a positive integer"))?
                as u32;
        }
        Ok(())
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

    fn make_page() -> ParsedPage {
        ParsedPage {
            url: "https://example.com/".into(),
            parse_ok: true,
            ..Default::default()
        }
    }

    #[test]
    fn test_large_page_flags_oversized() {
        let rule = LargePage::default();
        let ctx = make_ctx();
        let mut page = make_page();
        page.body_size = Some(5_000_000);

        let issues = rule.evaluate(&page, &ctx);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "perf.large_page");
    }

    #[test]
    fn test_large_page_passes_normal() {
        let rule = LargePage::default();
        let ctx = make_ctx();
        let mut page = make_page();
        page.body_size = Some(100_000);

        let issues = rule.evaluate(&page, &ctx);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_large_page_none_body_size() {
        let rule = LargePage::default();
        let ctx = make_ctx();
        let page = make_page();

        let issues = rule.evaluate(&page, &ctx);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_large_page_configurable() {
        let mut rule = LargePage::default();
        rule.configure(&serde_json::json!({ "max_bytes": 1000 }))
            .unwrap();
        assert_eq!(rule.max_bytes, 1000);

        let ctx = make_ctx();
        let mut page = make_page();
        page.body_size = Some(2000);

        let issues = rule.evaluate(&page, &ctx);
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn test_slow_response_flags_slow() {
        let rule = SlowResponse::default();
        let ctx = make_ctx();
        let mut page = make_page();
        page.response_time_ms = Some(5000);

        let issues = rule.evaluate(&page, &ctx);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "perf.slow_response");
    }

    #[test]
    fn test_slow_response_passes_fast() {
        let rule = SlowResponse::default();
        let ctx = make_ctx();
        let mut page = make_page();
        page.response_time_ms = Some(200);

        let issues = rule.evaluate(&page, &ctx);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_slow_response_none() {
        let rule = SlowResponse::default();
        let ctx = make_ctx();
        let page = make_page();

        let issues = rule.evaluate(&page, &ctx);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_slow_response_configurable() {
        let mut rule = SlowResponse::default();
        rule.configure(&serde_json::json!({ "max_ms": 500 }))
            .unwrap();
        assert_eq!(rule.max_ms, 500);

        let ctx = make_ctx();
        let mut page = make_page();
        page.response_time_ms = Some(1000);

        let issues = rule.evaluate(&page, &ctx);
        assert_eq!(issues.len(), 1);
    }
}
