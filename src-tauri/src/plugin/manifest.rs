//! Plugin manifest types and parsing.
//!
//! Each plugin ships a `plugin.toml` manifest in its directory. This module
//! defines the types for parsing and validating that manifest.

use serde::{Deserialize, Serialize};

use super::error::PluginError;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default fuel limit for WASM plugins (instructions per evaluate() call).
const DEFAULT_FUEL_LIMIT: u64 = 10_000_000;

/// Default memory limit for WASM plugin instances (megabytes).
const DEFAULT_MEMORY_LIMIT_MB: u32 = 64;

// ---------------------------------------------------------------------------
// Manifest types
// ---------------------------------------------------------------------------

/// Parsed `plugin.toml` manifest.
///
/// The `wasm` and `native` fields are `Option` for TOML deserialization
/// compatibility. After `parse()`, exactly one is guaranteed to be `Some`.
/// Use `runtime()` for a typed accessor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Unique plugin name (used as directory name and DB key).
    pub name: String,
    /// Semver version string.
    pub version: semver::Version,
    /// Short description.
    pub description: String,
    /// Author name or organization.
    pub author: Option<String>,
    /// SPDX license expression.
    pub license: Option<String>,
    /// Minimum OxideSEO version required.
    pub min_app_version: semver::VersionReq,
    /// What kind of plugin this is.
    pub kind: PluginKind,
    /// Capabilities the plugin requests.
    #[serde(default)]
    pub capabilities: Vec<Capability>,
    /// WASM-specific configuration (mutually exclusive with `native`).
    pub wasm: Option<WasmConfig>,
    /// Native-specific configuration (mutually exclusive with `wasm`).
    pub native: Option<NativeConfig>,
}

/// Resolved plugin runtime — the validated form of the wasm/native options.
///
/// After `parse()` validates the manifest, this enum provides a clean
/// type-level distinction between WASM and native plugins.
#[derive(Debug, Clone)]
pub enum PluginRuntime<'a> {
    Wasm(&'a WasmConfig),
    Native(&'a NativeConfig),
}

/// The type of extension a plugin provides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginKind {
    Rule,
    Exporter,
    PostProcessor,
    UiExtension,
}

impl std::fmt::Display for PluginKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rule => f.write_str("rule"),
            Self::Exporter => f.write_str("exporter"),
            Self::PostProcessor => f.write_str("post_processor"),
            Self::UiExtension => f.write_str("ui_extension"),
        }
    }
}

/// Sandboxing capabilities a plugin can request.
///
/// WASM plugins are only granted the capabilities they declare.
/// Native plugins bypass capability checks (they have full access).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// Make outbound HTTP GET requests.
    HttpRead,
    /// Execute read-only SQL queries against the crawl database.
    DbRead,
    /// Read files within the plugin's own directory.
    FsReadPluginDir,
    /// Write log messages to the host's tracing system.
    Log,
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HttpRead => f.write_str("http_read"),
            Self::DbRead => f.write_str("db_read"),
            Self::FsReadPluginDir => f.write_str("fs_read_plugin_dir"),
            Self::Log => f.write_str("log"),
        }
    }
}

/// Configuration for WASM plugins.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmConfig {
    /// Relative path to the `.wasm` file within the plugin directory.
    pub module: String,
    /// Fuel limit (instructions per call). Defaults to 10M.
    pub fuel_limit: Option<u64>,
    /// Memory limit in megabytes. Defaults to 64.
    pub memory_limit_mb: Option<u32>,
}

impl WasmConfig {
    /// Resolved fuel limit with default fallback.
    pub fn fuel_limit(&self) -> u64 {
        self.fuel_limit.unwrap_or(DEFAULT_FUEL_LIMIT)
    }

    /// Resolved memory limit (MB) with default fallback.
    pub fn memory_limit_mb(&self) -> u32 {
        self.memory_limit_mb.unwrap_or(DEFAULT_MEMORY_LIMIT_MB)
    }
}

/// Configuration for native (dynamic library) plugins.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeConfig {
    /// Relative path to the dynamic library within the plugin directory.
    pub library: String,
    /// Must be `true` to load. Native plugins execute arbitrary code.
    #[serde(default)]
    pub trusted: bool,
}

// ---------------------------------------------------------------------------
// Parsing and validation
// ---------------------------------------------------------------------------

impl PluginManifest {
    /// Parse and validate a manifest from TOML source.
    pub fn parse(toml_content: &str) -> Result<Self, PluginError> {
        let manifest: Self = toml::from_str(toml_content)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validate the manifest against the current app version and internal consistency.
    fn validate(&self) -> Result<(), PluginError> {
        let app_version = semver::Version::parse(env!("CARGO_PKG_VERSION"))
            .expect("CARGO_PKG_VERSION is valid semver");

        if !self.min_app_version.matches(&app_version) {
            return Err(PluginError::IncompatibleVersion {
                required: self.min_app_version.to_string(),
                current: app_version.to_string(),
            });
        }

        if self.name.is_empty() {
            return Err(PluginError::EmptyName);
        }

        // Plugin name is used as a directory name and DB key -- restrict to safe characters.
        if !self
            .name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(PluginError::InvalidName);
        }

        // Validate runtime-specific paths contain no directory traversal.
        if let Some(ref wasm) = self.wasm {
            validate_relative_filename(&wasm.module, "wasm.module")?;
        }
        if let Some(ref native) = self.native {
            validate_relative_filename(&native.library, "native.library")?;
        }

        if self.wasm.is_some() && self.native.is_some() {
            return Err(PluginError::DualRuntime);
        }

        if self.wasm.is_none() && self.native.is_none() {
            return Err(PluginError::NoRuntime);
        }

        Ok(())
    }

    /// Get the validated plugin runtime.
    ///
    /// Panics if called on an unvalidated manifest (always safe after `parse()`).
    pub fn runtime(&self) -> PluginRuntime<'_> {
        if let Some(ref wasm) = self.wasm {
            PluginRuntime::Wasm(wasm)
        } else if let Some(ref native) = self.native {
            PluginRuntime::Native(native)
        } else {
            unreachable!("PluginManifest::validate() ensures exactly one runtime is set")
        }
    }

    /// Whether this is a WASM plugin.
    pub fn is_wasm(&self) -> bool {
        self.wasm.is_some()
    }

    /// Whether this is a native plugin.
    pub fn is_native(&self) -> bool {
        self.native.is_some()
    }
}

/// Validate that a path is a simple filename (no directory traversal).
fn validate_relative_filename(path: &str, field_name: &str) -> Result<(), PluginError> {
    if path.is_empty()
        || path.contains('/')
        || path.contains('\\')
        || path.contains("..")
        || path.starts_with('.')
    {
        return Err(PluginError::InvalidPath {
            field: field_name.to_string(),
        });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_WASM_MANIFEST: &str = r#"
name = "test-plugin"
version = "0.1.0"
description = "A test plugin"
author = "Test Author"
license = "MIT"
min_app_version = ">=0.1.0"
kind = "rule"
capabilities = ["log"]

[wasm]
module = "test_plugin.wasm"
fuel_limit = 5000000
memory_limit_mb = 32
"#;

    const VALID_NATIVE_MANIFEST: &str = r#"
name = "native-test"
version = "1.0.0"
description = "A native test plugin"
min_app_version = ">=0.1.0"
kind = "exporter"

[native]
library = "libnative_test.dylib"
trusted = true
"#;

    #[test]
    fn test_parse_valid_wasm_manifest() {
        let manifest = PluginManifest::parse(VALID_WASM_MANIFEST).unwrap();
        assert_eq!(manifest.name, "test-plugin");
        assert_eq!(manifest.version, semver::Version::new(0, 1, 0));
        assert_eq!(manifest.description, "A test plugin");
        assert_eq!(manifest.author.as_deref(), Some("Test Author"));
        assert_eq!(manifest.license.as_deref(), Some("MIT"));
        assert_eq!(manifest.kind, PluginKind::Rule);
        assert_eq!(manifest.capabilities, vec![Capability::Log]);
        assert!(manifest.is_wasm());
        assert!(!manifest.is_native());

        assert!(
            matches!(manifest.runtime(), PluginRuntime::Wasm(w) if w.module == "test_plugin.wasm")
        );

        let wasm = manifest.wasm.unwrap();
        assert_eq!(wasm.fuel_limit(), 5_000_000);
        assert_eq!(wasm.memory_limit_mb(), 32);
    }

    #[test]
    fn test_parse_valid_native_manifest() {
        let manifest = PluginManifest::parse(VALID_NATIVE_MANIFEST).unwrap();
        assert_eq!(manifest.name, "native-test");
        assert_eq!(manifest.kind, PluginKind::Exporter);
        assert!(manifest.is_native());
        assert!(!manifest.is_wasm());

        assert!(matches!(manifest.runtime(), PluginRuntime::Native(n) if n.trusted));

        let native = manifest.native.unwrap();
        assert_eq!(native.library, "libnative_test.dylib");
        assert!(native.trusted);
    }

    #[test]
    fn test_wasm_config_defaults() {
        let toml_content = r#"
name = "defaults"
version = "0.1.0"
description = "test"
min_app_version = ">=0.1.0"
kind = "rule"

[wasm]
module = "plugin.wasm"
"#;
        let manifest = PluginManifest::parse(toml_content).unwrap();
        let wasm = manifest.wasm.unwrap();
        assert_eq!(wasm.fuel_limit(), DEFAULT_FUEL_LIMIT);
        assert_eq!(wasm.memory_limit_mb(), DEFAULT_MEMORY_LIMIT_MB);
    }

    #[test]
    fn test_missing_required_field() {
        let bad_toml = r#"
name = "incomplete"
version = "0.1.0"
"#;
        let err = PluginManifest::parse(bad_toml);
        assert!(err.is_err());
        assert!(
            matches!(err.unwrap_err(), PluginError::ManifestParse(_)),
            "expected ManifestParse error"
        );
    }

    #[test]
    fn test_bad_semver_version() {
        let bad_toml = r#"
name = "bad-version"
version = "not.a.version"
description = "test"
min_app_version = ">=0.1.0"
kind = "rule"

[wasm]
module = "plugin.wasm"
"#;
        assert!(PluginManifest::parse(bad_toml).is_err());
    }

    #[test]
    fn test_incompatible_app_version() {
        let toml_content = r#"
name = "future-plugin"
version = "0.1.0"
description = "test"
min_app_version = ">=99.0.0"
kind = "rule"

[wasm]
module = "plugin.wasm"
"#;
        let err = PluginManifest::parse(toml_content).unwrap_err();
        assert!(
            matches!(err, PluginError::IncompatibleVersion { .. }),
            "expected IncompatibleVersion, got {err:?}"
        );
    }

    #[test]
    fn test_both_wasm_and_native_rejected() {
        let toml_content = r#"
name = "both"
version = "0.1.0"
description = "test"
min_app_version = ">=0.1.0"
kind = "rule"

[wasm]
module = "plugin.wasm"

[native]
library = "libplugin.dylib"
trusted = true
"#;
        let err = PluginManifest::parse(toml_content).unwrap_err();
        assert!(matches!(err, PluginError::DualRuntime));
    }

    #[test]
    fn test_neither_wasm_nor_native_rejected() {
        let toml_content = r#"
name = "neither"
version = "0.1.0"
description = "test"
min_app_version = ">=0.1.0"
kind = "rule"
"#;
        let err = PluginManifest::parse(toml_content).unwrap_err();
        assert!(matches!(err, PluginError::NoRuntime));
    }

    #[test]
    fn test_empty_name_rejected() {
        let toml_content = r#"
name = ""
version = "0.1.0"
description = "test"
min_app_version = ">=0.1.0"
kind = "rule"

[wasm]
module = "plugin.wasm"
"#;
        let err = PluginManifest::parse(toml_content).unwrap_err();
        assert!(matches!(err, PluginError::EmptyName));
    }

    #[test]
    fn test_empty_capabilities_default() {
        let toml_content = r#"
name = "no-caps"
version = "0.1.0"
description = "test"
min_app_version = ">=0.1.0"
kind = "rule"

[wasm]
module = "plugin.wasm"
"#;
        let manifest = PluginManifest::parse(toml_content).unwrap();
        assert!(manifest.capabilities.is_empty());
    }

    #[test]
    fn test_all_plugin_kinds() {
        for (kind_str, expected) in [
            ("rule", PluginKind::Rule),
            ("exporter", PluginKind::Exporter),
            ("post_processor", PluginKind::PostProcessor),
            ("ui_extension", PluginKind::UiExtension),
        ] {
            let toml_content = format!(
                r#"
name = "kind-test"
version = "0.1.0"
description = "test"
min_app_version = ">=0.1.0"
kind = "{kind_str}"

[wasm]
module = "plugin.wasm"
"#
            );
            let manifest = PluginManifest::parse(&toml_content).unwrap();
            assert_eq!(manifest.kind, expected);
        }
    }

    #[test]
    fn test_all_capabilities() {
        let toml_content = r#"
name = "all-caps"
version = "0.1.0"
description = "test"
min_app_version = ">=0.1.0"
kind = "rule"
capabilities = ["http_read", "db_read", "fs_read_plugin_dir", "log"]

[wasm]
module = "plugin.wasm"
"#;
        let manifest = PluginManifest::parse(toml_content).unwrap();
        assert_eq!(manifest.capabilities.len(), 4);
        assert!(manifest.capabilities.contains(&Capability::HttpRead));
        assert!(manifest.capabilities.contains(&Capability::DbRead));
        assert!(manifest.capabilities.contains(&Capability::FsReadPluginDir));
        assert!(manifest.capabilities.contains(&Capability::Log));
    }

    #[test]
    fn test_invalid_name_characters_rejected() {
        // Names with non-alphanumeric/dash/underscore characters.
        for bad_name in ["foo/bar", "foo bar", "foo.bar", "../evil"] {
            let toml_content = format!(
                r#"
name = "{bad_name}"
version = "0.1.0"
description = "test"
min_app_version = ">=0.1.0"
kind = "rule"

[wasm]
module = "plugin.wasm"
"#
            );
            let err = PluginManifest::parse(&toml_content);
            assert!(err.is_err(), "name '{bad_name}' should be rejected");
            assert!(
                matches!(err.unwrap_err(), PluginError::InvalidName),
                "name '{bad_name}' should produce InvalidName"
            );
        }
    }

    #[test]
    fn test_path_traversal_in_wasm_module_rejected() {
        let toml_content = r#"
name = "test"
version = "0.1.0"
description = "test"
min_app_version = ">=0.1.0"
kind = "rule"

[wasm]
module = "../../../etc/evil.wasm"
"#;
        let err = PluginManifest::parse(toml_content).unwrap_err();
        assert!(matches!(err, PluginError::InvalidPath { .. }));
    }

    #[test]
    fn test_path_traversal_in_native_library_rejected() {
        let toml_content = r#"
name = "test"
version = "0.1.0"
description = "test"
min_app_version = ">=0.1.0"
kind = "exporter"

[native]
library = "../../../lib/evil.dylib"
trusted = true
"#;
        let err = PluginManifest::parse(toml_content).unwrap_err();
        assert!(matches!(err, PluginError::InvalidPath { .. }));
    }
}
