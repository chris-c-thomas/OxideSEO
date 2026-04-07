//! Plugin system for extending OxideSEO with custom rules, exporters,
//! and post-processors.
//!
//! Two runtimes are supported:
//! - **WASM** (sandboxed, community plugins) — uses wasmtime + WIT Component Model
//! - **Native** (trusted, first-party plugins) — uses libloading for dynamic libraries

pub mod error;
pub mod exporter;
pub mod manager;
pub mod manifest;
pub mod native_host;
pub mod post_processor;

#[cfg(feature = "plugin-wasm")]
pub mod wasm_host;
#[cfg(feature = "plugin-wasm")]
pub mod wasm_rule;
