# OxideSEO Coding Conventions

> Authoritative coding standards for all contributions to OxideSEO.
> This file lives at `.claude/CONVENTIONS.md` and is read by Claude Code on every session.
> For architecture invariants and project state, see `CLAUDE.md` at the project root.

---

## 1. General Principles

- **No dead code.** Delete unused imports, functions, variables, and commented-out blocks. Do not leave code "for later" — version control is the archive.
- **No duplication.** Before writing a new helper, check `src/lib/utils.ts` (frontend) or the relevant module (Rust). Extract shared logic into the lowest common ancestor module.
- **Single responsibility.** Each file, struct, component, and function does one thing. If a function exceeds ~40 lines, it likely needs decomposition.
- **Fail loudly in dev, gracefully in prod.** Use `debug_assert!` for invariants in Rust. Use Zod `.parse()` (throws) during development and `.safeParse()` at runtime boundaries.
- **Naming is documentation.** Prefer long, descriptive names over short ambiguous ones. `fetch_and_follow_redirects` over `do_fetch`. `useCrawlProgressSubscription` over `useProgress`.
- **No emoji characters** Do not use emojis in code, comments, commit messages, or documentation unless explicitly instructed to so. They can cause encoding issues and are not universally supported.

---

## 2. Rust Backend (`src-tauri/src/`)

### 2.1 Formatting and Linting

- All code must pass `cargo fmt --all` and `cargo clippy --all-targets -- -D warnings` with zero warnings.
- Do not suppress clippy lints with `#[allow(...)]` unless there is a documented justification in a comment directly above the attribute.

### 2.2 Error Handling

- **Library/internal code:** Use `anyhow::Result<T>` with `.context("descriptive message")` on every fallible call. Never use `.unwrap()` or `.expect()` outside of tests and infallible cases (e.g., known-valid regex).
- **IPC boundary (Tauri commands):** Return `Result<T, String>` as required by Tauri. Map errors at the command handler level with `.map_err(|e| format!("{e:#}"))`. Do not let `anyhow` errors leak across IPC.
- **Domain errors:** Use `thiserror` enums for errors with distinct recovery paths (e.g., `CrawlError::RobotsBlocked`, `CrawlError::MaxDepthExceeded`). Use `anyhow` for everything else.
- **Never panic in production paths.** No `unwrap()`, `expect()`, `panic!()`, or array indexing without bounds checks in non-test code. Use `.get()`, `if let`, or `match` instead.

### 2.3 Naming Conventions

| Item                     | Convention                                          | Example                                       |
| ------------------------ | --------------------------------------------------- | --------------------------------------------- |
| Crate / module           | `snake_case`                                        | `crawler::frontier`                           |
| Struct / Enum / Trait    | `PascalCase`                                        | `ParsedPage`, `SeoRule`, `RuleCategory`       |
| Function / method        | `snake_case`                                        | `normalize_url`, `evaluate_page`              |
| Constant                 | `SCREAMING_SNAKE_CASE`                              | `MAX_REDIRECT_HOPS`, `DEFAULT_CRAWL_DELAY_MS` |
| Type parameter           | Single uppercase letter or descriptive `PascalCase` | `T`, `F`, `PageData`                          |
| Boolean variables/fields | Prefix with `is_`, `has_`, `should_`, `can_`        | `is_noindex`, `has_canonical`                 |
| Builder methods          | Use `with_` prefix                                  | `with_timeout`, `with_concurrency`            |

### 2.4 Module and File Organization

- One primary public type per file. Supporting types (builders, iterators, error enums) can live in the same file if they are tightly coupled.
- Declare modules in `mod.rs` or the parent module file. Do not use `#[path = "..."]`.
- Order items within a file: `use` imports → constants → type definitions → trait impls → free functions → `#[cfg(test)] mod tests`.
- Group `use` imports in this order, separated by blank lines: (1) `std`, (2) external crates, (3) `crate::` internal imports.

```rust
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::crawler::frontier::UrlFrontier;
use crate::storage::db::Database;
```

### 2.5 Struct and Enum Design

- Derive `Debug` on all types. Derive `Clone` unless the type holds non-cloneable resources (channels, connections).
- IPC-crossing types must derive `Serialize, Deserialize` and use `#[serde(rename_all = "camelCase")]`. This is non-negotiable — the TypeScript frontend expects camelCase.
- Prefer struct fields over tuple structs unless the type is a genuine newtype wrapper.
- Use `Option<T>` for nullable fields. Never use sentinel values (`-1`, `""`) to represent absence.

### 2.6 Concurrency Rules

These rules reflect the architecture invariants in `CLAUDE.md` and must not be violated:

- **tokio** is for async I/O only (HTTP fetching, file I/O, Tauri event emission). Never do CPU-bound work on a tokio task.
- **rayon** is for CPU-bound work only (HTML parsing, rule evaluation, hashing). Never do I/O on a rayon thread.
- **SQLite writes** go through the dedicated storage writer thread via `mpsc::Sender<StorageCommand>`. Never write to SQLite from a fetch worker or parse thread.
- **SQLite reads** use `Database::with_conn()` and are safe from any thread.
- Do not hold a `MutexGuard` across an `.await` point. If you need async + mutex, use `tokio::sync::Mutex`.

### 2.7 Commenting Conventions (Rust)

- Use `///` doc comments on all public items (structs, enums, traits, functions, methods, constants). These generate `cargo doc` output.
- Use `//` inline comments only to explain **why**, not **what**. The code should be readable without inline comments.
- Use `// ---------------------------------------------------------------------------` separator bars (matching the existing codebase style) between major sections within a file (e.g., between rule implementations in `meta.rs`).
- Mark incomplete implementations with `// TODO(phase-N): description` where `N` is the phase number from the roadmap. Do not use bare `// TODO` without context.
- Mark known issues with `// FIXME: description` and include a brief explanation of the problem.
- Do not use `// HACK`, `// XXX`, or other informal markers.

```rust
/// Normalize a URL by removing fragments, sorting query params,
/// and lowercasing the scheme and host.
///
/// This must be called before `hash_url()` — the blake3 hash is
/// the dedup key and depends on consistent normalization.
pub fn normalize_url(raw: &str, base: &Url) -> Result<Url> {
    // Resolve relative URLs against the base before normalizing.
    let absolute = base.join(raw).context("failed to resolve relative URL")?;
    // ...
}
```

### 2.8 Testing (Rust)

- Place unit tests in the same file under `#[cfg(test)] mod tests { ... }`.
- Name test functions descriptively: `test_normalize_url_removes_fragment`, not `test_1` or `test_basic`.
- Use `#[test]` for sync tests, `#[tokio::test]` for async tests.
- Test both the happy path and at least one meaningful error/edge case per function.
- Integration tests go in `src-tauri/tests/`. Use the shared `axum` test server for crawl integration tests.
- Benchmark critical paths (HTML parsing, frontier operations) with Criterion in `src-tauri/benches/`.

### 2.9 SQL Conventions

- Write all SQL in the `storage/queries.rs` module. No inline SQL in command handlers or engine code.
- Use parameterized queries (`?1`, `?2`, ...) exclusively. Never interpolate values into SQL strings.
- Migrations live in `src-tauri/migrations/NNN_description.sql` and are registered in `MIGRATIONS` in `storage/db.rs`.
- Table and column names use `snake_case`. Index names use `idx_{table}_{column(s)}`.
- Always include `IF NOT EXISTS` on `CREATE TABLE` / `CREATE INDEX` in migrations.

---

## 3. TypeScript Frontend (`src/`)

### 3.1 Formatting and Linting

- All code must pass `npm run lint` (ESLint) and `npm run typecheck` (tsc --noEmit) with zero errors.
- Prefer `npm run format:check` (Prettier) consistency. Do not mix formatting styles.

### 3.2 File and Directory Structure

```
src/
  components/
    layout/        # App shell, Sidebar, Dashboard
    crawl/         # CrawlConfig, CrawlMonitor
    results/       # ResultsExplorer, DataTable, PageDetail
    settings/      # SettingsView
    ui/            # shadcn/ui primitives (managed by shadcn CLI)
  hooks/           # Custom React hooks (useTheme, useCrawlProgress, etc.)
  lib/             # Pure utility functions and typed IPC wrappers
  stores/          # Zustand stores
  types/           # TypeScript type definitions matching Rust IPC types
```

- Components are grouped by **feature domain**, not by component type. Do not create `buttons/`, `modals/`, `forms/` directories.
- One component per file. The filename matches the component name: `CrawlMonitor.tsx` exports `CrawlMonitor`.
- Co-locate component-specific hooks, types, and constants in the same file if they are small. Extract to separate files only when shared or exceeding ~20 lines.

### 3.3 Naming Conventions

| Item                 | Convention                                                | Example                                             |
| -------------------- | --------------------------------------------------------- | --------------------------------------------------- |
| Component            | `PascalCase` file and export                              | `CrawlMonitor.tsx` → `export function CrawlMonitor` |
| Hook                 | `camelCase` with `use` prefix                             | `useCrawlProgress`, `useTheme`                      |
| Store                | `camelCase` with `use` prefix                             | `useCrawlStore`, `useSettingsStore`                 |
| Utility function     | `camelCase`                                               | `formatDuration`, `extractDomain`                   |
| Type / Interface     | `PascalCase`                                              | `PageRow`, `CrawlConfig`, `PaginatedResponse`       |
| Constant             | `SCREAMING_SNAKE_CASE` or `PascalCase` for arrays/objects | `MAX_RESULTS`, `NAV_ITEMS`                          |
| CSS class (Tailwind) | utility-first, no custom class names                      | `className="flex items-center gap-2"`               |
| Event handler props  | `on` + event name                                         | `onCrawlStart`, `onFilterChange`                    |
| Boolean props/vars   | `is`, `has`, `should`, `can` prefix                       | `isLoading`, `hasResults`                           |

### 3.4 Component Patterns

- Use function declarations, not arrow function assignments, for components: `export function CrawlMonitor()` not `export const CrawlMonitor = () =>`.
- Prefer named exports over default exports. The only default export should be in entry files if required by the bundler.
- Destructure props in the function signature. Define a `Props` interface (not `type`) for components with more than two props.
- Keep components under 150 lines. Extract sub-components, hooks, or utility functions when approaching this limit.
- Do not use `React.FC` or `React.FunctionComponent`. Use explicit return types only when the return type is non-obvious.

```tsx
interface CrawlMonitorProps {
  crawlId: string;
  onComplete?: () => void;
}

export function CrawlMonitor({ crawlId, onComplete }: CrawlMonitorProps) {
  // ...
}
```

### 3.5 State Management

- **Zustand stores** hold cross-component and persisted state. Define them in `stores/` with typed interfaces.
- **React state** (`useState`) is for local UI state only — toggle visibility, form input values, hover states.
- **Never duplicate** Zustand store data into local component state. Select from the store with `useStore((s) => s.field)` using selectors for render optimization.
- Do not put derived/computed values in the store. Compute them in the component or in a `useMemo`.

### 3.6 IPC and Data Fetching

- All Tauri `invoke()` calls go through the typed wrappers in `lib/commands.ts`. Never call `invoke()` directly from a component.
- IPC wrapper functions must match the Rust command name exactly (with `snake_case` → `camelCase` conversion handled by the wrapper).
- Handle IPC errors at the call site with try/catch. Display user-facing errors via toast or inline error state, never `console.error` alone.
- Do not fire IPC calls in render. Use `useEffect` or event handlers.

### 3.7 Styling

- Use Tailwind utility classes exclusively. Do not write custom CSS except in `index.css` for CSS custom properties (design tokens).
- Use CSS custom properties (`var(--color-*)`) for all theme-aware colors. Reference the tokens defined in `index.css`.
- Use `cn()` from `lib/utils.ts` for conditional class merging. Do not use template literals for conditional classes.
- Prefer shadcn/ui primitives over custom implementations for standard UI patterns (buttons, dialogs, dropdowns, inputs).

```tsx
// Correct
<div className={cn("flex items-center gap-2", isActive && "bg-accent")}>

// Incorrect
<div className={`flex items-center gap-2 ${isActive ? "bg-accent" : ""}`}>
```

### 3.8 Type Safety

- Enable and respect all strict `tsconfig.json` settings: `strict`, `noUnusedLocals`, `noUnusedParameters`, `noUncheckedIndexedAccess`.
- Never use `any`. Use `unknown` and narrow with type guards when dealing with untyped external data.
- Validate all external data (IPC responses, storage reads) with Zod schemas defined in `lib/validation.ts`.
- All types that cross the IPC boundary must be defined in `types/index.ts` and must match their Rust counterparts exactly. When modifying an IPC type, update both sides in the same commit.

### 3.9 Commenting Conventions (TypeScript)

- Use `/** JSDoc */` comments on exported functions, hooks, components, and type definitions. Include `@param` and `@returns` for non-obvious signatures.
- Use `//` inline comments sparingly, only to explain non-obvious logic.
- File-level `/** ... */` block comments at the top of each file describing the module's purpose (matching the existing convention in `utils.ts`, `validation.ts`, `settingsStore.ts`).

```typescript
/**
 * Crawl state Zustand store.
 *
 * Holds the active crawl status, progress snapshots, and crawl history.
 * Updated by CrawlMonitor via Tauri event subscription.
 */
```

### 3.10 Testing (Frontend)

- Use Vitest + Testing Library. Test files live alongside source files as `ComponentName.test.tsx` or in a co-located `__tests__/` directory.
- Mock Tauri IPC calls using the setup in `tests/setup.ts`. Do not make real IPC calls in tests.
- Test user-visible behavior, not implementation details. Query by role, label, or text content — never by CSS class or internal state.
- Test loading, success, and error states for any component that fetches data.

---

## 4. IPC Boundary Contract

The IPC boundary between Rust and TypeScript is the most critical integration surface. These rules are strict:

1. **All Rust IPC types** use `#[serde(rename_all = "camelCase")]`. TypeScript types use native camelCase. These must match field-for-field.
2. **Changes to IPC types require simultaneous updates** in `src-tauri/src/` (Rust structs) and `src/types/index.ts` (TypeScript interfaces). Never change one without the other.
3. **New Tauri commands** require: (a) the `#[tauri::command]` function, (b) registration in `invoke_handler![]` in `main.rs`, (c) a typed wrapper in `lib/commands.ts`, (d) types in both languages.
4. **Tauri event payloads** (e.g., `crawl://progress`) follow the same serialization rules. The event name is the contract identifier.
5. **Pagination** is always server-side. The frontend requests `offset` + `limit` and receives `PaginatedResponse<T>`. Never transfer unbounded result sets.

---

## 5. Performance Conventions

- **No allocations in hot loops.** In the crawl pipeline (fetch → parse → evaluate → store), prefer reusing buffers, pre-allocated `Vec`s, and `&str` over `String` where lifetimes permit.
- **Batch SQLite writes.** The storage writer must accumulate 100-500 records before committing a transaction. Single-row inserts in a loop are forbidden.
- **Throttle UI updates.** Crawl progress events emit at most every 250ms. Do not increase this frequency. Frontend components consuming progress should not trigger re-renders on every event — use `requestAnimationFrame` or debounce.
- **Virtualize large lists.** Any list or table that could exceed 100 items must use TanStack Virtual. Never render unbounded DOM node counts.
- **Measure before optimizing.** Use Criterion benchmarks for Rust hot paths. Use React DevTools Profiler for component re-renders. Do not optimize without data.

---

## 6. Git and Commit Conventions

- **Commit messages** follow Conventional Commits: `type(scope): description`
  - Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `perf`, `ci`
  - Scopes: `crawler`, `rules`, `storage`, `frontend`, `ipc`, `ci`
  - Example: `feat(crawler): implement streaming body read with blake3 hashing`
- **One logical change per commit.** Do not mix refactoring with feature work.
- **All commits must compile.** Both `cargo check` and `npm run typecheck` must pass on every commit.
- **Branch naming:** `feature/short-description`, `fix/short-description`, `refactor/short-description`.

---

## 7. Dependency Management

- **Rust:** Pin major versions in `Cargo.toml` (e.g., `tokio = "1"`, not `tokio = "*"`). Group dependencies with section comments (`# --- Async runtime ---`), matching the existing style.
- **Frontend:** Use exact versions in `package.json` for production dependencies. Dev dependencies can use `^` ranges.
- **Do not add dependencies without justification.** Before adding a crate or npm package, verify that the functionality cannot be achieved with existing dependencies or a small utility function.
- **Keep `Cargo.lock` and `package-lock.json` committed.** These are not gitignored.

---

## 8. Anti-Patterns to Avoid

| Do Not                                   | Do Instead                                                       |
| ---------------------------------------- | ---------------------------------------------------------------- |
| `unwrap()` / `expect()` in non-test code | `.context("message")?` or `if let` / `match`                     |
| `any` type in TypeScript                 | `unknown` + type narrowing, or define a proper type              |
| Inline SQL in command handlers           | Call functions from `storage/queries.rs`                         |
| `console.log` for error handling         | Structured error state in UI, `tracing::error!` in Rust          |
| CSS-in-JS or custom CSS classes          | Tailwind utilities + CSS custom properties                       |
| `useEffect` for derived state            | `useMemo` or compute inline                                      |
| Passing entire store to components       | Select specific fields with `useStore((s) => s.field)`           |
| Nested ternaries in JSX                  | Extract to variables or early-return patterns                    |
| `clone()` in Rust without necessity      | Use references, `&str`, `Cow<'_, str>`, or `Arc<T>`              |
| Magic numbers / strings                  | Named constants with doc comments                                |
| Catching and silently swallowing errors  | Log, surface to user, or propagate with context                  |
| Mutating function arguments              | Return new values; use builder patterns for complex construction |
