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

use std::fmt;
use std::str::FromStr;

use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
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

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        };
        f.write_str(s)
    }
}

impl FromStr for Severity {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "error" => Ok(Severity::Error),
            "warning" => Ok(Severity::Warning),
            "info" => Ok(Severity::Info),
            other => Err(format!("unknown severity: {other}")),
        }
    }
}

impl ToSql for Severity {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.to_string()))
    }
}

impl FromSql for Severity {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value
            .as_str()?
            .parse()
            .map_err(|e: String| FromSqlError::Other(e.into()))
    }
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

impl fmt::Display for RuleCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            RuleCategory::Meta => "meta",
            RuleCategory::Content => "content",
            RuleCategory::Links => "links",
            RuleCategory::Images => "images",
            RuleCategory::Performance => "performance",
            RuleCategory::Security => "security",
            RuleCategory::Indexability => "indexability",
            RuleCategory::Structured => "structured",
            RuleCategory::International => "international",
        };
        f.write_str(s)
    }
}

impl FromStr for RuleCategory {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "meta" => Ok(RuleCategory::Meta),
            "content" => Ok(RuleCategory::Content),
            "links" => Ok(RuleCategory::Links),
            "images" => Ok(RuleCategory::Images),
            "performance" => Ok(RuleCategory::Performance),
            "security" => Ok(RuleCategory::Security),
            "indexability" => Ok(RuleCategory::Indexability),
            "structured" => Ok(RuleCategory::Structured),
            "international" => Ok(RuleCategory::International),
            other => Err(format!("unknown rule category: {other}")),
        }
    }
}

impl ToSql for RuleCategory {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.to_string()))
    }
}

impl FromSql for RuleCategory {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value
            .as_str()?
            .parse()
            .map_err(|e: String| FromSqlError::Other(e.into()))
    }
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
