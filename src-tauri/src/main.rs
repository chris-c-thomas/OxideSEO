//! OxideSEO application entry point.
//!
//! Initializes the Tauri runtime, registers IPC command handlers,
//! and configures logging.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use oxide_seo_lib::commands;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() {
    // Initialize structured logging.
    // Default to `info` level; override with RUST_LOG env var in dev.
    tracing_subscriber::registry()
        .with(fmt::layer().with_target(true).json())
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,oxide_seo_lib=debug")),
        )
        .init();

    tracing::info!("Starting OxideSEO v{}", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Initialize the SQLite database on first launch.
            // The storage module handles migrations automatically.
            let app_handle = app.handle().clone();
            let db = oxide_seo_lib::storage::db::Database::init(&app_handle)?;

            // Store the database handle in Tauri managed state so
            // command handlers can access it via `State<Database>`.
            app.manage(db);

            tracing::info!("Database initialized");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Crawl lifecycle
            commands::crawl::start_crawl,
            commands::crawl::pause_crawl,
            commands::crawl::resume_crawl,
            commands::crawl::stop_crawl,
            commands::crawl::get_crawl_status,
            // Results queries
            commands::results::get_recent_crawls,
            commands::results::get_crawl_results,
            commands::results::get_crawl_summary,
            commands::results::get_page_detail,
            commands::results::get_issues,
            commands::results::get_links,
            // Settings
            commands::settings::get_settings,
            commands::settings::set_settings,
            commands::settings::get_rule_config,
            commands::settings::set_rule_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OxideSEO");
}
