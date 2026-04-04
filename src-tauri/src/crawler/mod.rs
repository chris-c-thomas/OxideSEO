//! Core crawl engine: orchestrator, frontier, fetcher, parser, politeness.
//!
//! The crawl engine uses a channel-based actor architecture:
//! - **Orchestrator** (single tokio task): dequeues URLs, enforces politeness, dispatches work
//! - **Fetch Pool** (N async workers): HTTP requests via reqwest
//! - **Parse + Analyze** (rayon thread pool): HTML parsing + rule evaluation
//! - **Storage Writer** (dedicated thread): batched SQLite writes

pub mod engine;
pub mod fetcher;
pub mod frontier;
pub mod parser;
pub mod politeness;
pub mod robots;

use serde::{Deserialize, Serialize};

/// Result of fetching a single URL.
#[derive(Debug, Clone)]
pub struct FetchResult {
    pub url: String,
    pub final_url: String,
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body_bytes: Vec<u8>,
    pub body_size: usize,
    pub body_hash: Option<[u8; 32]>,
    pub content_type: Option<String>,
    pub response_time_ms: u32,
    pub redirect_chain: Vec<RedirectHop>,
}

/// Single hop in a redirect chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedirectHop {
    pub url: String,
    pub status_code: u16,
}

/// Fully parsed page with all extracted SEO-relevant data.
#[derive(Debug, Clone, Default)]
pub struct ParsedPage {
    pub url: String,
    pub title: Option<String>,
    pub meta_description: Option<String>,
    pub meta_robots: Option<String>,
    pub canonical: Option<String>,
    pub viewport: Option<String>,
    pub h1s: Vec<String>,
    pub h2s: Vec<String>,
    pub h3s: Vec<String>,
    pub h4s: Vec<String>,
    pub h5s: Vec<String>,
    pub h6s: Vec<String>,
    pub links: Vec<ExtractedLink>,
    pub images: Vec<ExtractedImage>,
    pub scripts: Vec<String>,
    pub stylesheets: Vec<String>,
    pub word_count: u32,
    pub base_url: Option<String>,
    /// Whether the parse completed successfully.
    pub parse_ok: bool,
}

/// A link extracted from HTML.
#[derive(Debug, Clone)]
pub struct ExtractedLink {
    pub href: String,
    pub anchor_text: Option<String>,
    pub rel: Option<String>,
    pub is_internal: bool,
    pub is_nofollow: bool,
}

/// An image extracted from HTML.
#[derive(Debug, Clone)]
pub struct ExtractedImage {
    pub src: String,
    pub alt: Option<String>,
    pub has_srcset: bool,
}
