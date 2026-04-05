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

The project has been scaffolded with **all module stubs, types, traits, and IPC contracts in place**. Phase 1 scaffolding is largely complete. The codebase compiles structurally but many implementations are stubbed with `TODO` comments indicating which phase they belong to.

### What Exists and Works

- Full Rust module structure: `commands/`, `crawler/`, `rules/`, `storage/`, `ai/`
- All Tauri IPC command signatures defined with typed request/response structs
- SQLite schema (`migrations/001_initial.sql`) with all core tables and indexes
- `Database` struct with init, migration runner, and connection management
- URL frontier with `BinaryHeap` priority queue, blake3 dedup, and normalization (with passing unit tests)
- `Fetcher` struct with reqwest client builder and manual redirect chain tracking
- HTML parser scaffold with lol_html primary path and scraper fallback
- `PolitenessController` with per-domain delays and semaphore-based concurrency
- `RobotsCache` scaffold for robots.txt fetch/parse/cache
- `SeoRule` trait with full contract (`id`, `name`, `category`, `default_severity`, `evaluate`, `config_schema`, `configure`)
- `RuleRegistry` with `register_builtins()`, config overlay, and `evaluate_page()`
- 20 built-in rules implemented across meta, content, links, images, performance, security (all per-page rules fully implemented; 5 cross-page rules via PostCrawlAnalyzer)
- `LlmProvider` async trait for Phase 7
- Full React frontend shell: App, Sidebar, Dashboard, CrawlConfig, CrawlMonitor, ResultsExplorer, SettingsView
- Typed Tauri IPC wrappers in `src/lib/commands.ts`
- Zustand stores for crawl state and settings
- Zod validation schema for crawl config
- Theme hook with system preference detection and localStorage persistence
- Crawl progress event subscription hook
- CSS design tokens for light/dark mode
- App icons generated in `src-tauri/icons/` (placeholder — replace with real branding)
- shadcn/ui initialized with 12 components in `src/components/ui/` (badge, button, dialog, input, label, select, separator, sheet, table, tabs, tooltip). Add more with `npx shadcn@latest add <component>`.
- husky + lint-staged configured for pre-commit hooks (eslint + prettier on staged `.ts`/`.tsx` files)
- GitHub Actions CI for cross-platform builds

### What Needs Implementation (by Phase)

**Phase 1 — Remaining (finish first):**
- ~~Verify `cargo tauri dev` compiles and opens the webview window end-to-end~~ ✅
- ~~Resolve any dependency version conflicts in `Cargo.toml`~~ ✅
- ~~Install shadcn/ui components (run `npx shadcn@latest init` and add needed primitives)~~ ✅
- ~~Verify the SQLite database file is created in the Tauri app data directory on launch~~ ✅
- ~~Add husky + lint-staged for pre-commit hooks~~ ✅
- ~~Ensure CI workflow runs successfully~~ ✅

**Phase 2 — Core Crawl Engine:** ~~All work units implemented~~ ✅
- Full crawl pipeline: parser (lol_html + scraper), streaming fetcher with blake3, texting_robots, batched storage writer, frontier SQLite overflow, engine orchestrator with rayon parse dispatch, Tauri command wiring
- 6 integration tests (axum test server) + 21 new unit tests

**Phase 3 — SEO Rule Engine:** ~~All work units implemented~~ ✅
- Performance rules (`perf.large_page`, `perf.slow_response`) with configurable thresholds via `ParsedPage` fetch metadata
- `PostCrawlAnalyzer` with 5 cross-page rules: `meta.title_duplicate`, `meta.desc_duplicate`, `content.h1_duplicate`, `links.broken_internal`, `links.orphan_page`
- `FlushAck` storage command for synchronizing post-crawl analysis after all writes
- 8 post-crawl unit tests + 8 performance rule tests + 1 integration test (7 total integration tests)

**Phase 4 — Frontend UI (MVP gate):** ~~All work units implemented~~ ✅
- `DataTable` component using TanStack Table v8 + TanStack Virtual v3 with infinite scroll
- All 6 result commands implemented (`get_recent_crawls`, `get_crawl_results`, `get_crawl_summary`, `get_page_detail`, `get_issues`, `get_links`) with pagination, sorting, and filtering
- Column definitions for all 4 tabs (pages, issues, links, images) with color-coded badges and formatted values
- Filter toolbars per tab (URL search, status codes, content type, severity, category, link type, scope, broken status, missing alt text)
- Page Detail slide-out sheet with SEO metadata, performance stats, issues, and link tables
- Summary bar with issue counts, Dashboard with severity indicators
- Images tab reuses `getLinks` with `linkType: "img"` filter; `anchorText` = alt text

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

All Rust types use `#[serde(rename_all = "camelCase")]`. TypeScript types use camelCase natively. These must match exactly.

## File-by-File Guide

### Rust Backend (`src-tauri/src/`)

| File | Purpose | Status |
|---|---|---|
| `main.rs` | Tauri entry point, logging init, command registration | Complete |
| `lib.rs` | Module declarations, shared enums | Complete |
| `commands/crawl.rs` | Crawl lifecycle IPC handlers | Complete |
| `commands/results.rs` | Data query IPC handlers | Complete (all 6 commands with pagination, sorting, filtering) |
| `commands/settings.rs` | Settings IPC handlers | Signatures complete, bodies stubbed |
| `crawler/mod.rs` | Crawler types: FetchResult, ParsedPage, ExtractedLink | Complete |
| `crawler/engine.rs` | Crawl orchestrator | Complete |
| `crawler/frontier.rs` | URL priority queue + dedup + SQLite overflow | Complete with tests |
| `crawler/fetcher.rs` | HTTP client with redirect tracking + blake3 | Complete |
| `crawler/parser.rs` | HTML parser (lol_html + scraper) | Complete with tests |
| `crawler/politeness.rs` | Per-domain rate limiting | Complete |
| `crawler/robots.rs` | robots.txt cache (texting_robots) | Complete with tests |
| `rules/rule.rs` | SeoRule trait + Issue struct | Complete |
| `rules/engine.rs` | Rule registry + executor | Complete |
| `rules/builtin/meta.rs` | 7 meta rules | Complete with tests |
| `rules/builtin/content.rs` | 4 content rules | Complete |
| `rules/builtin/links.rs` | 3 link rules | 1 per-page complete; 2 cross-page via PostCrawlAnalyzer |
| `rules/builtin/images.rs` | 2 image rules | Complete |
| `rules/builtin/performance.rs` | 2 performance rules | Complete with tests (configurable thresholds) |
| `rules/builtin/security.rs` | 2 security rules | Complete |
| `rules/post_crawl.rs` | PostCrawlAnalyzer for cross-page rules | Complete with tests |
| `storage/db.rs` | SQLite connection + migrations | Complete with tests |
| `storage/models.rs` | Data structs + StorageCommand enum | Complete |
| `storage/queries.rs` | SQL statements + execution functions | Complete (paginated queries with dynamic filtering for pages, issues, links) |
| `storage/writer.rs` | Batched storage writer thread | Complete with tests |
| `ai/provider.rs` | LlmProvider trait | Complete (Phase 7) |

### Frontend (`src/`)

| File | Purpose | Status |
|---|---|---|
| `App.tsx` | Root component, view routing | Complete |
| `types/index.ts` | All TypeScript types matching Rust IPC | Complete |
| `lib/commands.ts` | Typed Tauri invoke wrappers | Complete |
| `lib/validation.ts` | Zod schemas for forms | Complete |
| `lib/utils.ts` | Formatting, classnames, helpers | Complete |
| `stores/crawlStore.ts` | Crawl state Zustand store | Complete |
| `stores/settingsStore.ts` | Settings Zustand store | Complete |
| `hooks/useTheme.ts` | Theme management | Complete |
| `hooks/useCrawlProgress.ts` | Tauri event subscription | Complete |
| `hooks/useServerData.ts` | Infinite-scroll data fetching with sort/filter | Complete |
| `components/layout/Sidebar.tsx` | Navigation sidebar | Complete |
| `components/layout/Dashboard.tsx` | Dashboard with recent crawls + severity indicators | Complete |
| `components/crawl/CrawlConfig.tsx` | Crawl config form | Complete |
| `components/crawl/CrawlMonitor.tsx` | Live crawl monitor | Complete |
| `components/results/ResultsExplorer.tsx` | Tabbed results view with summary bar + page detail | Complete |
| `components/results/DataTable.tsx` | Virtualized table (TanStack Table + Virtual) | Complete |
| `components/results/PagesTab.tsx` | Pages tab with filters | Complete |
| `components/results/IssuesTab.tsx` | Issues tab with filters | Complete |
| `components/results/LinksTab.tsx` | Links tab with filters | Complete |
| `components/results/ImagesTab.tsx` | Images tab (filtered links) with filters | Complete |
| `components/results/PageDetail.tsx` | Slide-out sheet with SEO metadata, issues, links | Complete |
| `components/results/columns/*.tsx` | Column definitions for each tab | Complete |
| `components/results/filters/*.tsx` | Filter toolbar components per tab | Complete |
| `components/settings/SettingsView.tsx` | Settings page | Complete |

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

## Dependencies to Note

- **lol_html v2** — Cloudflare's streaming HTML parser. Does not build a DOM. Uses element content handlers registered before parse. Cannot go back to re-read earlier content.
- **texting_robots v0.2** — RFC 9309 compliant robots.txt parser. Exposes `Robot::new()` and `robot.allowed()`.
- **rusqlite v0.32 (bundled)** — Bundles SQLite. No system SQLite dependency. WAL mode enabled.
- **blake3 v1** — Fast cryptographic hash used for URL dedup. `blake3::hash(bytes).into()` returns `[u8; 32]`.
- **TanStack Table v8** — Headless table (no DOM). Column definitions, sorting state, filter state are all managed by the library. You provide the render functions.
- **TanStack Virtual v3** — Row virtualization. Renders only ~50 DOM nodes regardless of total row count.
- **Zustand v5** — Minimal state management. Stores are plain functions. Access with `useStore((s) => s.field)`.

## Gotchas

- **Tauri v2 `Manager` trait** — `use tauri::Manager;` is required in any file that calls `.manage()`, `.path()`, `.emit()`, or other trait methods on `AppHandle`/`App`. The compiler error says "method not found" rather than "trait not in scope" — easy to miss.
- **Icons must exist before build** — `tauri::generate_context!()` panics at compile time if `src-tauri/icons/` is missing. Run `npx tauri icon app-icon.png` to generate from a source PNG.
- **`tsconfig.node.json` must set `composite: true`** — Required by the project reference in `tsconfig.json`. Use `emitDeclarationOnly: true` instead of `noEmit: true` (they conflict with `composite`).
- **Commit both lock files** — `Cargo.lock` and `package-lock.json` are checked in (this is an application, not a library).
- **ESLint v9 flat config** — This project uses `eslint.config.js` (flat config). The `--ext` flag does not work; target directories instead (`eslint src/`). Requires `typescript-eslint` and `@eslint/js` as devDependencies.
- **`cargo fix` breaks formatting** — Always run `cargo fmt --all` after `cargo fix --allow-dirty`.
- **Production build requires `@types/node`** — `tsc -b` (used by `npm run build`) needs Node type definitions for `vite.config.ts`. Install with `npm i -D @types/node`.
