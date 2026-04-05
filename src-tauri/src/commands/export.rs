//! Export commands: CSV, NDJSON, HTML report, and .seocrawl file management.

use serde::{Deserialize, Serialize};
use tauri::State;

use std::sync::Arc;

use crate::commands::settings::ExportFormat;
use crate::storage::db::Database;

// ---------------------------------------------------------------------------
// IPC types
// ---------------------------------------------------------------------------

/// What kind of data to export.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportDataType {
    Pages,
    Issues,
    Links,
    Images,
    FullReport,
}

/// Request payload for the export_data command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRequest {
    pub crawl_id: String,
    pub format: ExportFormat,
    pub data_type: ExportDataType,
    pub columns: Option<Vec<String>>,
}

/// Result returned after a successful export.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub file_path: String,
    pub rows_exported: u64,
}

// ---------------------------------------------------------------------------
// Export command
// ---------------------------------------------------------------------------

/// Export crawl data in the requested format.
#[tauri::command]
pub async fn export_data(
    request: ExportRequest,
    db: State<'_, Arc<Database>>,
    app: tauri::AppHandle,
) -> Result<ExportResult, String> {
    match request.format {
        ExportFormat::Csv => export_csv(&request, &db, &app).await,
        ExportFormat::Json => export_ndjson(&request, &db, &app).await,
        ExportFormat::Html => export_html_report(&request, &db, &app).await,
        ExportFormat::Xlsx => Err("XLSX export is not yet implemented".into()),
    }
}

// ---------------------------------------------------------------------------
// CSV export
// ---------------------------------------------------------------------------

async fn export_csv(
    _request: &ExportRequest,
    _db: &Database,
    _app: &tauri::AppHandle,
) -> Result<ExportResult, String> {
    // TODO(phase-5): Implement CSV export with streaming and column selection.
    Err("CSV export not yet implemented".into())
}

// ---------------------------------------------------------------------------
// NDJSON export
// ---------------------------------------------------------------------------

async fn export_ndjson(
    _request: &ExportRequest,
    _db: &Database,
    _app: &tauri::AppHandle,
) -> Result<ExportResult, String> {
    // TODO(phase-5): Implement NDJSON export with streaming.
    Err("NDJSON export not yet implemented".into())
}

// ---------------------------------------------------------------------------
// HTML report
// ---------------------------------------------------------------------------

async fn export_html_report(
    _request: &ExportRequest,
    _db: &Database,
    _app: &tauri::AppHandle,
) -> Result<ExportResult, String> {
    // TODO(phase-5): Implement HTML report generation.
    Err("HTML report not yet implemented".into())
}

// ---------------------------------------------------------------------------
// .seocrawl file management
// ---------------------------------------------------------------------------

/// Save a crawl to a standalone .seocrawl file.
#[tauri::command]
pub async fn save_crawl_file(
    _crawl_id: String,
    _db: State<'_, Arc<Database>>,
    _app: tauri::AppHandle,
) -> Result<Option<String>, String> {
    // TODO(phase-5): Create new SQLite DB, copy crawl data via ATTACH.
    Err("Save crawl file not yet implemented".into())
}

/// Open a .seocrawl file and import its crawl data.
#[tauri::command]
pub async fn open_crawl_file(
    _db: State<'_, Arc<Database>>,
    _app: tauri::AppHandle,
) -> Result<Option<String>, String> {
    // TODO(phase-5): Open dialog, validate, import via ATTACH, return crawl_id.
    Err("Open crawl file not yet implemented".into())
}
