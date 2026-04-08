//! Export commands: CSV, NDJSON, HTML, PDF, XLSX reports, and .seocrawl file management.

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
    AiAnalyses,
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
        ExportDataType::AiAnalyses => vec![
            "pageId",
            "analysisType",
            "provider",
            "model",
            "resultJson",
            "inputTokens",
            "outputTokens",
            "costUsd",
            "latencyMs",
            "createdAt",
        ],
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
        ExportDataType::AiAnalyses => "ai-analyses",
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
        ExportFormat::Pdf => export_pdf_report(&request, &db, &app),
        ExportFormat::Xlsx => export_xlsx(&request, &db, &app),
        ExportFormat::Plugin(ref _name) => {
            // Plugin export dispatch will be wired when PluginExporter
            // adapters are loaded via the PluginManager.
            Err(anyhow::anyhow!("Plugin export not yet implemented"))
        }
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
            ExportDataType::AiAnalyses => {
                queries::for_each_ai_analysis(conn, &crawl_id, |analysis| {
                    write_csv_row(&mut writer, &analysis, &columns)?;
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
            ExportDataType::AiAnalyses => {
                queries::for_each_ai_analysis(conn, &crawl_id, |analysis| {
                    write_ndjson_row(&mut writer, &analysis, &columns)?;
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
        let report = gather_report_data(conn, crawl_id)?;
        let top_issues_html = build_top_issues_table(&report.top_issues);

        let template = include_str!("../../templates/report.html");
        let html = template
            .replace("{{START_URL}}", &html_escape(&report.start_url))
            .replace(
                "{{STARTED_AT}}",
                &html_escape(&report.started_at.unwrap_or_default()),
            )
            .replace("{{STATUS}}", &html_escape(&report.status))
            .replace("{{TOTAL_PAGES}}", &report.total_pages.to_string())
            .replace("{{ERRORS}}", &report.errors.to_string())
            .replace("{{WARNINGS}}", &report.warnings.to_string())
            .replace("{{INFO_COUNT}}", &report.info_count.to_string())
            .replace("{{AVG_RESPONSE_MS}}", &report.avg_response_ms)
            .replace("{{STATUS_2XX}}", &report.status_2xx.to_string())
            .replace("{{STATUS_3XX}}", &report.status_3xx.to_string())
            .replace("{{STATUS_4XX}}", &report.status_4xx.to_string())
            .replace("{{STATUS_5XX}}", &report.status_5xx.to_string())
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

fn build_top_issues_table(issues: &[queries::TopIssueEntry]) -> String {
    if issues.is_empty() {
        return "<p>No issues found.</p>".into();
    }

    let mut html = String::from(
        r#"<table>
<thead><tr><th>Rule</th><th>Severity</th><th>Category</th><th>Count</th></tr></thead>
<tbody>
"#,
    );

    for issue in issues {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            html_escape(&issue.rule_id),
            severity_badge(&issue.severity),
            html_escape(&issue.category),
            issue.count,
        ));
    }

    html.push_str("</tbody></table>");
    html
}

// ---------------------------------------------------------------------------
// PDF report
// ---------------------------------------------------------------------------

fn export_pdf_report(
    request: &ExportRequest,
    db: &Database,
    app: &tauri::AppHandle,
) -> anyhow::Result<ExportResult> {
    use printpdf::*;

    let filename = default_filename(db, &request.crawl_id, ExportDataType::FullReport, "pdf");

    let file_path = app
        .dialog()
        .file()
        .add_filter("PDF Report", &["pdf"])
        .set_file_name(&filename)
        .set_title("Export PDF Report")
        .blocking_save_file()
        .and_then(|fp| fp.into_path().ok())
        .ok_or_else(|| anyhow::anyhow!("Export cancelled"))?;

    let crawl_id = &request.crawl_id;

    // Gather report data (same queries as the HTML report).
    let report = db.with_conn(|conn| gather_report_data(conn, crawl_id))?;

    let (doc, page1, layer1) =
        PdfDocument::new("SEO Audit Report", Mm(210.0), Mm(297.0), "Layer 1");
    let font_regular = doc.add_builtin_font(BuiltinFont::Helvetica)?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;

    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Helper: draw text at a position.
    let mut y = 270.0;
    let left = 20.0;

    // Title.
    current_layer.use_text("SEO Audit Report", 18.0, Mm(left), Mm(y), &font_bold);
    y -= 8.0;
    current_layer.use_text(
        report.start_url.clone(),
        10.0,
        Mm(left),
        Mm(y),
        &font_regular,
    );
    y -= 5.0;
    current_layer.use_text(
        format!(
            "Crawled: {} | Status: {}",
            report.started_at.as_deref().unwrap_or("N/A"),
            report.status
        ),
        8.0,
        Mm(left),
        Mm(y),
        &font_regular,
    );
    y -= 10.0;

    // Summary section.
    current_layer.use_text("Summary", 14.0, Mm(left), Mm(y), &font_bold);
    y -= 7.0;

    let summary_lines = [
        format!("Pages Crawled: {}", report.total_pages),
        format!("Errors: {}", report.errors),
        format!("Warnings: {}", report.warnings),
        format!("Info: {}", report.info_count),
        format!("Avg Response Time: {}ms", report.avg_response_ms),
    ];
    for line in &summary_lines {
        current_layer.use_text(line, 10.0, Mm(left + 4.0), Mm(y), &font_regular);
        y -= 5.0;
    }
    y -= 5.0;

    // Status code distribution.
    current_layer.use_text(
        "Status Code Distribution",
        14.0,
        Mm(left),
        Mm(y),
        &font_bold,
    );
    y -= 7.0;

    let status_lines = [
        format!("2xx Success: {}", report.status_2xx),
        format!("3xx Redirect: {}", report.status_3xx),
        format!("4xx Client Error: {}", report.status_4xx),
        format!("5xx Server Error: {}", report.status_5xx),
    ];
    for line in &status_lines {
        current_layer.use_text(line, 10.0, Mm(left + 4.0), Mm(y), &font_regular);
        y -= 5.0;
    }
    y -= 5.0;

    // Top issues table.
    current_layer.use_text("Top Issues", 14.0, Mm(left), Mm(y), &font_bold);
    y -= 7.0;

    if report.top_issues.is_empty() {
        current_layer.use_text(
            "No issues found.",
            10.0,
            Mm(left + 4.0),
            Mm(y),
            &font_regular,
        );
    } else {
        // Table header.
        current_layer.use_text("Rule", 8.0, Mm(left + 4.0), Mm(y), &font_bold);
        current_layer.use_text("Severity", 8.0, Mm(left + 70.0), Mm(y), &font_bold);
        current_layer.use_text("Category", 8.0, Mm(left + 100.0), Mm(y), &font_bold);
        current_layer.use_text("Count", 8.0, Mm(left + 140.0), Mm(y), &font_bold);
        y -= 5.0;

        for issue in &report.top_issues {
            if y < 20.0 {
                break; // Avoid writing off the page.
            }
            current_layer.use_text(&issue.rule_id, 8.0, Mm(left + 4.0), Mm(y), &font_regular);
            current_layer.use_text(&issue.severity, 8.0, Mm(left + 70.0), Mm(y), &font_regular);
            current_layer.use_text(&issue.category, 8.0, Mm(left + 100.0), Mm(y), &font_regular);
            current_layer.use_text(
                issue.count.to_string(),
                8.0,
                Mm(left + 140.0),
                Mm(y),
                &font_regular,
            );
            y -= 4.5;
        }
    }

    // Footer.
    let footer_y = 10.0;
    current_layer.use_text(
        "Generated by OxideSEO - Open-source SEO Crawler & Audit Platform",
        7.0,
        Mm(left),
        Mm(footer_y),
        &font_regular,
    );

    doc.save(&mut BufWriter::new(File::create(&file_path)?))?;

    let path_str = file_path.to_string_lossy().to_string();
    tracing::info!(path = %path_str, "PDF report export complete");

    Ok(ExportResult {
        file_path: path_str,
        rows_exported: 0,
    })
}

// ---------------------------------------------------------------------------
// XLSX export
// ---------------------------------------------------------------------------

fn export_xlsx(
    request: &ExportRequest,
    db: &Database,
    app: &tauri::AppHandle,
) -> anyhow::Result<ExportResult> {
    use rust_xlsxwriter::*;

    let is_report = matches!(request.data_type, ExportDataType::FullReport);
    let filename = if is_report {
        default_filename(db, &request.crawl_id, ExportDataType::FullReport, "xlsx")
    } else {
        default_filename(db, &request.crawl_id, request.data_type, "xlsx")
    };

    let file_path = app
        .dialog()
        .file()
        .add_filter("Excel Workbook", &["xlsx"])
        .set_file_name(&filename)
        .set_title("Export as Excel")
        .blocking_save_file()
        .and_then(|fp| fp.into_path().ok())
        .ok_or_else(|| anyhow::anyhow!("Export cancelled"))?;

    let mut workbook = Workbook::new();
    let crawl_id = &request.crawl_id;

    // Shared formats.
    let header_format = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0xF1F5F9))
        .set_border_bottom(FormatBorder::Thin);
    let error_format = Format::new().set_font_color(Color::RGB(0xDC2626));
    let warning_format = Format::new().set_font_color(Color::RGB(0xD97706));
    let info_format = Format::new().set_font_color(Color::RGB(0x2563EB));

    let mut total_rows: u64 = 0;

    if is_report {
        // Full report: write all four sheets.
        total_rows += xlsx_write_pages_sheet(&mut workbook, db, crawl_id, &header_format)?;
        total_rows += xlsx_write_issues_sheet(
            &mut workbook,
            db,
            crawl_id,
            &header_format,
            &error_format,
            &warning_format,
            &info_format,
        )?;
        total_rows += xlsx_write_links_sheet(&mut workbook, db, crawl_id, &header_format)?;
        total_rows += xlsx_write_images_sheet(&mut workbook, db, crawl_id, &header_format)?;
    } else {
        // Single data type.
        total_rows += match request.data_type {
            ExportDataType::Pages => {
                xlsx_write_pages_sheet(&mut workbook, db, crawl_id, &header_format)?
            }
            ExportDataType::Issues => xlsx_write_issues_sheet(
                &mut workbook,
                db,
                crawl_id,
                &header_format,
                &error_format,
                &warning_format,
                &info_format,
            )?,
            ExportDataType::Links => {
                xlsx_write_links_sheet(&mut workbook, db, crawl_id, &header_format)?
            }
            ExportDataType::Images => {
                xlsx_write_images_sheet(&mut workbook, db, crawl_id, &header_format)?
            }
            ExportDataType::AiAnalyses => {
                xlsx_write_ai_sheet(&mut workbook, db, crawl_id, &header_format)?
            }
            ExportDataType::FullReport => 0, // Handled above.
        };
    }

    workbook
        .save(&file_path)
        .with_context(|| format!("failed to write XLSX to {}", file_path.display()))?;

    let path_str = file_path.to_string_lossy().to_string();
    tracing::info!(path = %path_str, rows = total_rows, "XLSX export complete");

    Ok(ExportResult {
        file_path: path_str,
        rows_exported: total_rows,
    })
}

/// Write a header row to an XLSX worksheet.
fn xlsx_write_header(
    sheet: &mut rust_xlsxwriter::Worksheet,
    headers: &[&str],
    format: &rust_xlsxwriter::Format,
) -> anyhow::Result<()> {
    for (col, &header) in headers.iter().enumerate() {
        sheet.write_string_with_format(0, col as u16, header, format)?;
    }
    Ok(())
}

fn xlsx_write_pages_sheet(
    workbook: &mut rust_xlsxwriter::Workbook,
    db: &Database,
    crawl_id: &str,
    header_format: &rust_xlsxwriter::Format,
) -> anyhow::Result<u64> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("Pages")?;

    let headers = [
        "URL",
        "Status Code",
        "Title",
        "Meta Description",
        "H1",
        "Canonical",
        "Content Type",
        "Response Time (ms)",
        "Body Size",
        "Depth",
        "State",
        "Fetched At",
        "Error",
    ];
    xlsx_write_header(sheet, &headers, header_format)?;

    let mut row: u32 = 1;
    db.with_conn(|conn| {
        queries::for_each_page(conn, crawl_id, |page| {
            sheet.write_string(row, 0, &page.url)?;
            if let Some(sc) = page.status_code {
                sheet.write_number(row, 1, sc as f64)?;
            }
            if let Some(ref t) = page.title {
                sheet.write_string(row, 2, t)?;
            }
            if let Some(ref d) = page.meta_desc {
                sheet.write_string(row, 3, d)?;
            }
            if let Some(ref h) = page.h1 {
                sheet.write_string(row, 4, h)?;
            }
            if let Some(ref c) = page.canonical {
                sheet.write_string(row, 5, c)?;
            }
            if let Some(ref ct) = page.content_type {
                sheet.write_string(row, 6, ct)?;
            }
            if let Some(rt) = page.response_time_ms {
                sheet.write_number(row, 7, rt as f64)?;
            }
            if let Some(bs) = page.body_size {
                sheet.write_number(row, 8, bs as f64)?;
            }
            sheet.write_number(row, 9, page.depth as f64)?;
            sheet.write_string(row, 10, &page.state)?;
            if let Some(ref fa) = page.fetched_at {
                sheet.write_string(row, 11, fa)?;
            }
            if let Some(ref e) = page.error_message {
                sheet.write_string(row, 12, e)?;
            }
            row += 1;
            Ok(())
        })?;
        Ok(())
    })?;

    Ok((row - 1) as u64)
}

fn xlsx_write_issues_sheet(
    workbook: &mut rust_xlsxwriter::Workbook,
    db: &Database,
    crawl_id: &str,
    header_format: &rust_xlsxwriter::Format,
    error_format: &rust_xlsxwriter::Format,
    warning_format: &rust_xlsxwriter::Format,
    info_format: &rust_xlsxwriter::Format,
) -> anyhow::Result<u64> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("Issues")?;

    let headers = ["Rule ID", "Severity", "Category", "Message", "Page ID"];
    xlsx_write_header(sheet, &headers, header_format)?;

    let mut row: u32 = 1;
    db.with_conn(|conn| {
        queries::for_each_issue(conn, crawl_id, |issue| {
            sheet.write_string(row, 0, &issue.rule_id)?;
            let sev_str = issue.severity.to_string();
            let sev_format = match sev_str.as_str() {
                "error" => error_format,
                "warning" => warning_format,
                _ => info_format,
            };
            sheet.write_string_with_format(row, 1, &sev_str, sev_format)?;
            sheet.write_string(row, 2, issue.category.to_string())?;
            sheet.write_string(row, 3, &issue.message)?;
            sheet.write_number(row, 4, issue.page_id as f64)?;
            row += 1;
            Ok(())
        })?;
        Ok(())
    })?;

    Ok((row - 1) as u64)
}

fn xlsx_write_links_sheet(
    workbook: &mut rust_xlsxwriter::Workbook,
    db: &Database,
    crawl_id: &str,
    header_format: &rust_xlsxwriter::Format,
) -> anyhow::Result<u64> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("Links")?;

    let headers = [
        "Source Page",
        "Target URL",
        "Anchor Text",
        "Link Type",
        "Internal",
        "Nofollow",
    ];
    xlsx_write_header(sheet, &headers, header_format)?;

    let mut row: u32 = 1;
    db.with_conn(|conn| {
        queries::for_each_link(conn, crawl_id, |link| {
            sheet.write_number(row, 0, link.source_page as f64)?;
            sheet.write_string(row, 1, &link.target_url)?;
            if let Some(ref a) = link.anchor_text {
                sheet.write_string(row, 2, a)?;
            }
            sheet.write_string(row, 3, &link.link_type)?;
            sheet.write_boolean(row, 4, link.is_internal)?;
            sheet.write_boolean(row, 5, link.nofollow)?;
            row += 1;
            Ok(())
        })?;
        Ok(())
    })?;

    Ok((row - 1) as u64)
}

fn xlsx_write_images_sheet(
    workbook: &mut rust_xlsxwriter::Workbook,
    db: &Database,
    crawl_id: &str,
    header_format: &rust_xlsxwriter::Format,
) -> anyhow::Result<u64> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("Images")?;

    let headers = ["Image URL", "Source Page", "Alt Text", "Internal"];
    xlsx_write_header(sheet, &headers, header_format)?;

    let mut row: u32 = 1;
    db.with_conn(|conn| {
        queries::for_each_image(conn, crawl_id, |img| {
            sheet.write_string(row, 0, &img.target_url)?;
            sheet.write_number(row, 1, img.source_page as f64)?;
            if let Some(ref a) = img.anchor_text {
                sheet.write_string(row, 2, a)?;
            }
            sheet.write_boolean(row, 3, img.is_internal)?;
            row += 1;
            Ok(())
        })?;
        Ok(())
    })?;

    Ok((row - 1) as u64)
}

fn xlsx_write_ai_sheet(
    workbook: &mut rust_xlsxwriter::Workbook,
    db: &Database,
    crawl_id: &str,
    header_format: &rust_xlsxwriter::Format,
) -> anyhow::Result<u64> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("AI Analyses")?;

    let headers = [
        "Page ID",
        "Analysis Type",
        "Provider",
        "Model",
        "Result (JSON)",
        "Input Tokens",
        "Output Tokens",
        "Cost (USD)",
        "Latency (ms)",
        "Created At",
    ];
    xlsx_write_header(sheet, &headers, header_format)?;

    let mut row: u32 = 1;
    db.with_conn(|conn| {
        queries::for_each_ai_analysis(conn, crawl_id, |a| {
            sheet.write_number(row, 0, a.page_id as f64)?;
            sheet.write_string(row, 1, &a.analysis_type)?;
            sheet.write_string(row, 2, &a.provider)?;
            sheet.write_string(row, 3, &a.model)?;
            sheet.write_string(row, 4, &a.result_json)?;
            sheet.write_number(row, 5, a.input_tokens as f64)?;
            sheet.write_number(row, 6, a.output_tokens as f64)?;
            sheet.write_number(row, 7, a.cost_usd)?;
            sheet.write_number(row, 8, a.latency_ms as f64)?;
            sheet.write_string(row, 9, &a.created_at)?;
            row += 1;
            Ok(())
        })?;
        Ok(())
    })?;

    Ok((row - 1) as u64)
}

// ---------------------------------------------------------------------------
// Shared report data
// ---------------------------------------------------------------------------

/// Data gathered from a crawl for PDF and HTML reports.
#[derive(Debug)]
struct ReportData {
    start_url: String,
    started_at: Option<String>,
    status: String,
    total_pages: i64,
    errors: u64,
    warnings: u64,
    info_count: u64,
    avg_response_ms: String,
    status_2xx: u64,
    status_3xx: u64,
    status_4xx: u64,
    status_5xx: u64,
    top_issues: Vec<queries::TopIssueEntry>,
}

fn gather_report_data(conn: &rusqlite::Connection, crawl_id: &str) -> anyhow::Result<ReportData> {
    let crawl = queries::select_crawl_by_id(conn, crawl_id)?
        .ok_or_else(|| anyhow::anyhow!("Crawl not found: {crawl_id}"))?;
    let (errors, warnings, info) = queries::count_issues_by_severity(conn, crawl_id)?;
    let (s2xx, s3xx, s4xx, s5xx, _other) = queries::count_pages_by_status_group(conn, crawl_id)?;
    let avg_ms = queries::avg_response_time(conn, crawl_id)?;
    let top_issues = queries::select_top_issues_by_rule(conn, crawl_id, 15)?;

    Ok(ReportData {
        start_url: crawl.start_url,
        started_at: crawl.started_at,
        status: crawl.status,
        total_pages: crawl.urls_crawled,
        errors,
        warnings,
        info_count: info,
        avg_response_ms: format_avg_ms(avg_ms),
        status_2xx: s2xx,
        status_3xx: s3xx,
        status_4xx: s4xx,
        status_5xx: s5xx,
        top_issues,
    })
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
