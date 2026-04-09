# Markdown Exporter Plugin

An example OxideSEO native exporter plugin that generates Markdown reports from crawl data.

## Building

```bash
cargo build --release
```

The output library will be at:

- macOS: `target/release/libmarkdown_exporter.dylib`
- Linux: `target/release/libmarkdown_exporter.so`
- Windows: `target/release/markdown_exporter.dll`

## Installation

Copy this directory (including `plugin.toml` and the built library) to `{app_data_dir}/plugins/markdown-exporter/`.

## Security Note

This is a native plugin. Native plugins execute arbitrary code and must be explicitly trusted. Only install native plugins from sources you trust.
