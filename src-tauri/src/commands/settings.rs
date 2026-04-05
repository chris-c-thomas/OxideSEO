//! Settings commands: application preferences and rule configuration.

use serde::{Deserialize, Serialize};
use tauri::State;

use std::sync::Arc;

use crate::storage::db::Database;
use crate::storage::queries;

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

// ---------------------------------------------------------------------------
// Settings commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_settings(db: State<'_, Arc<Database>>) -> Result<AppSettings, String> {
    db.with_conn(|conn| {
        let theme: ThemePreference = queries::get_setting(conn, "theme")?
            .and_then(|v| serde_json::from_str(&format!("\"{v}\"")).ok())
            .unwrap_or(ThemePreference::System);

        let default_export_format: ExportFormat =
            queries::get_setting(conn, "default_export_format")?
                .and_then(|v| serde_json::from_str(&format!("\"{v}\"")).ok())
                .unwrap_or(ExportFormat::Csv);

        let default_crawl_config: serde_json::Value =
            queries::get_setting(conn, "default_crawl_config")?
                .and_then(|v| serde_json::from_str(&v).ok())
                .unwrap_or_else(|| serde_json::json!({}));

        Ok(AppSettings {
            default_crawl_config,
            theme,
            default_export_format,
        })
    })
    .map_err(|e| format!("{e:#}"))
}

#[tauri::command]
pub async fn set_settings(
    settings: AppSettings,
    db: State<'_, Arc<Database>>,
) -> Result<(), String> {
    db.with_conn(|conn| {
        let theme_str = serde_json::to_value(settings.theme)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "system".into());
        queries::set_setting(conn, "theme", &theme_str)?;

        let export_str = serde_json::to_value(settings.default_export_format)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "csv".into());
        queries::set_setting(conn, "default_export_format", &export_str)?;

        let config_str =
            serde_json::to_string(&settings.default_crawl_config).unwrap_or_else(|_| "{}".into());
        queries::set_setting(conn, "default_crawl_config", &config_str)?;

        Ok(())
    })
    .map_err(|e| format!("{e:#}"))
}

// ---------------------------------------------------------------------------
// Rule config commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_rule_config(
    db: State<'_, Arc<Database>>,
) -> Result<Vec<RuleConfigOverride>, String> {
    db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT rule_id, enabled, severity, params FROM rule_config WHERE profile = 'default'",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(RuleConfigOverride {
                    rule_id: row.get(0)?,
                    enabled: row.get(1)?,
                    severity: row.get(2)?,
                    params: row
                        .get::<_, Option<String>>(3)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    })
    .map_err(|e| format!("{e:#}"))
}

#[tauri::command]
pub async fn set_rule_config(
    overrides: Vec<RuleConfigOverride>,
    db: State<'_, Arc<Database>>,
) -> Result<(), String> {
    db.with_conn_mut(|conn| {
        let tx = conn.transaction()?;

        tx.execute("DELETE FROM rule_config WHERE profile = 'default'", [])?;

        let mut stmt = tx.prepare(
            "INSERT INTO rule_config (profile, rule_id, enabled, severity, params) VALUES ('default', ?1, ?2, ?3, ?4)",
        )?;

        for o in &overrides {
            let params_json = o
                .params
                .as_ref()
                .and_then(|p| serde_json::to_string(p).ok());
            stmt.execute(rusqlite::params![o.rule_id, o.enabled, o.severity, params_json])?;
        }

        drop(stmt);
        tx.commit()?;
        Ok(())
    })
    .map_err(|e| format!("{e:#}"))
}
