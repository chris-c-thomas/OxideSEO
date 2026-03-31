//! OxideSEO — Open-source SEO crawler and audit platform.
//!
//! This crate provides the core crawl engine, SEO rule evaluation,
//! storage layer, and Tauri IPC command handlers for the OxideSEO
//! desktop application.

pub mod ai;
pub mod commands;
pub mod crawler;
pub mod rules;
pub mod storage;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Shared enums used across modules
// ---------------------------------------------------------------------------

/// State machine for every crawled URL.
/// Drives storage schema, UI indicators, and error recovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UrlState {
    Discovered,
    Queued,
    Fetching,
    Fetched,
    Parsed,
    Analyzed,
    Errored,
}

impl std::fmt::Display for UrlState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Discovered => "discovered",
            Self::Queued => "queued",
            Self::Fetching => "fetching",
            Self::Fetched => "fetched",
            Self::Parsed => "parsed",
            Self::Analyzed => "analyzed",
            Self::Errored => "errored",
        };
        write!(f, "{}", s)
    }
}

/// Crawl lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrawlState {
    Created,
    Running,
    Paused,
    Completed,
    Stopped,
    Error,
}

/// SEO issue severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// SEO rule categories for grouping in UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleCategory {
    Meta,
    Content,
    Links,
    Images,
    Performance,
    Security,
    Indexability,
    Structured,
    International,
}

/// Link types stored in the `links` table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkType {
    Anchor,
    Image,
    Script,
    Stylesheet,
    Canonical,
    Redirect,
}

impl std::fmt::Display for LinkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Anchor => "a",
            Self::Image => "img",
            Self::Script => "script",
            Self::Stylesheet => "link",
            Self::Canonical => "canonical",
            Self::Redirect => "redirect",
        };
        write!(f, "{}", s)
    }
}
