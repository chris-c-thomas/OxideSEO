# Plugin API Reference

## WIT Interface Types

The WASM plugin interface is defined in `src-tauri/wit/oxide-seo-plugin.wit`.

### PluginParsedPage Record

```wit
record plugin-parsed-page {
    url: string,
    title: option<string>,
    meta-description: option<string>,
    meta-robots: option<string>,
    canonical: option<string>,
    viewport: option<string>,
    h1s: list<string>,
    h2s: list<string>,
    word-count: u32,
    body-text: option<string>,
    body-size: option<u32>,
    response-time-ms: option<u32>,
    links-count: u32,
    images-count: u32,
    scripts: list<string>,
    stylesheets: list<string>,
}
```

### PluginIssue Record

```wit
record plugin-issue {
    rule-id: string,
    severity: severity,
    category: string,
    message: string,
    detail-json: option<string>,
}
```

### Severity Enum

```wit
enum severity {
    error,
    warning,
    info,
}
```

### Plugin Worlds

- **seo-rule-plugin** — exports: `id()`, `name()`, `category()`, `default-severity()`, `evaluate(page) -> list<plugin-issue>`
- **seo-exporter-plugin** — exports: `format-name()`, `file-extension()`, `export-data(json) -> result<list<u8>, string>`

## Manifest Format

See `plugin-development.md` for the full TOML manifest specification.

### Capabilities

| Capability | Description | WASM | Native |
|-----------|-------------|------|--------|
| `log` | Write log messages to host tracing | Yes | N/A (full access) |
| `http_read` | Make outbound HTTP GET requests | Yes | N/A |
| `db_read` | Execute read-only SQL queries | Yes | N/A |
| `fs_read_plugin_dir` | Read files in plugin directory | Yes | N/A |

## Rust Types

### PluginManifest

Defined in `src-tauri/src/plugin/manifest.rs`. Parsed from `plugin.toml`.

### PluginInfo / PluginDetail

IPC types defined in `src-tauri/src/plugin/manager.rs` and `src/types/index.ts`. Used by the frontend Plugin Manager view.

### SeoRule Trait

The core trait that all rule plugins (WASM and native) ultimately implement:

```rust
pub trait SeoRule: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn category(&self) -> RuleCategory;
    fn default_severity(&self) -> Severity;
    fn evaluate(&self, page: &ParsedPage, ctx: &CrawlContext) -> Vec<Issue>;
}
```

## IPC Commands

| Command | Parameters | Returns |
|---------|-----------|---------|
| `list_plugins` | — | `Vec<PluginInfo>` |
| `enable_plugin` | `name: String` | `()` |
| `disable_plugin` | `name: String` | `()` |
| `get_plugin_detail` | `name: String` | `PluginDetail` |
| `reload_plugins` | — | `Vec<PluginInfo>` |
| `install_plugin_from_file` | — (file dialog) | `Option<PluginInfo>` |
| `uninstall_plugin` | `name: String` | `()` |
