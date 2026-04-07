//! Plugin exporter trait and integration.
//!
//! Allows plugins to provide custom export formats beyond the built-in
//! CSV, NDJSON, and HTML exporters.

use std::path::Path;

use anyhow::Result;

/// Trait for plugin-provided export formats.
///
/// WASM exporters receive data as JSON and return output bytes (host writes to disk).
/// Native exporters get direct filesystem access.
pub trait PluginExporter: Send + Sync {
    /// Human-readable format name (e.g., "Markdown Report").
    fn format_name(&self) -> &str;

    /// File extension for the exported file (e.g., "md").
    fn file_extension(&self) -> &str;

    /// Export crawl data.
    ///
    /// `data_json` contains the serialized crawl data.
    /// `output_path` is where the output should be written.
    /// Returns the number of bytes written.
    fn export(&self, data_json: &str, output_path: &Path) -> Result<u64>;
}
