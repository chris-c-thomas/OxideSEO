# CLAUDE.md — OxideSEO Development Guide

## What This Project Is

OxideSEO is an open-source, cross-platform desktop application for SEO site crawling and technical auditing. It competes with Screaming Frog, Sitebulb, and Netpeak Spider. The architecture is Tauri v2 (Rust backend) + React (TypeScript frontend), dual licensed MIT / Apache 2.0.

## Quick Reference

```bash
# Dev mode (frontend + Rust backend with hot reload)
npx tauri dev

# Production build
npx tauri build

# Rust tests
cd src-tauri && cargo test

# Rust lint
cd src-tauri && cargo clippy --all-targets -- -D warnings

# Rust format check
cd src-tauri && cargo fmt --all -- --check

# Frontend tests
npm run test

# Frontend lint
npm run lint

# Frontend type check
npm run typecheck

# Frontend format
npm run format:check

# Generate app icons (requires a 1024x1024 source PNG)
npx tauri icon app-icon.png
```

## Current State

**All 8 phases and 52 deliverables are complete.** The application is feature-complete per the development plan (`.claude/plans/seo-crawler-development-plan.pdf`). Deferred features D-1 through D-7 and D-9 are also implemented. Only D-8 (crawl scheduling) was intentionally skipped — it requires OS-native scheduling and headless CLI mode with poor effort-to-value ratio. See `.claude/plans/release-tasks.md` for remaining pre-release operational tasks (E2E tests, code signing, auto-update, crash reporting).

### Deferred Features (post-plan)

Beyond the 52 plan deliverables, these features were added:
- **D-1: Crawl deletion** — Delete crawl + cascade pages/issues/links
- **D-2: Crawl re-run** — Clone config from a completed crawl into a new crawl
- **D-3: Keyboard shortcuts** — Global hotkeys for navigation and actions
- **D-4: ResourceMeter** — Real-time memory RSS gauge + throughput stats in CrawlMonitor (raw FFI on macOS, `/proc/self/status` on Linux)
- **D-5: PDF export** — A4 summary report via `printpdf` crate
- **D-6: XLSX export** — Multi-sheet Excel with severity color-coding via `rust_xlsxwriter` crate
- **D-7: SiteTreeView** — Collapsible hierarchical URL tree in results explorer
- **D-9: Crawl comparison** — Cross-crawl diff with overview, page/issue/metadata diff tabs, Dashboard compare mode
- **D-8: Crawl scheduling** — NOT implemented (requires OS-native cron/Task Scheduler + headless mode)

## Architecture Invariants

These design decisions are intentional and should not be changed:

1. **Channel-based actor model** — The crawl engine uses `mpsc` channels between orchestrator, fetch workers, parse pool, and storage writer. Do not collapse these into a single loop.

2. **Rayon for CPU-bound work** — HTML parsing and rule evaluation happen on the rayon thread pool, NOT on tokio. Tokio is for async I/O only.

3. **Dedicated storage writer thread** — All SQLite writes go through a single dedicated thread to avoid WAL contention. Reads can happen from any thread via `Database::with_conn`.

4. **Batched transactions** — The storage writer accumulates 100-500 records before flushing in a single transaction. This is critical for write throughput.

5. **Server-side data operations** — Sorting, filtering, and pagination happen in Rust/SQLite via Tauri commands. The frontend NEVER holds the full dataset in memory.

6. **URL normalization before hashing** — Every URL must pass through `normalize_url()` before `hash_url()`. The blake3 hash is the dedup key.

7. **Manual redirect tracking** — reqwest redirect policy is set to `Policy::none()`. The fetcher follows redirects manually and records each hop.

8. **Progress event throttling** — `crawl://progress` events emit at most every 250ms or 50 URLs. Do not increase this frequency.

## Key Type Contracts

These types cross the IPC boundary. Changes must be synchronized between Rust and TypeScript:

| Rust (serde) | TypeScript | File |
|---|---|---|
| `CrawlConfig` | `CrawlConfig` | `commands/crawl.rs` ↔ `types/index.ts` |
| `CrawlStatus` | `CrawlStatus` | `commands/crawl.rs` ↔ `types/index.ts` |
| `CrawlProgress` | `CrawlProgress` | `commands/crawl.rs` ↔ `types/index.ts` |
| `PageRow` | `PageRow` | `storage/models.rs` ↔ `types/index.ts` |
| `IssueRow` | `IssueRow` | `storage/models.rs` ↔ `types/index.ts` |
| `LinkRow` | `LinkRow` | `storage/models.rs` ↔ `types/index.ts` |
| `PaginatedResponse<T>` | `PaginatedResponse<T>` | `commands/results.rs` ↔ `types/index.ts` |
| `PluginInfo` | `PluginInfo` | `plugin/manager.rs` ↔ `types/index.ts` |
| `PluginDetail` | `PluginDetail` | `plugin/manager.rs` ↔ `types/index.ts` |
| `CrawlComparisonSummary` | `CrawlComparisonSummary` | `commands/results.rs` ↔ `types/index.ts` |
| `PageDiffRow` | `PageDiffRow` | `storage/models.rs` ↔ `types/index.ts` |
| `IssueDiffRow` | `IssueDiffRow` | `storage/models.rs` ↔ `types/index.ts` |
| `SiteTreeNode` | `SiteTreeNode` | `commands/results.rs` ↔ `types/index.ts` |

All Rust types use `#[serde(rename_all = "camelCase")]`. TypeScript types use camelCase natively. These must match exactly.

## File Guide

### Rust Backend (`src-tauri/src/`)

| File | Purpose |
|---|---|
| `main.rs` | Tauri entry point, logging init, command registration |
| `commands/crawl.rs` | Crawl lifecycle IPC: start, pause, resume, stop, delete, re-run |
| `commands/results.rs` | Data query IPC: pages, issues, links, site tree, crawl comparison (10+ commands) |
| `commands/export.rs` | Export: CSV, NDJSON, HTML, PDF, XLSX reports + .seocrawl save/open |
| `commands/settings.rs` | Settings persistence IPC |
| `commands/ai.rs` | 12 AI IPC commands (config, keys, analysis, batch, summary) |
| `commands/plugin.rs` | 7 plugin management IPC commands |
| `crawler/engine.rs` | Crawl orchestrator (channel-based actor model) |
| `crawler/frontier.rs` | URL priority queue + blake3 dedup + SQLite overflow |
| `crawler/fetcher.rs` | HTTP client with redirect tracking |
| `crawler/parser.rs` | HTML parser (lol_html + scraper fallback) |
| `crawler/politeness.rs` | Per-domain rate limiting |
| `crawler/robots.rs` | robots.txt cache (texting_robots) |
| `crawler/sitemap.rs` | Sitemap XML parser + discovery |
| `crawler/external_checker.rs` | External link HEAD checker |
| `crawler/js_renderer.rs` | JS rendering via hidden Tauri webviews (experimental) |
| `rules/rule.rs` | SeoRule trait + Issue struct |
| `rules/engine.rs` | Rule registry + executor |
| `rules/builtin/*.rs` | 21 built-in rules: meta (7), content (4), links (3), images (2), performance (3), security (2) |
| `rules/post_crawl.rs` | PostCrawlAnalyzer for cross-page rules (duplicates, orphans, broken links) |
| `storage/db.rs` | SQLite connection + migrations |
| `storage/models.rs` | Data structs (PageRow, IssueRow, LinkRow, diff types, StorageCommand) |
| `storage/queries.rs` | All SQL: paginated reads, comparison diffs, report aggregates |
| `storage/writer.rs` | Batched storage writer thread |
| `ai/engine.rs` | AiAnalysisEngine with caching, rate limiting, budget enforcement |
| `ai/provider.rs` | LlmProvider trait + CompletionRequest/Response |
| `ai/adapters/*.rs` | OpenAI, Anthropic, Ollama adapters |
| `ai/keystore.rs` | OS-native API key storage (keyring crate) |
| `ai/prompts.rs` | Prompt templates for all AI analysis types |
| `plugin/manager.rs` | Plugin discovery, enable/disable, install/uninstall |
| `plugin/wasm_host.rs` | WASM Component Model runtime (wasmtime) |
| `plugin/native_host.rs` | Native plugin loading (libloading) |

### Frontend (`src/`)

| File | Purpose |
|---|---|
| `App.tsx` | Root component, view routing (`AppView` union type) |
| `types/index.ts` | All TypeScript types matching Rust IPC |
| `lib/commands.ts` | Typed Tauri invoke wrappers |
| `hooks/useServerData.ts` | Infinite-scroll data fetching with sort/filter |
| `components/layout/Dashboard.tsx` | Recent crawls, compare mode, quick-start |
| `components/crawl/CrawlConfig.tsx` | Crawl config form with advanced sections |
| `components/crawl/CrawlMonitor.tsx` | Live crawl monitor + ResourceMeter |
| `components/results/ResultsExplorer.tsx` | Tabbed results: pages, issues, links, images, sitemap, external, tree, AI |
| `components/results/DataTable.tsx` | Virtualized table (TanStack Table + Virtual) |
| `components/comparison/*.tsx` | Crawl comparison: overview, page/issue/metadata diff tabs |
| `components/export/ExportDialog.tsx` | Export dialog with format/type/column selection |
| `components/plugins/PluginManagerView.tsx` | Plugin manager grid with detail sheet |

## Testing Approach

**Rust unit tests** — Run with `cargo test` from `src-tauri/`. Tests exist for:
- URL normalization and hashing (`crawler/frontier.rs`)
- Frontier dedup and priority ordering (`crawler/frontier.rs`)
- URL resolution and internal classification (`crawler/parser.rs`)
- Word counting (`crawler/parser.rs`)
- Meta rules: title missing, title length (`rules/builtin/meta.rs`)
- Database migration (`storage/db.rs`)

When adding new functionality, write tests in the same file using `#[cfg(test)] mod tests`.

**Rust integration tests** — Place in `src-tauri/tests/`. Phase 2 needs a local `axum` HTTP server serving HTML fixtures from `tests/fixtures/`. Test full crawl cycles against it.

**Frontend tests** — Run with `npm run test`. Vitest + Testing Library. The test setup in `tests/setup.ts` mocks `@tauri-apps/api/core` and `@tauri-apps/api/event`. Test components by mocking invoke responses.

## Performance Targets

| Metric | Target |
|---|---|
| Crawl throughput | >500 pages/sec (8-core) |
| HTML parse time | <1ms per 50KB page |
| Results table | 60fps with 100k rows |
| IPC latency | <1ms per invoke |
| Memory (10k pages) | <200MB RSS |
| Binary size | <20MB |
| Cold start | <2s to interactive |

## Common Tasks

### Adding a new SEO rule

1. Create a struct in the appropriate `rules/builtin/` file
2. Implement `SeoRule` trait (id, name, category, default_severity, evaluate)
3. Register it in `RuleRegistry::register_builtins()` in `rules/engine.rs`
4. Write unit tests with HTML fixture `ParsedPage` structs
5. If configurable, implement `config_schema()` and `configure()`

### Adding a new Tauri command

1. Define the command function with `#[tauri::command]` in the appropriate `commands/` file
2. Add the command to the `invoke_handler![]` macro in `main.rs`
3. Add a typed wrapper function in `src/lib/commands.ts`
4. Add the request/response types to both `src-tauri/src/` (Rust) and `src/types/index.ts` (TypeScript)

### Adding a new frontend view

1. Create the component in the appropriate `src/components/` subdirectory
2. Add the view ID to the `AppView` type in `App.tsx`
3. Add a route case in `App.tsx`'s `renderView()` switch
4. Add a nav item in `Sidebar.tsx`'s `NAV_ITEMS` array

### Adding a new SQLite migration

1. Create `src-tauri/migrations/NNN_description.sql`
2. Add the entry to the `MIGRATIONS` array in `storage/db.rs`
3. Migrations run automatically on app start and are tracked in the `_migrations` table

### Adding a new export format

1. Add variant to `ExportFormat` enum in `commands/settings.rs`
2. Add match arm in `export_data()` in `commands/export.rs`
3. Implement the export function following the CSV/NDJSON pattern: dialog → stream → write
4. Use `for_each_page`/`for_each_issue`/`for_each_link`/`for_each_image` from `queries.rs` for streaming

## Dependencies to Note

- **csv v1** — Streaming CSV writer. Used in `commands/export.rs` for CSV export with column filtering.
- **lol_html v2** — Cloudflare's streaming HTML parser. Does not build a DOM. Uses element content handlers registered before parse. Cannot go back to re-read earlier content.
- **texting_robots v0.2** — RFC 9309 compliant robots.txt parser. Exposes `Robot::new()` and `robot.allowed()`.
- **rusqlite v0.32 (bundled)** — Bundles SQLite. No system SQLite dependency. WAL mode enabled.
- **blake3 v1** — Fast cryptographic hash used for URL dedup. `blake3::hash(bytes).into()` returns `[u8; 32]`.
- **TanStack Table v8** — Headless table (no DOM). Column definitions, sorting state, filter state are all managed by the library. You provide the render functions.
- **TanStack Virtual v3** — Row virtualization. Renders only ~50 DOM nodes regardless of total row count.
- **Zustand v5** — Minimal state management. Stores are plain functions. Access with `useStore((s) => s.field)`.
- **quick-xml v0.36** — Streaming XML parser for sitemap XML. Uses `Reader::from_reader` with event-based parsing.
- **flate2 v1** — Gzip decompression for `.xml.gz` sitemaps. Uses `GzDecoder`.
- **regex v1** — URL include/exclude pattern matching and rewrite rules. Compiled once at crawl start via `CompiledPatterns`.
- **keyring v3** — OS-native credential storage for AI provider API keys. Uses macOS Keychain, Windows Credential Manager, or Linux Secret Service. Keys are never stored in plaintext files.
- **wasmtime v29** (optional, `plugin-wasm` feature) — WASM Component Model runtime for plugin sandboxing. Adds ~5-10MB to binary. Feature-gated via `plugin-wasm` in Cargo.toml.
- **libloading v0.8** — Dynamic library loading for native plugins. Loads `.dylib`/`.so`/`.dll` via C-ABI constructor.
- **toml v0.8** — TOML parser for plugin manifests (`plugin.toml`).
- **semver v1** — Semver parsing and version requirement matching for plugin compatibility checks.
- **printpdf v0.7** — PDF generation with built-in fonts. Used for PDF report export in `commands/export.rs`.
- **rust_xlsxwriter v0.82** — Excel XLSX writer with multi-sheet support and cell formatting. Used for XLSX export in `commands/export.rs`.

## Gotchas

- **Tauri v2 `Manager` trait** — `use tauri::Manager;` is required in any file that calls `.manage()`, `.path()`, `.emit()`, or other trait methods on `AppHandle`/`App`. The compiler error says "method not found" rather than "trait not in scope" — easy to miss.
- **Icons must exist before build** — `tauri::generate_context!()` panics at compile time if `src-tauri/icons/` is missing. Run `npx tauri icon app-icon.png` to generate from a source PNG.
- **`tsconfig.node.json` must set `composite: true`** — Required by the project reference in `tsconfig.json`. Use `emitDeclarationOnly: true` instead of `noEmit: true` (they conflict with `composite`).
- **Commit both lock files** — `Cargo.lock` and `package-lock.json` are checked in (this is an application, not a library).
- **ESLint v9 flat config** — This project uses `eslint.config.js` (flat config). The `--ext` flag does not work; target directories instead (`eslint src/`). Requires `typescript-eslint` and `@eslint/js` as devDependencies.
- **`cargo fix` breaks formatting** — Always run `cargo fmt --all` after `cargo fix --allow-dirty`.
- **Production build requires `@types/node`** — `tsc -b` (used by `npm run build`) needs Node type definitions for `vite.config.ts`. Install with `npm i -D @types/node`.
- **Cargo commands require `src-tauri/` CWD** — `cargo check`, `cargo test`, `cargo fmt`, `cargo clippy` must run from `src-tauri/`, not the project root. There is no workspace `Cargo.toml` at the root.
- **Run Prettier via npx** — `npx prettier --write <file>` for formatting. It is not installed globally. The pre-commit hook (husky + lint-staged) runs Prettier automatically on staged `.ts`/`.tsx` files.
- **Run `npx prettier --write` on new/modified `.tsx` files before committing** — `npm run typecheck` and `npm run lint` do not check Prettier formatting. The pre-commit hook will reject unformatted files.
- **Use `.clamp()` not `.min().max()`** — Clippy's `manual_clamp` lint rejects the `.min(max).max(min)` pattern. Use `value.clamp(min, max)`.
- **`Severity` and `RuleCategory` have `Display`/`FromStr`/`ToSql`/`FromSql`** — Use these enums directly in `IssueRow`, SQLite params, and string formatting. No manual `format!("{:?}").to_lowercase()` conversion needed.
- **Adding a field to `PageRow` touches many locations** — Update: `storage/models.rs` (struct), `queries.rs` (UPSERT_PAGE SQL + all SELECT queries + `row_to_page` mapper), `types/index.ts` (TS interface), and every `PageRow { ... }` construction including test helpers in `writer.rs`, `post_crawl.rs`, and the two non-HTML/errored page constructions in `engine.rs`.
- **`ResultsTab` type is duplicated** — `ResultsExplorer.tsx` and `ExportDialog.tsx` each define their own `ResultsTab` union type. Adding a new tab requires updating both. The ExportDialog copy is easy to miss.
- **`tauri-plugin-dialog` Rust API** — Import `use tauri_plugin_dialog::DialogExt;`. Use `app.dialog().file().add_filter(...).blocking_save_file()` which returns `Option<FilePath>`. Call `.into_path()` for `PathBuf`. `blocking_*` methods are safe from async Tauri commands (they run on tokio worker threads, not the main thread).
- **`plugin-wasm` feature flag** — wasmtime and wasmtime-wasi are behind the `plugin-wasm` Cargo feature (default-enabled). Building without WASM support: `cargo build --no-default-features`. WASM-related code is gated with `#[cfg(feature = "plugin-wasm")]`.
- **`ExportFormat` is no longer `Copy`** — Adding `Plugin(String)` removed the `Copy` derive. Use `Clone` where needed.
- **`spawn_crawl` signature change (Phase 8)** — Added `plugin_manager: Option<Arc<tokio::sync::Mutex<PluginManager>>>` as the 6th parameter. Integration tests pass `None`. Tauri commands pass `Some(pm.inner().clone())`.
- **Plugin directory must exist** — `{app_data_dir}/plugins/` is created in `main.rs` setup. Plugin discovery silently skips if the directory doesn't exist.
- **Native plugin ABI stability** — Native plugins use Rust `dyn` trait objects. Vtable layout is not stable across Rust compiler versions. Native plugins must be compiled with the same toolchain as the host.
- **Memory RSS uses raw FFI, not `mach2` crate** — `get_memory_rss()` in `crawler/engine.rs` uses manually defined `MachTaskBasicInfo` struct + `task_info()` extern on macOS, and reads `/proc/self/status` on Linux. The `mach2` crate was intentionally avoided because its type definitions didn't match what was needed. Do not refactor to use `mach2` without verifying the struct layout.
