//! Plugin manager — discovery, lifecycle, and runtime loading.
//!
//! Scans `{app_data_dir}/plugins/*/plugin.toml` for installed plugins,
//! manages enable/disable state, and loads plugin runtimes on demand.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;

use super::error::PluginError;
use super::manifest::{Capability, PluginKind, PluginManifest, PluginRuntime};
use crate::rules::rule::SeoRule;
use crate::storage::db::Database;
use crate::storage::queries;

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

/// A discovered plugin with its parsed manifest and runtime state.
#[derive(Debug)]
struct LoadedPlugin {
    manifest: PluginManifest,
    dir: PathBuf,
    enabled: bool,
}

// ---------------------------------------------------------------------------
// Plugin info types for IPC
// ---------------------------------------------------------------------------

/// Summary info about a plugin (for list views).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub kind: PluginKind,
    pub enabled: bool,
    pub is_native: bool,
    /// Last load error, if any. Populated when `load_rules()` fails for this plugin.
    pub load_error: Option<String>,
}

/// Detailed info about a plugin (for detail sheets).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginDetail {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub license: Option<String>,
    pub kind: PluginKind,
    pub capabilities: Vec<Capability>,
    pub enabled: bool,
    pub is_native: bool,
    pub config: Option<serde_json::Value>,
    pub installed_at: String,
}

// ---------------------------------------------------------------------------
// PluginManager
// ---------------------------------------------------------------------------

/// Manages plugin discovery, state persistence, and runtime loading.
///
/// Wrapped in `Arc<tokio::sync::Mutex<>>` and stored as Tauri managed state.
pub struct PluginManager {
    plugins_dir: PathBuf,
    plugins: HashMap<String, LoadedPlugin>,
    db: Arc<Database>,
    /// Last load error per plugin, populated by `load_rules()`.
    load_errors: HashMap<String, String>,
    #[cfg(feature = "plugin-wasm")]
    wasm_host: Option<super::wasm_host::WasmPluginHost>,
}

impl PluginManager {
    /// Create a new plugin manager.
    ///
    /// Does not scan for plugins — call `discover()` after construction.
    pub fn new(plugins_dir: PathBuf, db: Arc<Database>) -> Self {
        #[cfg(feature = "plugin-wasm")]
        let wasm_host = super::wasm_host::WasmPluginHost::new()
            .map_err(|e| tracing::warn!("Failed to init WASM host: {e}"))
            .ok();

        Self {
            plugins_dir,
            plugins: HashMap::new(),
            db,
            load_errors: HashMap::new(),
            #[cfg(feature = "plugin-wasm")]
            wasm_host,
        }
    }

    /// Scan the plugins directory for installed plugins.
    ///
    /// Parses `plugin.toml` manifests, checks version compatibility, and
    /// loads enable/disable state from the database.
    pub fn discover(&mut self) -> anyhow::Result<()> {
        self.plugins.clear();

        if !self.plugins_dir.exists() {
            tracing::debug!(dir = %self.plugins_dir.display(), "Plugins directory does not exist");
            return Ok(());
        }

        let entries = std::fs::read_dir(&self.plugins_dir)
            .with_context(|| format!("reading plugins dir: {}", self.plugins_dir.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("plugin.toml");
            if !manifest_path.exists() {
                continue;
            }

            match self.load_plugin_from_dir(&path) {
                Ok(name) => {
                    tracing::info!(plugin = %name, "Discovered plugin");
                }
                Err(e) => {
                    tracing::warn!(
                        dir = %path.display(),
                        error = %e,
                        "Failed to load plugin"
                    );
                }
            }
        }

        tracing::info!(count = self.plugins.len(), "Plugin discovery complete");
        Ok(())
    }

    /// Load a single plugin from its directory.
    fn load_plugin_from_dir(&mut self, dir: &Path) -> anyhow::Result<String> {
        let manifest_path = dir.join("plugin.toml");
        let toml_content = std::fs::read_to_string(&manifest_path)
            .with_context(|| format!("reading {}", manifest_path.display()))?;

        let manifest = PluginManifest::parse(&toml_content)?;
        let name = manifest.name.clone();

        // Load enable/disable state from DB (defaults to disabled for new plugins).
        let enabled = self
            .db
            .with_conn(|conn| {
                if let Some(row) = queries::select_plugin(conn, &name)? {
                    Ok(row.enabled)
                } else {
                    // Register new plugin in DB as disabled.
                    queries::upsert_plugin(
                        conn,
                        &name,
                        &manifest.version.to_string(),
                        &manifest.kind.to_string(),
                        false,
                        None,
                    )?;
                    Ok(false)
                }
            })
            .with_context(|| format!("reading plugin state for {name}"))?;

        self.plugins.insert(
            name.clone(),
            LoadedPlugin {
                manifest,
                dir: dir.to_path_buf(),
                enabled,
            },
        );

        Ok(name)
    }

    /// Enable a plugin.
    pub fn enable(&mut self, name: &str) -> Result<(), PluginError> {
        let plugin = self
            .plugins
            .get_mut(name)
            .ok_or_else(|| PluginError::NotFound(name.into()))?;

        plugin.enabled = true;
        self.db
            .with_conn(|conn| queries::set_plugin_enabled(conn, name, true))
            .map_err(|e| PluginError::Storage(e.to_string()))?;

        tracing::info!(plugin = %name, "Plugin enabled");
        Ok(())
    }

    /// Disable a plugin.
    pub fn disable(&mut self, name: &str) -> Result<(), PluginError> {
        let plugin = self
            .plugins
            .get_mut(name)
            .ok_or_else(|| PluginError::NotFound(name.into()))?;

        plugin.enabled = false;
        self.db
            .with_conn(|conn| queries::set_plugin_enabled(conn, name, false))
            .map_err(|e| PluginError::Storage(e.to_string()))?;

        tracing::info!(plugin = %name, "Plugin disabled");
        Ok(())
    }

    /// Load rule adapters for all enabled Rule-kind plugins.
    ///
    /// Returns boxed `SeoRule` implementors ready for registration in
    /// the `RuleRegistry`. Errors per-plugin are logged and stored in
    /// `load_errors` so the frontend can display them via `list_plugins()`.
    pub fn load_rules(&mut self) -> Vec<Box<dyn SeoRule>> {
        self.load_errors.clear();
        let mut rules: Vec<Box<dyn SeoRule>> = Vec::new();

        // Collect names first to avoid borrow conflict with &mut self.
        let enabled_rule_plugins: Vec<String> = self
            .plugins
            .iter()
            .filter(|(_, p)| p.enabled && p.manifest.kind == PluginKind::Rule)
            .map(|(name, _)| name.clone())
            .collect();

        for name in &enabled_rule_plugins {
            let plugin = &self.plugins[name];
            match self.load_rule_for_plugin(name, plugin) {
                Ok(rule) => {
                    tracing::info!(plugin = %name, rule_id = %rule.id(), "Loaded plugin rule");
                    rules.push(rule);
                }
                Err(e) => {
                    tracing::warn!(plugin = %name, error = %e, "Failed to load plugin rule");
                    self.load_errors.insert(name.clone(), e.to_string());
                }
            }
        }

        rules
    }

    /// Load a single rule from a plugin.
    fn load_rule_for_plugin(
        &self,
        name: &str,
        plugin: &LoadedPlugin,
    ) -> Result<Box<dyn SeoRule>, PluginError> {
        match plugin.manifest.runtime() {
            PluginRuntime::Native(native_config) => {
                if !native_config.trusted {
                    return Err(PluginError::NotTrusted(name.into()));
                }

                let lib_path = plugin.dir.join(&native_config.library);
                unsafe { super::native_host::NativePluginHost::load_rule(&lib_path) }
            }
            PluginRuntime::Wasm(wasm_config) => {
                #[cfg(feature = "plugin-wasm")]
                {
                    let wasm_host = self
                        .wasm_host
                        .as_ref()
                        .ok_or_else(|| PluginError::WasmLoad("WASM host not initialized".into()))?;

                    let wasm_path = plugin.dir.join(&wasm_config.module);
                    let component = wasm_host.compile_component_from_file(&wasm_path)?;

                    // For now, use manifest metadata as rule metadata.
                    // Full WIT integration will call the component's exported
                    // id(), name(), category(), default_severity() functions.
                    let adapter =
                        super::wasm_rule::WasmRuleAdapter::new(super::wasm_rule::WasmRuleConfig {
                            engine: wasm_host.engine().clone(),
                            component,
                            plugin_name: name.to_string(),
                            id: format!("plugin.{name}"),
                            name: plugin.manifest.description.clone(),
                            category: crate::RuleCategory::Structured,
                            default_severity: crate::Severity::Warning,
                            fuel_limit: wasm_config.fuel_limit(),
                        });

                    Ok(Box::new(adapter))
                }
                #[cfg(not(feature = "plugin-wasm"))]
                {
                    let _ = wasm_config;
                    Err(PluginError::WasmLoad(
                        "WASM support not compiled (plugin-wasm feature disabled)".into(),
                    ))
                }
            }
        }
    }

    /// Get a summary list of all discovered plugins.
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        self.plugins
            .iter()
            .map(|(name, p)| PluginInfo {
                name: p.manifest.name.clone(),
                version: p.manifest.version.to_string(),
                description: p.manifest.description.clone(),
                kind: p.manifest.kind,
                enabled: p.enabled,
                is_native: p.manifest.is_native(),
                load_error: self.load_errors.get(name).cloned(),
            })
            .collect()
    }

    /// Get detailed info about a specific plugin.
    pub fn get_plugin_detail(&self, name: &str) -> Option<PluginDetail> {
        let plugin = self.plugins.get(name)?;

        let installed_at = self
            .db
            .with_conn(|conn| {
                Ok(queries::select_plugin(conn, name)?
                    .map(|r| r.installed_at)
                    .unwrap_or_default())
            })
            .unwrap_or_default();

        let config = plugin
            .manifest
            .wasm
            .as_ref()
            .map(|w| {
                serde_json::json!({
                    "module": w.module,
                    "fuel_limit": w.fuel_limit(),
                    "memory_limit_mb": w.memory_limit_mb(),
                })
            })
            .or_else(|| {
                plugin.manifest.native.as_ref().map(|n| {
                    serde_json::json!({
                        "library": n.library,
                        "trusted": n.trusted,
                    })
                })
            });

        Some(PluginDetail {
            name: plugin.manifest.name.clone(),
            version: plugin.manifest.version.to_string(),
            description: plugin.manifest.description.clone(),
            author: plugin.manifest.author.clone(),
            license: plugin.manifest.license.clone(),
            kind: plugin.manifest.kind,
            capabilities: plugin.manifest.capabilities.clone(),
            enabled: plugin.enabled,
            is_native: plugin.manifest.is_native(),
            config,
            installed_at,
        })
    }

    /// Install a plugin from a directory path (copies to plugins dir).
    pub fn install_from_path(&mut self, source_dir: &Path) -> anyhow::Result<String> {
        let manifest_path = source_dir.join("plugin.toml");
        if !manifest_path.exists() {
            anyhow::bail!("No plugin.toml found in {}", source_dir.display());
        }

        let toml_content = std::fs::read_to_string(&manifest_path)?;
        let manifest = PluginManifest::parse(&toml_content)?;
        let name = manifest.name.clone();

        let dest = self.plugins_dir.join(&name);
        if dest.exists() {
            std::fs::remove_dir_all(&dest)?;
        }

        copy_dir_recursive(source_dir, &dest)?;

        // Register in DB.
        self.db.with_conn(|conn| {
            queries::upsert_plugin(
                conn,
                &name,
                &manifest.version.to_string(),
                &manifest.kind.to_string(),
                false,
                None,
            )
        })?;

        self.plugins.insert(
            name.clone(),
            LoadedPlugin {
                manifest,
                dir: dest,
                enabled: false,
            },
        );

        tracing::info!(plugin = %name, "Plugin installed");
        Ok(name)
    }

    /// Uninstall a plugin by name.
    pub fn uninstall(&mut self, name: &str) -> Result<(), PluginError> {
        let plugin = self
            .plugins
            .remove(name)
            .ok_or_else(|| PluginError::NotFound(name.into()))?;

        // Remove from filesystem.
        if plugin.dir.exists() {
            std::fs::remove_dir_all(&plugin.dir).map_err(PluginError::Io)?;
        }

        // Remove from DB.
        self.db
            .with_conn(|conn| queries::delete_plugin(conn, name))
            .map_err(|e| PluginError::Storage(e.to_string()))?;

        tracing::info!(plugin = %name, "Plugin uninstalled");
        Ok(())
    }

    /// Get the plugins directory path.
    pub fn plugins_dir(&self) -> &Path {
        &self.plugins_dir
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Recursively copy a directory.
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        // Skip symlinks to prevent information disclosure and traversal attacks.
        if file_type.is_symlink() {
            tracing::warn!(path = %entry.path().display(), "Skipping symlink in plugin directory");
            continue;
        }

        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_db() -> Arc<Database> {
        Arc::new(Database::new_in_memory().expect("in-memory DB"))
    }

    fn write_manifest(dir: &Path, toml_content: &str) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join("plugin.toml"), toml_content).unwrap();
    }

    const TEST_WASM_MANIFEST: &str = r#"
name = "test-rule"
version = "0.1.0"
description = "A test WASM rule plugin"
min_app_version = ">=0.1.0"
kind = "rule"
capabilities = ["log"]

[wasm]
module = "test.wasm"
"#;

    const TEST_NATIVE_MANIFEST: &str = r#"
name = "test-exporter"
version = "1.0.0"
description = "A test native exporter"
min_app_version = ">=0.1.0"
kind = "exporter"

[native]
library = "libtest.dylib"
trusted = true
"#;

    #[test]
    fn test_discover_finds_plugins() {
        let tmp = TempDir::new().unwrap();
        let plugins_dir = tmp.path().join("plugins");

        write_manifest(&plugins_dir.join("test-rule"), TEST_WASM_MANIFEST);
        write_manifest(&plugins_dir.join("test-exporter"), TEST_NATIVE_MANIFEST);

        let db = create_test_db();
        let mut pm = PluginManager::new(plugins_dir, db);
        pm.discover().unwrap();

        let list = pm.list_plugins();
        assert_eq!(list.len(), 2);

        let names: Vec<&str> = list.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"test-rule"));
        assert!(names.contains(&"test-exporter"));
    }

    #[test]
    fn test_discover_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let plugins_dir = tmp.path().join("plugins");
        std::fs::create_dir_all(&plugins_dir).unwrap();

        let db = create_test_db();
        let mut pm = PluginManager::new(plugins_dir, db);
        pm.discover().unwrap();

        assert!(pm.list_plugins().is_empty());
    }

    #[test]
    fn test_discover_nonexistent_dir() {
        let tmp = TempDir::new().unwrap();
        let plugins_dir = tmp.path().join("nonexistent");

        let db = create_test_db();
        let mut pm = PluginManager::new(plugins_dir, db);
        pm.discover().unwrap();

        assert!(pm.list_plugins().is_empty());
    }

    #[test]
    fn test_discover_skips_invalid_manifest() {
        let tmp = TempDir::new().unwrap();
        let plugins_dir = tmp.path().join("plugins");

        write_manifest(&plugins_dir.join("valid"), TEST_WASM_MANIFEST);
        write_manifest(&plugins_dir.join("invalid"), "this is not valid toml [[[");

        let db = create_test_db();
        let mut pm = PluginManager::new(plugins_dir, db);
        pm.discover().unwrap();

        assert_eq!(pm.list_plugins().len(), 1);
    }

    #[test]
    fn test_enable_disable() {
        let tmp = TempDir::new().unwrap();
        let plugins_dir = tmp.path().join("plugins");
        write_manifest(&plugins_dir.join("test-rule"), TEST_WASM_MANIFEST);

        let db = create_test_db();
        let mut pm = PluginManager::new(plugins_dir, db.clone());
        pm.discover().unwrap();

        // Initially disabled.
        let list = pm.list_plugins();
        assert!(!list[0].enabled);

        // Enable.
        pm.enable("test-rule").unwrap();
        let list = pm.list_plugins();
        assert!(list.iter().find(|p| p.name == "test-rule").unwrap().enabled);

        // Verify persisted in DB.
        let stored = db
            .with_conn(|conn| queries::select_plugin(conn, "test-rule"))
            .unwrap()
            .unwrap();
        assert!(stored.enabled);

        // Disable.
        pm.disable("test-rule").unwrap();
        let list = pm.list_plugins();
        assert!(!list.iter().find(|p| p.name == "test-rule").unwrap().enabled);
    }

    #[test]
    fn test_enable_nonexistent_plugin() {
        let tmp = TempDir::new().unwrap();
        let plugins_dir = tmp.path().join("plugins");
        std::fs::create_dir_all(&plugins_dir).unwrap();

        let db = create_test_db();
        let mut pm = PluginManager::new(plugins_dir, db);
        pm.discover().unwrap();

        let result = pm.enable("nonexistent");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PluginError::NotFound(_)));
    }

    #[test]
    fn test_get_plugin_detail() {
        let tmp = TempDir::new().unwrap();
        let plugins_dir = tmp.path().join("plugins");
        write_manifest(&plugins_dir.join("test-rule"), TEST_WASM_MANIFEST);

        let db = create_test_db();
        let mut pm = PluginManager::new(plugins_dir, db);
        pm.discover().unwrap();

        let detail = pm.get_plugin_detail("test-rule").unwrap();
        assert_eq!(detail.name, "test-rule");
        assert_eq!(detail.version, "0.1.0");
        assert_eq!(detail.kind, PluginKind::Rule);
        assert!(!detail.is_native);
        assert!(!detail.enabled);
    }

    #[test]
    fn test_install_and_uninstall() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("source");
        write_manifest(&source_dir, TEST_WASM_MANIFEST);
        // Create a dummy wasm file.
        std::fs::write(source_dir.join("test.wasm"), b"fake wasm").unwrap();

        let plugins_dir = tmp.path().join("plugins");
        std::fs::create_dir_all(&plugins_dir).unwrap();

        let db = create_test_db();
        let mut pm = PluginManager::new(plugins_dir.clone(), db);

        // Install.
        let name = pm.install_from_path(&source_dir).unwrap();
        assert_eq!(name, "test-rule");
        assert_eq!(pm.list_plugins().len(), 1);
        assert!(plugins_dir.join("test-rule").join("plugin.toml").exists());
        assert!(plugins_dir.join("test-rule").join("test.wasm").exists());

        // Uninstall.
        pm.uninstall("test-rule").unwrap();
        assert!(pm.list_plugins().is_empty());
        assert!(!plugins_dir.join("test-rule").exists());
    }

    #[test]
    fn test_persistence_across_discover_calls() {
        let tmp = TempDir::new().unwrap();
        let plugins_dir = tmp.path().join("plugins");
        write_manifest(&plugins_dir.join("test-rule"), TEST_WASM_MANIFEST);

        let db = create_test_db();

        // First discover + enable.
        {
            let mut pm = PluginManager::new(plugins_dir.clone(), db.clone());
            pm.discover().unwrap();
            pm.enable("test-rule").unwrap();
        }

        // Second discover reads persisted state.
        {
            let mut pm = PluginManager::new(plugins_dir, db);
            pm.discover().unwrap();
            let list = pm.list_plugins();
            assert!(list.iter().find(|p| p.name == "test-rule").unwrap().enabled);
        }
    }
}
