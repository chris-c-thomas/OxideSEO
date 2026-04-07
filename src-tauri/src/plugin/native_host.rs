//! Native plugin loading via dynamic libraries.
//!
//! Native plugins are loaded via `libloading` and execute arbitrary code.
//! They require explicit trust — the `NativeConfig.trusted` flag must be
//! set to `true` in the plugin manifest.

use std::path::Path;

use super::error::PluginError;
use crate::rules::rule::SeoRule;

/// Host for native (dynamic library) plugins.
pub struct NativePluginHost;

impl NativePluginHost {
    /// Load a native rule plugin from a dynamic library.
    ///
    /// The library must export a C-ABI constructor:
    /// ```c
    /// extern "C" fn oxide_seo_create_rule() -> *mut dyn SeoRule
    /// ```
    ///
    /// # Safety
    ///
    /// This loads and executes arbitrary native code. Only call for plugins
    /// that the user has explicitly marked as trusted.
    pub unsafe fn load_rule(lib_path: &Path) -> Result<Box<dyn SeoRule>, PluginError> {
        let lib = unsafe { libloading::Library::new(lib_path) }
            .map_err(|e| PluginError::NativeLoad(format!("failed to load library: {e}")))?;

        let constructor: libloading::Symbol<unsafe extern "C" fn() -> *mut dyn SeoRule> =
            unsafe { lib.get(b"oxide_seo_create_rule") }
                .map_err(|e| PluginError::NativeLoad(format!("missing symbol: {e}")))?;

        let raw = unsafe { constructor() };
        if raw.is_null() {
            return Err(PluginError::NativeLoad("constructor returned null".into()));
        }

        // The library must remain loaded for the lifetime of the rule object.
        // Plugin lifetime matches app lifetime, so leaking is intentional.
        std::mem::forget(lib);

        Ok(unsafe { Box::from_raw(raw) })
    }
}
