# Schema Validator Plugin

An example OxideSEO WASM rule plugin that validates JSON-LD structured data on crawled pages.

## Rules

- `missing_schema` — No JSON-LD structured data found
- `invalid_json` — JSON-LD block contains invalid JSON
- `missing_context` — JSON-LD block is missing `@context` property
- `missing_type` — JSON-LD block is missing `@type` property

## Building

```bash
# Install the WASM target
rustup target add wasm32-wasip2

# Build the plugin
cargo build --target wasm32-wasip2 --release

# Copy to your plugins directory
cp target/wasm32-wasip2/release/schema_validator.wasm .
```

## Installation

Copy this directory (including `plugin.toml` and `schema_validator.wasm`) to `{app_data_dir}/plugins/schema-validator/`.
