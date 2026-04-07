# Plugin Examples Walkthrough

## Example 1: JSON-LD Schema Validator (WASM Rule Plugin)

**Location:** `plugins/examples/schema-validator/`

This plugin checks crawled pages for valid JSON-LD structured data. It reports four types of issues:

- `missing_schema` — No `<script type="application/ld+json">` found
- `invalid_json` — JSON-LD block contains unparseable JSON
- `missing_context` — JSON-LD is missing the `@context` property
- `missing_type` — JSON-LD is missing the `@type` property

### How It Works

1. The host calls `evaluate(page)` with the parsed page data
2. The plugin searches for JSON-LD blocks in the page content
3. Each block is validated for JSON syntax, `@context`, and `@type`
4. Issues are returned with severity `error` (invalid JSON) or `warning` (missing properties)

### Building

```bash
cd plugins/examples/schema-validator
rustup target add wasm32-wasip2
cargo build --target wasm32-wasip2 --release
cp target/wasm32-wasip2/release/schema_validator.wasm .
```

### Installing

Copy the entire `schema-validator/` directory to `{app_data_dir}/plugins/`.

## Example 2: Markdown Report Exporter (Native Exporter Plugin)

**Location:** `plugins/examples/markdown-exporter/`

This plugin exports crawl data as a formatted Markdown report with:
- Crawl metadata (ID, URL, timestamps)
- Summary statistics table
- Issue breakdown by severity
- Top 50 issues table with rule, message, and URL

### How It Works

1. The host serializes crawl data as JSON
2. The plugin deserializes and formats as Markdown
3. The formatted bytes are returned to the host for writing

### Building

```bash
cd plugins/examples/markdown-exporter
cargo build --release
```

### Installing

Copy the directory (including `plugin.toml` and the built `.dylib`/`.so`/`.dll`) to `{app_data_dir}/plugins/markdown-exporter/`. Set `trusted = true` in `plugin.toml`.
