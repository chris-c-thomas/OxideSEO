//! OxideSEO application entry point.
//!
//! Initializes the Tauri runtime, registers IPC command handlers,
//! and configures logging.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use oxide_seo_lib::commands;
use oxide_seo_lib::commands::crawl::{CrawlHandles, PluginManagerState};
use oxide_seo_lib::plugin::manager::PluginManager;
use tauri::Manager;
use tokio::sync::Mutex;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

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

            // Wrap in Arc for shared ownership across crawl engine and commands.
            let db_arc = Arc::new(db);
            app.manage(db_arc.clone());

            // Initialize empty crawl handles map.
            app.manage(CrawlHandles::default());

            // Initialize plugin manager.
            let app_data_dir = app.path().app_data_dir()?;
            let plugins_dir = app_data_dir.join("plugins");
            std::fs::create_dir_all(&plugins_dir)?;

            let mut pm = PluginManager::new(plugins_dir, db_arc);
            if let Err(e) = pm.discover() {
                tracing::warn!(error = %e, "Plugin discovery failed");
            }
            let pm_state: PluginManagerState = Arc::new(Mutex::new(pm));
            app.manage(pm_state);

            tracing::info!("Database and plugin manager initialized");
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
            commands::results::get_sitemap_report,
            commands::results::get_external_links,
            // Settings
            commands::settings::get_settings,
            commands::settings::set_settings,
            commands::settings::get_rule_config,
            commands::settings::set_rule_config,
            // Export
            commands::export::export_data,
            commands::export::save_crawl_file,
            commands::export::open_crawl_file,
            // Plugins
            commands::plugin::list_plugins,
            commands::plugin::enable_plugin,
            commands::plugin::disable_plugin,
            commands::plugin::get_plugin_detail,
            commands::plugin::reload_plugins,
            commands::plugin::install_plugin_from_file,
            commands::plugin::uninstall_plugin,
            // AI analysis
            commands::ai::get_ai_config,
            commands::ai::set_ai_config,
            commands::ai::set_api_key,
            commands::ai::delete_api_key,
            commands::ai::has_api_key,
            commands::ai::test_ai_connection,
            commands::ai::analyze_page,
            commands::ai::batch_analyze_pages,
            commands::ai::generate_crawl_summary,
            commands::ai::get_page_analyses,
            commands::ai::get_ai_usage,
            commands::ai::get_crawl_ai_summary,
            commands::ai::list_ollama_models,
            commands::ai::estimate_batch_cost,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OxideSEO");
}
