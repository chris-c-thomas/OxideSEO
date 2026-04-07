//! WASM plugin host using wasmtime's Component Model.
//!
//! Loads `.wasm` components compiled against the OxideSEO plugin WIT
//! interface. Manages engine configuration, fuel metering, and memory limits.

use std::path::Path;
use std::sync::Arc;

use wasmtime::component::Component;
use wasmtime::{Config, Engine};

use super::error::PluginError;

// ---------------------------------------------------------------------------
// Host types
// ---------------------------------------------------------------------------

/// Shared WASM engine for compiling and running plugin components.
///
/// The `Engine` is expensive to create but cheap to clone (it's `Arc` internally).
/// One engine instance is shared across all WASM plugins.
pub struct WasmPluginHost {
    engine: Arc<Engine>,
}

impl WasmPluginHost {
    /// Create a new WASM plugin host with the appropriate engine configuration.
    pub fn new() -> Result<Self, PluginError> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.consume_fuel(true);

        let engine =
            Engine::new(&config).map_err(|e| PluginError::WasmLoad(format!("engine init: {e}")))?;

        Ok(Self {
            engine: Arc::new(engine),
        })
    }

    /// Get a reference to the shared engine.
    pub fn engine(&self) -> &Arc<Engine> {
        &self.engine
    }

    /// Compile a WASM component from a file on disk.
    pub fn compile_component_from_file(&self, path: &Path) -> Result<Component, PluginError> {
        Component::from_file(&self.engine, path)
            .map_err(|e| PluginError::WasmLoad(format!("compile {}: {e}", path.display())))
    }

    /// Compile a WASM component from bytes in memory.
    pub fn compile_component(&self, wasm_bytes: &[u8]) -> Result<Component, PluginError> {
        Component::from_binary(&self.engine, wasm_bytes)
            .map_err(|e| PluginError::WasmLoad(format!("compile from bytes: {e}")))
    }
}
