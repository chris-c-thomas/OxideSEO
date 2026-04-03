//! Content rules: heading structure and thin content detection.

use crate::crawler::ParsedPage;
use crate::rules::rule::{CrawlContext, Issue, SeoRule};
use crate::{RuleCategory, Severity};

pub struct H1Missing;

impl SeoRule for H1Missing {
    fn id(&self) -> &str {
        "content.h1_missing"
    }
    fn name(&self) -> &str {
        "Missing H1 Tag"
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Content
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn evaluate(&self, page: &ParsedPage, _ctx: &CrawlContext) -> Vec<Issue> {
        if page.h1s.is_empty() {
            vec![Issue {
                rule_id: self.id().into(),
                severity: self.default_severity(),
                category: self.category(),
                message: "Page has no H1 tag.".into(),
                detail: None,
            }]
        } else {
            vec![]
        }
    }
}

pub struct H1Multiple;

impl SeoRule for H1Multiple {
    fn id(&self) -> &str {
        "content.h1_multiple"
    }
    fn name(&self) -> &str {
        "Multiple H1 Tags"
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Content
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn evaluate(&self, page: &ParsedPage, _ctx: &CrawlContext) -> Vec<Issue> {
        if page.h1s.len() > 1 {
            vec![Issue {
                rule_id: self.id().into(),
                severity: self.default_severity(),
                category: self.category(),
                message: format!("Page has {} H1 tags (expected 1).", page.h1s.len()),
                detail: Some(serde_json::json!({ "h1s": page.h1s })),
            }]
        } else {
            vec![]
        }
    }
}

pub struct HeadingHierarchy;

impl SeoRule for HeadingHierarchy {
    fn id(&self) -> &str {
        "content.heading_hierarchy"
    }
    fn name(&self) -> &str {
        "Heading Hierarchy Skip"
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Content
    }
    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn evaluate(&self, page: &ParsedPage, _ctx: &CrawlContext) -> Vec<Issue> {
        // Check if heading levels skip (e.g., H1 then H3 with no H2).
        let mut has_level = [false; 6];
        if !page.h1s.is_empty() {
            has_level[0] = true;
        }
        if !page.h2s.is_empty() {
            has_level[1] = true;
        }
        if !page.h3s.is_empty() {
            has_level[2] = true;
        }
        if !page.h4s.is_empty() {
            has_level[3] = true;
        }
        if !page.h5s.is_empty() {
            has_level[4] = true;
        }
        if !page.h6s.is_empty() {
            has_level[5] = true;
        }

        let mut skips = Vec::new();
        let mut last_present = 0;
        for (i, &present) in has_level.iter().enumerate() {
            if present && i > last_present + 1 && last_present < i {
                // Check if any level between last_present and i is missing.
                let gap = (last_present + 1..i).any(|j| !has_level[j]);
                if gap {
                    skips.push(format!("H{} to H{}", last_present + 1, i + 1));
                }
            }
            if present {
                last_present = i;
            }
        }

        if !skips.is_empty() {
            vec![Issue {
                rule_id: self.id().into(),
                severity: self.default_severity(),
                category: self.category(),
                message: format!("Heading levels skipped: {}.", skips.join(", ")),
                detail: None,
            }]
        } else {
            vec![]
        }
    }
}

pub struct ThinContent {
    pub min_words: u32,
}

impl Default for ThinContent {
    fn default() -> Self {
        Self { min_words: 200 }
    }
}

impl SeoRule for ThinContent {
    fn id(&self) -> &str {
        "content.thin_content"
    }
    fn name(&self) -> &str {
        "Thin Content"
    }
    fn category(&self) -> RuleCategory {
        RuleCategory::Content
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn evaluate(&self, page: &ParsedPage, _ctx: &CrawlContext) -> Vec<Issue> {
        if page.word_count > 0 && page.word_count < self.min_words {
            vec![Issue {
                rule_id: self.id().into(),
                severity: self.default_severity(),
                category: self.category(),
                message: format!(
                    "Page has thin content ({} words, minimum {} recommended).",
                    page.word_count, self.min_words
                ),
                detail: Some(
                    serde_json::json!({ "word_count": page.word_count, "min_words": self.min_words }),
                ),
            }]
        } else {
            vec![]
        }
    }

    fn configure(&mut self, params: &serde_json::Value) -> anyhow::Result<()> {
        if let Some(min) = params.get("min_words").and_then(|v| v.as_u64()) {
            self.min_words = min as u32;
        }
        Ok(())
    }
}
