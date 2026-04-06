//! Export commands: CSV, NDJSON, HTML report, and .seocrawl file management.

use std::fs::File;
use std::io::BufWriter;
use std::sync::Arc;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tauri::State;
use tauri_plugin_dialog::DialogExt;

use crate::commands::settings::ExportFormat;
use crate::storage::db::Database;
use crate::storage::queries;

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
// Helpers
// ---------------------------------------------------------------------------

/// Default column keys for each data type (camelCase, matching serde output).
fn default_columns(data_type: ExportDataType) -> Vec<String> {
    match data_type {
        ExportDataType::Pages => vec![
            "url",
            "statusCode",
            "title",
            "metaDesc",
            "h1",
            "canonical",
            "contentType",
            "responseTimeMs",
            "bodySize",
            "depth",
            "state",
            "robotsDirectives",
            "fetchedAt",
            "errorMessage",
        ],
        ExportDataType::Issues => vec![
            "ruleId",
            "severity",
            "category",
            "message",
            "pageId",
            "detailJson",
        ],
        ExportDataType::Links => vec![
            "sourcePage",
            "targetUrl",
            "anchorText",
            "linkType",
            "isInternal",
            "nofollow",
        ],
        ExportDataType::Images => vec!["sourcePage", "targetUrl", "anchorText", "isInternal"],
        ExportDataType::FullReport => vec![],
    }
    .into_iter()
    .map(String::from)
    .collect()
}

fn data_type_label(data_type: ExportDataType) -> &'static str {
    match data_type {
        ExportDataType::Pages => "pages",
        ExportDataType::Issues => "issues",
        ExportDataType::Links => "links",
        ExportDataType::Images => "images",
        ExportDataType::FullReport => "report",
    }
}

/// Extract the domain from a crawl's start_url for use in filenames.
fn domain_from_crawl(db: &Database, crawl_id: &str) -> String {
    db.with_conn(|conn| {
        let crawl = queries::select_crawl_by_id(conn, crawl_id)?;
        Ok(crawl
            .and_then(|c| url::Url::parse(&c.start_url).ok())
            .and_then(|u| u.host_str().map(String::from))
            .unwrap_or_else(|| "export".into()))
    })
    .unwrap_or_else(|_| "export".into())
}

/// Build a default filename like "example.com-pages-2026-04-05.csv".
fn default_filename(db: &Database, crawl_id: &str, data_type: ExportDataType, ext: &str) -> String {
    let domain = domain_from_crawl(db, crawl_id);
    let date = chrono::Local::now().format("%Y-%m-%d");
    format!("{}-{}-{}.{}", domain, data_type_label(data_type), date, ext)
}

/// Write a single serializable row to CSV, keeping only the requested columns.
fn write_csv_row<W: std::io::Write>(
    writer: &mut csv::Writer<W>,
    row: &impl Serialize,
    columns: &[String],
) -> anyhow::Result<()> {
    let value = serde_json::to_value(row).context("failed to serialize row")?;
    let full_map = match value {
        serde_json::Value::Object(m) => m,
        _ => anyhow::bail!("expected JSON object from row serialization"),
    };
    let record: Vec<String> = columns
        .iter()
        .map(|col| match full_map.get(col) {
            Some(serde_json::Value::Null) | None => String::new(),
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(v) => v.to_string(),
        })
        .collect();
    writer.write_record(&record)?;
    Ok(())
}

/// Write a single serializable row as one NDJSON line, keeping only the requested columns.
fn write_ndjson_row<W: std::io::Write>(
    writer: &mut W,
    row: &impl Serialize,
    columns: &[String],
) -> anyhow::Result<()> {
    let value = serde_json::to_value(row).context("failed to serialize row")?;
    let full_map = match value {
        serde_json::Value::Object(m) => m,
        _ => anyhow::bail!("expected JSON object from row serialization"),
    };
    let filtered: serde_json::Map<String, serde_json::Value> = columns
        .iter()
        .filter_map(|col| full_map.get(col).map(|v| (col.clone(), v.clone())))
        .collect();
    let line = serde_json::to_string(&filtered).context("failed to serialize NDJSON line")?;
    std::io::Write::write_all(writer, line.as_bytes())?;
    std::io::Write::write_all(writer, b"\n")?;
    Ok(())
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
    let result = match request.format {
        ExportFormat::Csv => export_csv(&request, &db, &app),
        ExportFormat::Json => export_ndjson(&request, &db, &app),
        ExportFormat::Html => export_html_report_inner(&request, &db, &app),
    };
    result.map_err(|e| format!("{e:#}"))
}

// ---------------------------------------------------------------------------
// CSV export
// ---------------------------------------------------------------------------

fn export_csv(
    request: &ExportRequest,
    db: &Database,
    app: &tauri::AppHandle,
) -> anyhow::Result<ExportResult> {
    let filename = default_filename(db, &request.crawl_id, request.data_type, "csv");

    let file_path = app
        .dialog()
        .file()
        .add_filter("CSV Files", &["csv"])
        .set_file_name(&filename)
        .set_title("Export as CSV")
        .blocking_save_file()
        .and_then(|fp| fp.into_path().ok())
        .ok_or_else(|| anyhow::anyhow!("Export cancelled"))?;

    let columns = request
        .columns
        .clone()
        .unwrap_or_else(|| default_columns(request.data_type));

    let file = File::create(&file_path)
        .with_context(|| format!("failed to create {}", file_path.display()))?;
    let mut writer = csv::Writer::from_writer(BufWriter::new(file));

    // Write header row.
    writer.write_record(&columns)?;

    let mut rows_exported: u64 = 0;
    let crawl_id = request.crawl_id.clone();

    db.with_conn(|conn| {
        match request.data_type {
            ExportDataType::Pages => {
                queries::for_each_page(conn, &crawl_id, |page| {
                    write_csv_row(&mut writer, &page, &columns)?;
                    rows_exported += 1;
                    Ok(())
                })?;
            }
            ExportDataType::Issues => {
                queries::for_each_issue(conn, &crawl_id, |issue| {
                    write_csv_row(&mut writer, &issue, &columns)?;
                    rows_exported += 1;
                    Ok(())
                })?;
            }
            ExportDataType::Links => {
                queries::for_each_link(conn, &crawl_id, |link| {
                    write_csv_row(&mut writer, &link, &columns)?;
                    rows_exported += 1;
                    Ok(())
                })?;
            }
            ExportDataType::Images => {
                queries::for_each_image(conn, &crawl_id, |img| {
                    write_csv_row(&mut writer, &img, &columns)?;
                    rows_exported += 1;
                    Ok(())
                })?;
            }
            ExportDataType::FullReport => {
                anyhow::bail!("Use HTML format for full report export");
            }
        }
        Ok(())
    })?;

    writer.flush()?;

    let path_str = file_path.to_string_lossy().to_string();
    tracing::info!(path = %path_str, rows = rows_exported, "CSV export complete");

    Ok(ExportResult {
        file_path: path_str,
        rows_exported,
    })
}

// ---------------------------------------------------------------------------
// NDJSON export
// ---------------------------------------------------------------------------

fn export_ndjson(
    request: &ExportRequest,
    db: &Database,
    app: &tauri::AppHandle,
) -> anyhow::Result<ExportResult> {
    let filename = default_filename(db, &request.crawl_id, request.data_type, "jsonl");

    let file_path = app
        .dialog()
        .file()
        .add_filter("JSON Lines", &["jsonl", "json"])
        .set_file_name(&filename)
        .set_title("Export as NDJSON")
        .blocking_save_file()
        .and_then(|fp| fp.into_path().ok())
        .ok_or_else(|| anyhow::anyhow!("Export cancelled"))?;

    let columns = request
        .columns
        .clone()
        .unwrap_or_else(|| default_columns(request.data_type));

    let file = File::create(&file_path)
        .with_context(|| format!("failed to create {}", file_path.display()))?;
    let mut writer = BufWriter::new(file);

    let mut rows_exported: u64 = 0;
    let crawl_id = request.crawl_id.clone();

    db.with_conn(|conn| {
        match request.data_type {
            ExportDataType::Pages => {
                queries::for_each_page(conn, &crawl_id, |page| {
                    write_ndjson_row(&mut writer, &page, &columns)?;
                    rows_exported += 1;
                    Ok(())
                })?;
            }
            ExportDataType::Issues => {
                queries::for_each_issue(conn, &crawl_id, |issue| {
                    write_ndjson_row(&mut writer, &issue, &columns)?;
                    rows_exported += 1;
                    Ok(())
                })?;
            }
            ExportDataType::Links => {
                queries::for_each_link(conn, &crawl_id, |link| {
                    write_ndjson_row(&mut writer, &link, &columns)?;
                    rows_exported += 1;
                    Ok(())
                })?;
            }
            ExportDataType::Images => {
                queries::for_each_image(conn, &crawl_id, |img| {
                    write_ndjson_row(&mut writer, &img, &columns)?;
                    rows_exported += 1;
                    Ok(())
                })?;
            }
            ExportDataType::FullReport => {
                anyhow::bail!("Use HTML format for full report export");
            }
        }
        Ok(())
    })?;

    let path_str = file_path.to_string_lossy().to_string();
    tracing::info!(path = %path_str, rows = rows_exported, "NDJSON export complete");

    Ok(ExportResult {
        file_path: path_str,
        rows_exported,
    })
}

// ---------------------------------------------------------------------------
// HTML report
// ---------------------------------------------------------------------------

fn export_html_report_inner(
    request: &ExportRequest,
    db: &Database,
    app: &tauri::AppHandle,
) -> anyhow::Result<ExportResult> {
    let filename = default_filename(db, &request.crawl_id, ExportDataType::FullReport, "html");

    let file_path = app
        .dialog()
        .file()
        .add_filter("HTML Report", &["html"])
        .set_file_name(&filename)
        .set_title("Export HTML Report")
        .blocking_save_file()
        .and_then(|fp| fp.into_path().ok())
        .ok_or_else(|| anyhow::anyhow!("Export cancelled"))?;

    let crawl_id = &request.crawl_id;

    let html = db.with_conn(|conn| {
        let crawl = queries::select_crawl_by_id(conn, crawl_id)?
            .ok_or_else(|| anyhow::anyhow!("Crawl not found: {crawl_id}"))?;

        let (errors, warnings, info) = queries::count_issues_by_severity(conn, crawl_id)?;
        let (s2xx, s3xx, s4xx, s5xx, _other) =
            queries::count_pages_by_status_group(conn, crawl_id)?;
        let avg_ms = queries::avg_response_time(conn, crawl_id)?;
        let top_issues = queries::select_top_issues_by_rule(conn, crawl_id, 15)?;

        let top_issues_html = build_top_issues_table(&top_issues);

        let template = include_str!("../../templates/report.html");
        let html = template
            .replace("{{START_URL}}", &html_escape(&crawl.start_url))
            .replace(
                "{{STARTED_AT}}",
                &html_escape(&crawl.started_at.unwrap_or_default()),
            )
            .replace("{{STATUS}}", &html_escape(&crawl.status))
            .replace("{{TOTAL_PAGES}}", &crawl.urls_crawled.to_string())
            .replace("{{ERRORS}}", &errors.to_string())
            .replace("{{WARNINGS}}", &warnings.to_string())
            .replace("{{INFO_COUNT}}", &info.to_string())
            .replace("{{AVG_RESPONSE_MS}}", &format_avg_ms(avg_ms))
            .replace("{{STATUS_2XX}}", &s2xx.to_string())
            .replace("{{STATUS_3XX}}", &s3xx.to_string())
            .replace("{{STATUS_4XX}}", &s4xx.to_string())
            .replace("{{STATUS_5XX}}", &s5xx.to_string())
            .replace("{{TOP_ISSUES_TABLE}}", &top_issues_html);

        Ok(html)
    })?;

    std::fs::write(&file_path, &html)
        .with_context(|| format!("failed to write report to {}", file_path.display()))?;

    let path_str = file_path.to_string_lossy().to_string();
    tracing::info!(path = %path_str, "HTML report export complete");

    Ok(ExportResult {
        file_path: path_str,
        rows_exported: 0,
    })
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn format_avg_ms(avg: Option<f64>) -> String {
    match avg {
        Some(ms) => format!("{:.0}", ms),
        None => "N/A".into(),
    }
}

fn severity_badge(severity: &str) -> String {
    let class = match severity {
        "error" => "badge-error",
        "warning" => "badge-warning",
        _ => "badge-info",
    };
    format!(r#"<span class="badge {class}">{severity}</span>"#)
}

fn build_top_issues_table(issues: &[(String, String, String, i64)]) -> String {
    if issues.is_empty() {
        return "<p>No issues found.</p>".into();
    }

    let mut html = String::from(
        r#"<table>
<thead><tr><th>Rule</th><th>Severity</th><th>Category</th><th>Count</th></tr></thead>
<tbody>
"#,
    );

    for (rule_id, severity, category, count) in issues {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            html_escape(rule_id),
            severity_badge(severity),
            html_escape(category),
            count,
        ));
    }

    html.push_str("</tbody></table>");
    html
}

// ---------------------------------------------------------------------------
// .seocrawl file management
// ---------------------------------------------------------------------------

/// Save a crawl to a standalone .seocrawl file.
///
/// Creates a new SQLite database via `create_crawl_db`, renames it to the
/// user-chosen path, then copies the crawl's data from the app DB via
/// ATTACH DATABASE. Returns the saved file path, or None if the user
/// cancelled the dialog.
#[tauri::command]
pub async fn save_crawl_file(
    crawl_id: String,
    db: State<'_, Arc<Database>>,
    app: tauri::AppHandle,
) -> Result<Option<String>, String> {
    let domain = domain_from_crawl(&db, &crawl_id);
    let filename = format!("{domain}-{crawl_id}.seocrawl");

    let file_path = app
        .dialog()
        .file()
        .add_filter("OxideSEO Crawl", &["seocrawl"])
        .set_file_name(&filename)
        .set_title("Save Crawl File")
        .blocking_save_file()
        .and_then(|fp| fp.into_path().ok());

    let file_path = match file_path {
        Some(p) => p,
        None => return Ok(None),
    };

    // create_crawl_db builds its filename as {crawl_id}.seocrawl,
    // so we create it there and rename to the user-chosen path below.
    let target = Database::create_crawl_db(
        file_path.parent().unwrap_or(std::path::Path::new(".")),
        &crawl_id,
    )
    .map_err(|e| format!("Failed to create crawl file: {e:#}"))?;

    // The file was created at dir/crawl_id.seocrawl — rename to the user's chosen name.
    let created_path = target.path.clone();
    drop(target); // Close the connection before renaming.
    if created_path != file_path {
        std::fs::rename(&created_path, &file_path)
            .map_err(|e| format!("Failed to rename crawl file: {e}"))?;
    }

    // Copy data from the app DB into the target via ATTACH.
    let copy_result = db.with_conn_mut(|conn| {
        let escaped_path = file_path.to_string_lossy().replace('\'', "''");
        conn.execute_batch(&format!("ATTACH DATABASE '{}' AS target", escaped_path))?;

        let result = (|| -> anyhow::Result<()> {
            let tx = conn.transaction()?;
            tx.execute(
                "INSERT INTO target.crawls SELECT * FROM crawls WHERE id = ?1",
                rusqlite::params![crawl_id],
            )?;
            tx.execute(
                "INSERT INTO target.pages SELECT * FROM pages WHERE crawl_id = ?1",
                rusqlite::params![crawl_id],
            )?;
            tx.execute(
                "INSERT INTO target.links SELECT * FROM links WHERE crawl_id = ?1",
                rusqlite::params![crawl_id],
            )?;
            tx.execute(
                "INSERT INTO target.issues SELECT * FROM issues WHERE crawl_id = ?1",
                rusqlite::params![crawl_id],
            )?;
            tx.commit()?;
            Ok(())
        })();

        // Always detach, regardless of success or failure.
        let _ = conn.execute_batch("DETACH DATABASE target");
        result
    });

    if let Err(e) = copy_result {
        // Clean up the orphaned file on failure.
        let _ = std::fs::remove_file(&file_path);
        return Err(format!("Failed to save crawl data: {e:#}"));
    }

    let path_str = file_path.to_string_lossy().to_string();
    tracing::info!(path = %path_str, crawl_id = %crawl_id, "Crawl file saved");
    Ok(Some(path_str))
}

/// Open a .seocrawl file and import its crawl data into the app database.
///
/// Attaches the external file, copies all crawl/page/link/issue rows into
/// the app DB, then detaches. Returns the imported crawl ID, or None if cancelled.
#[tauri::command]
pub async fn open_crawl_file(
    db: State<'_, Arc<Database>>,
    app: tauri::AppHandle,
) -> Result<Option<String>, String> {
    let file_path = app
        .dialog()
        .file()
        .add_filter("OxideSEO Crawl", &["seocrawl"])
        .set_title("Open Crawl File")
        .blocking_pick_file()
        .and_then(|fp| fp.into_path().ok());

    let file_path = match file_path {
        Some(p) => p,
        None => return Ok(None),
    };

    // Validate the file has a crawls table.
    {
        let check_conn = rusqlite::Connection::open_with_flags(
            &file_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .map_err(|e| format!("Cannot open file: {e}"))?;

        let has_crawls: bool = check_conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='crawls'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Cannot read file structure: {e}"))?;

        if !has_crawls {
            return Err("Invalid .seocrawl file: missing crawls table".into());
        }
    }

    // Import data via ATTACH.
    let crawl_id = db
        .with_conn_mut(|conn| {
            let escaped_path = file_path.to_string_lossy().replace('\'', "''");
            conn.execute_batch(&format!("ATTACH DATABASE '{}' AS source", escaped_path))?;

            let result = (|| -> anyhow::Result<String> {
                // Get the crawl ID from the source file.
                let crawl_id: String = conn
                    .query_row("SELECT id FROM source.crawls LIMIT 1", [], |row| row.get(0))
                    .context("No crawl found in .seocrawl file")?;

                // Check if this crawl already exists in the app DB.
                let exists: bool = conn
                    .query_row(
                        "SELECT COUNT(*) > 0 FROM crawls WHERE id = ?1",
                        rusqlite::params![crawl_id],
                        |row| row.get(0),
                    )
                    .context("Failed to check for existing crawl")?;

                if exists {
                    anyhow::bail!("Crawl {} is already imported", crawl_id);
                }

                let tx = conn.transaction()?;
                tx.execute(
                    "INSERT INTO crawls SELECT * FROM source.crawls WHERE id = ?1",
                    rusqlite::params![crawl_id],
                )?;
                tx.execute(
                    "INSERT INTO pages SELECT * FROM source.pages WHERE crawl_id = ?1",
                    rusqlite::params![crawl_id],
                )?;
                tx.execute(
                    "INSERT INTO links SELECT * FROM source.links WHERE crawl_id = ?1",
                    rusqlite::params![crawl_id],
                )?;
                tx.execute(
                    "INSERT INTO issues SELECT * FROM source.issues WHERE crawl_id = ?1",
                    rusqlite::params![crawl_id],
                )?;
                tx.commit()?;
                Ok(crawl_id)
            })();

            // Always detach, regardless of success or failure.
            let _ = conn.execute_batch("DETACH DATABASE source");
            result
        })
        .map_err(|e| format!("Failed to import crawl: {e:#}"))?;

    let path_str = file_path.to_string_lossy().to_string();
    tracing::info!(path = %path_str, crawl_id = %crawl_id, "Crawl file imported");
    Ok(Some(crawl_id))
}
