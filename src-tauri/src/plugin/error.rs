//! Plugin error types.

/// Errors that can occur during plugin operations.
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("manifest parse error: {0}")]
    ManifestParse(#[from] toml::de::Error),

    #[error("incompatible app version: plugin requires {required}, app is {current}")]
    IncompatibleVersion { required: String, current: String },

    #[error("plugin name must not be empty")]
    EmptyName,

    #[error("plugin name must contain only [a-zA-Z0-9_-] characters")]
    InvalidName,

    #[error("plugin cannot specify both [wasm] and [native] sections")]
    DualRuntime,

    #[error("plugin must specify either [wasm] or [native] section")]
    NoRuntime,

    #[error("{field} must be a simple filename without path separators")]
    InvalidPath { field: String },

    #[error("plugin not found: {0}")]
    NotFound(String),

    #[error("WASM load error: {0}")]
    WasmLoad(String),

    #[error("native load error: {0}")]
    NativeLoad(String),

    #[error("plugin I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("plugin not trusted: {0}")]
    NotTrusted(String),

    #[error("storage error: {0}")]
    Storage(String),
}
