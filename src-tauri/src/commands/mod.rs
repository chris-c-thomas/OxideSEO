//! Tauri IPC command handlers.
//!
//! Each submodule exposes `#[tauri::command]` functions that the frontend
//! invokes via `@tauri-apps/api/core`. All inputs are validated through
//! serde deserialization; all outputs are serialized to JSON automatically.

pub mod ai;
pub mod crawl;
pub mod export;
pub mod plugin;
pub mod results;
pub mod settings;
