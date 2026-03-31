//! Settings commands: application preferences and rule configuration.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::storage::db::Database;

/// Application-level settings persisted across sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// Default crawl configuration applied when creating new crawls.
    pub default_crawl_config: serde_json::Value,
    /// UI theme preference.
    pub theme: ThemePreference,
    /// Default export format.
    pub default_export_format: ExportFormat,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreference {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Csv,
    Json,
    Html,
    Xlsx,
}

/// Per-rule configuration overlay.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleConfigOverride {
    pub rule_id: String,
    pub enabled: Option<bool>,
    pub severity: Option<String>,
    pub params: Option<serde_json::Value>,
}

#[tauri::command]
pub async fn get_settings(db: State<'_, Database>) -> Result<AppSettings, String> {
    // TODO(phase-4): Read from settings table or return defaults.
    Ok(AppSettings {
        default_crawl_config: serde_json::json!({}),
        theme: ThemePreference::System,
        default_export_format: ExportFormat::Csv,
    })
}

#[tauri::command]
pub async fn set_settings(
    settings: AppSettings,
    db: State<'_, Database>,
) -> Result<(), String> {
    // TODO(phase-4): Persist to settings table.
    Ok(())
}

#[tauri::command]
pub async fn get_rule_config(
    db: State<'_, Database>,
) -> Result<Vec<RuleConfigOverride>, String> {
    // TODO(phase-3): Read from rule_config table.
    Ok(Vec::new())
}

#[tauri::command]
pub async fn set_rule_config(
    overrides: Vec<RuleConfigOverride>,
    db: State<'_, Database>,
) -> Result<(), String> {
    // TODO(phase-3): Persist to rule_config table.
    Ok(())
}
