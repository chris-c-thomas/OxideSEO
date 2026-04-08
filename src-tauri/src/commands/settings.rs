//! Settings commands: application preferences and rule configuration.

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tauri::State;

use std::sync::Arc;

use crate::Severity;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Csv,
    Json,
    Html,
    Pdf,
    Xlsx,
    /// Plugin-provided export format. The string is the plugin name.
    Plugin(String),
}

/// Per-rule configuration overlay.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleConfigOverride {
    pub rule_id: String,
    pub enabled: Option<bool>,
    pub severity: Option<Severity>,
    pub params: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Settings commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_settings(db: State<'_, Arc<Database>>) -> Result<AppSettings, String> {
    db.with_conn(|conn| {
        let theme: ThemePreference = match queries::get_setting(conn, "theme")? {
            Some(v) => match v.as_str() {
                "system" => ThemePreference::System,
                "light" => ThemePreference::Light,
                "dark" => ThemePreference::Dark,
                _ => {
                    tracing::warn!(key = "theme", value = %v, "Invalid setting value, using default");
                    ThemePreference::System
                }
            },
            None => ThemePreference::System,
        };

        let default_export_format: ExportFormat =
            match queries::get_setting(conn, "default_export_format")? {
                Some(v) => serde_json::from_str(&v).unwrap_or_else(|_| {
                    tracing::warn!(key = "default_export_format", value = %v, "Invalid setting value, using default");
                    ExportFormat::Csv
                }),
                None => ExportFormat::Csv,
            };

        let default_crawl_config: serde_json::Value =
            match queries::get_setting(conn, "default_crawl_config")? {
                Some(v) => serde_json::from_str(&v).unwrap_or_else(|e| {
                    tracing::warn!(key = "default_crawl_config", error = %e, "Invalid JSON in setting, using default");
                    serde_json::json!({})
                }),
                None => serde_json::json!({}),
            };

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
            .context("failed to serialize theme")?
            .as_str()
            .map(String::from)
            .ok_or_else(|| anyhow::anyhow!("theme did not serialize to a string"))?;
        queries::set_setting(conn, "theme", &theme_str)?;

        // Serialize as JSON string to support both simple ("csv") and complex ({"plugin":"name"}) variants.
        let export_str = serde_json::to_string(&settings.default_export_format)
            .context("failed to serialize default_export_format")?;
        queries::set_setting(conn, "default_export_format", &export_str)?;

        let config_str = serde_json::to_string(&settings.default_crawl_config)
            .context("failed to serialize default_crawl_config")?;
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
                let rule_id: String = row.get(0)?;
                let severity_str: Option<String> = row.get(2)?;
                let severity = severity_str.and_then(|s| {
                    s.parse::<Severity>().map_err(|e| {
                        tracing::warn!(rule_id = %rule_id, error = %e, "Invalid severity in rule_config, ignoring");
                        e
                    }).ok()
                });
                let params_str: Option<String> = row.get(3)?;
                let params = params_str.and_then(|s| {
                    serde_json::from_str(&s).map_err(|e| {
                        tracing::warn!(rule_id = %rule_id, error = %e, "Invalid params JSON in rule_config, ignoring");
                        e
                    }).ok()
                });
                Ok(RuleConfigOverride {
                    rule_id,
                    enabled: row.get(1)?,
                    severity,
                    params,
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
            let severity_str = o.severity.map(|s| s.to_string());
            let params_json = o
                .params
                .as_ref()
                .map(serde_json::to_string)
                .transpose()
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            stmt.execute(rusqlite::params![o.rule_id, o.enabled, severity_str, params_json])?;
        }

        drop(stmt);
        tx.commit()?;
        Ok(())
    })
    .map_err(|e| format!("{e:#}"))
}
