//! SEO rule engine: trait definitions, registry, and built-in rule implementations.
//!
//! Rules are decoupled from the crawl engine — they evaluate a `ParsedPage`
//! and return zero or more `Issue` structs. Each rule is independently
//! configurable, testable, and can be toggled per crawl profile.

pub mod builtin;
pub mod engine;
pub mod post_crawl;
pub mod rule;

pub use engine::RuleRegistry;
pub use post_crawl::PostCrawlAnalyzer;
pub use rule::{Issue, SeoRule};
