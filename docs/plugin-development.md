# Plugin Development Guide

This guide covers how to create plugins for OxideSEO. Plugins extend the application with custom SEO rules, export formats, and post-crawl processors.

## Plugin Types

| Type | Description | Runtime |
|------|-------------|---------|
| `rule` | Custom SEO rules that evaluate each crawled page | WASM or Native |
| `exporter` | Custom export formats (e.g., Markdown, PDF) | WASM or Native |
| `post_processor` | Post-crawl analysis with database access | WASM or Native |
| `ui_extension` | Frontend UI tabs (experimental) | JavaScript |

## Plugin Structure

Every plugin is a directory containing a `plugin.toml` manifest:

```
my-plugin/
├── plugin.toml          # Required: manifest
├── my_plugin.wasm       # For WASM plugins
├── libmy_plugin.dylib   # For native plugins (macOS)
└── README.md            # Recommended
```

## Manifest Format

```toml
name = "my-plugin"
version = "0.1.0"
description = "What this plugin does"
author = "Your Name"
license = "MIT"
min_app_version = ">=0.3.0"
kind = "rule"                    # rule | exporter | post_processor | ui_extension
capabilities = ["log"]           # log | http_read | db_read | fs_read_plugin_dir

# For WASM plugins:
[wasm]
module = "my_plugin.wasm"
fuel_limit = 10000000            # Optional, default: 10M instructions
memory_limit_mb = 64             # Optional, default: 64MB

# For native plugins:
[native]
library = "libmy_plugin.dylib"   # .dylib (macOS), .so (Linux), .dll (Windows)
trusted = true                   # Must be true to load
```

## WASM Plugins

WASM plugins use the WIT (WebAssembly Interface Types) Component Model. They are sandboxed and can only access capabilities declared in their manifest.

### Building a WASM Rule Plugin

1. Install the WASM target: `rustup target add wasm32-wasip2`
2. Create a new Rust crate with `crate-type = ["cdylib"]`
3. Add `wit-bindgen = "0.36"` to dependencies
4. Implement the `seo-rule-plugin` world from `src-tauri/wit/oxide-seo-plugin.wit`
5. Build: `cargo build --target wasm32-wasip2 --release`

See `plugins/examples/schema-validator/` for a complete example.

### Sandbox Limits

- **Fuel:** 10M instructions per `evaluate()` call (configurable)
- **Memory:** 64MB max per instance (configurable)
- **Capabilities:** Only declared capabilities are granted

## Native Plugins

Native plugins are dynamic libraries loaded via `libloading`. They have full system access and require explicit trust.

### Building a Native Plugin

1. Create a Rust crate with `crate-type = ["cdylib"]`
2. Export the `SeoRule` trait implementation via the C-ABI constructor
3. Build: `cargo build --release`

See `plugins/examples/markdown-exporter/` for a complete example.

### Trust Model

Native plugins must have `trusted = true` in their manifest. The UI shows a warning badge for native plugins. Only install native plugins from sources you trust.

## Installation

Copy the plugin directory to `{app_data_dir}/plugins/{plugin-name}/`. The app discovers plugins on startup and when you click "Reload" in the Plugins view.

## Plugin Issues

Plugin-generated issues use the existing issues system. Rule IDs are automatically namespaced: `plugin.{plugin-name}.{rule-id}`. For example, a rule with ID `missing_schema` from plugin `schema-validator` becomes `plugin.schema-validator.missing_schema`.
