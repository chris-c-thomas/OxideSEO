# Development

This document covers everything needed to develop OxideSEO locally: prerequisites, setup, common workflows, and troubleshooting.

For the project overview, see [README.md](../README.md).
For architecture details, see [ARCHITECTURE.md](ARCHITECTURE.md).

## Prerequisites

| Tool    | Version               | Notes                                                              |
| ------- | --------------------- | ------------------------------------------------------------------ |
| Rust    | 1.85+ (stable)        | Install via [rustup](https://rustup.rs/)                           |
| Node.js | 22 LTS                | Install via [nodejs.org](https://nodejs.org/) or a version manager |
| npm     | Included with Node.js | Used as the package manager                                        |

### Platform-Specific Dependencies

**macOS:**

- Xcode Command Line Tools: `xcode-select --install`

**Windows:**

- [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- WebView2 (pre-installed on Windows 11; download from Microsoft for Windows 10)

**Linux (Debian/Ubuntu):**

```bash
sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

For other distributions, see the [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/).

## Initial Setup

```bash
# Clone the repository
git clone https://github.com/chris-c-thomas/OxideSEO.git
cd OxideSEO

# Install frontend dependencies
npm install

# Run in development mode (starts both Vite dev server and Rust backend)
npx tauri dev
```

The app window opens automatically. The Vite dev server runs on `http://localhost:1420` with hot module replacement. The Rust backend recompiles on source changes.

## Environment Variables

OxideSEO uses no environment variables for runtime configuration. All user-facing settings are managed through the in-app Settings view, persisted in SQLite, and API keys are stored in the OS-native credential store.

The only env var that affects the backend is the standard `RUST_LOG` variable for log verbosity:

```bash
# Override log level (default: info)
RUST_LOG=debug npx tauri dev
```

## Scripts

All npm scripts are defined in `package.json`:

| Script                      | Command                                    | Purpose                              |
| --------------------------- | ------------------------------------------ | ------------------------------------ |
| `npm run dev`               | `vite`                                     | Frontend dev server only (port 1420) |
| `npm run build`             | `tsc -b && vite build`                     | TypeScript check + production bundle |
| `npm run preview`           | `vite preview`                             | Preview production build locally     |
| `npm run lint`              | `eslint src/ --max-warnings 0`             | Lint frontend (zero-warning policy)  |
| `npm run lint:fix`          | `eslint src/ --fix`                        | Auto-fix ESLint violations           |
| `npm run format`            | `prettier --write "src/**/*.{ts,tsx,css}"` | Auto-format frontend code            |
| `npm run format:check`      | `prettier --check "src/**/*.{ts,tsx,css}"` | Check formatting without modifying   |
| `npm run test`              | `vitest run`                               | Run frontend tests (single pass)     |
| `npm run test:watch`        | `vitest`                                   | Run frontend tests in watch mode     |
| `npm run typecheck`         | `tsc --noEmit`                             | TypeScript type checking only        |
| `npm run changelog`         | `git cliff -o CHANGELOG.md`                | Generate changelog from commits      |
| `npm run changelog:preview` | `git cliff --unreleased`                   | Preview unreleased changelog         |

Rust commands run from the `src-tauri/` directory:

| Command                                     | Purpose                             |
| ------------------------------------------- | ----------------------------------- |
| `cargo test`                                | Run Rust unit and integration tests |
| `cargo clippy --all-targets -- -D warnings` | Rust lint (zero-warning policy)     |
| `cargo fmt --all -- --check`                | Check Rust formatting               |
| `cargo fmt --all`                           | Auto-format Rust code               |
| `cargo bench`                               | Run Criterion benchmarks            |

## Code Quality

All checks run in CI on every PR. Run them locally before pushing:

```bash
# Frontend
npm run lint
npm run format:check
npm run typecheck
npm run test

# Rust (from src-tauri/)
cd src-tauri
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Pre-commit hooks (husky + lint-staged) automatically check ESLint and Prettier on staged `.ts`, `.tsx`, and `.css` files.

## Testing

### Frontend Tests

- **Framework:** Vitest 3.2.4 + Testing Library
- **Environment:** jsdom
- **Setup:** `tests/setup.ts` mocks `@tauri-apps/api/core` (invoke) and `@tauri-apps/api/event` (listen/emit)

```bash
npm run test              # Single pass
npm run test:watch        # Watch mode
```

Test files live alongside source files as `*.test.tsx` or `*.test.ts`.

### Rust Tests

- **Unit tests:** `#[cfg(test)] mod tests` blocks in 17+ source files
- **Integration tests:** `src-tauri/tests/` (crawl integration, AI integration)
- **Benchmarks:** `src-tauri/benches/` (html_parsing, frontier_ops via Criterion)

```bash
cd src-tauri
cargo test                         # All unit + integration tests
cargo test --lib                   # Unit tests only
cargo test --test crawl_integration  # Specific integration test
cargo bench                        # Run benchmarks
```

### Writing Tests

- **Frontend:** Mock Tauri IPC using the setup in `tests/setup.ts`. Test user-visible behavior, not implementation. Query by role, label, or text content.
- **Rust:** Place unit tests in the same file under `#[cfg(test)] mod tests`. Name tests descriptively: `test_normalize_url_removes_fragment`, not `test_1`.

## TypeScript Configuration

The project uses strict TypeScript with these non-default options:

| Option                     | Value     | Effect                                       |
| -------------------------- | --------- | -------------------------------------------- |
| `strict`                   | `true`    | All strict checks enabled                    |
| `noUncheckedIndexedAccess` | `true`    | Array/object access returns `T \| undefined` |
| `noUnusedLocals`           | `true`    | Errors on unused local variables             |
| `noUnusedParameters`       | `true`    | Errors on unused function parameters         |
| `moduleResolution`         | `bundler` | Vite-compatible module resolution            |

Path aliases configured:

| Alias            | Target               |
| ---------------- | -------------------- |
| `@/*`            | `./src/*`            |
| `@/components/*` | `./src/components/*` |
| `@/hooks/*`      | `./src/hooks/*`      |
| `@/lib/*`        | `./src/lib/*`        |
| `@/stores/*`     | `./src/stores/*`     |
| `@/types/*`      | `./src/types/*`      |

## Common Workflows

Step-by-step instructions for common development tasks are maintained in [`CLAUDE.md`](../CLAUDE.md#common-tasks) (the authoritative source). This includes:

- Adding a new SEO rule
- Adding a new Tauri command
- Adding a new frontend view
- Adding a new SQLite migration
- Adding a new export format

### Generating App Icons

```bash
npx tauri icon app-icon.png
```

This generates all platform-specific icon formats from the 1024x1024 source PNG into `src-tauri/icons/`.

## Plugin Development

Plugins extend OxideSEO with custom rules, export formats, and post-crawl processors. See [ARCHITECTURE.md](ARCHITECTURE.md#plugin-system) for the system design.

### Plugin Structure

Every plugin is a directory containing a `plugin.toml` manifest:

```
my-plugin/
├── plugin.toml          # Required: manifest
├── my_plugin.wasm       # WASM plugins
└── libmy_plugin.dylib   # Native plugins (macOS)
```

### Manifest Format

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

### Building a WASM Rule Plugin

1. Install the WASM target: `rustup target add wasm32-wasip2`
2. Create a Rust crate with `crate-type = ["cdylib"]`
3. Add `wit-bindgen = "0.36"` to dependencies
4. Implement the `seo-rule-plugin` world from `src-tauri/wit/oxide-seo-plugin.wit`
5. Build: `cargo build --target wasm32-wasip2 --release`

See `plugins/examples/schema-validator/` for a complete example.

### Building a Native Plugin

1. Create a Rust crate with `crate-type = ["cdylib"]`
2. Export the `SeoRule` trait via the C-ABI constructor `oxide_seo_create_rule`
3. Build: `cargo build --release`

See `plugins/examples/markdown-exporter/` for a complete example.

### Plugin API Reference

#### PluginParsedPage (data passed to rule plugins)

| Field              | Type             | Description                   |
| ------------------ | ---------------- | ----------------------------- |
| `url`              | `string`         | Page URL                      |
| `title`            | `option<string>` | Page title                    |
| `meta_description` | `option<string>` | Meta description              |
| `meta_robots`      | `option<string>` | Meta robots directive         |
| `canonical`        | `option<string>` | Canonical URL                 |
| `viewport`         | `option<string>` | Viewport meta tag             |
| `h1s`              | `list<string>`   | H1 heading texts              |
| `h2s`              | `list<string>`   | H2 heading texts              |
| `word_count`       | `u32`            | Body word count               |
| `body_text`        | `option<string>` | Body text (first ~8000 chars) |
| `body_size`        | `option<u32>`    | Response body size in bytes   |
| `response_time_ms` | `option<u32>`    | Response time in milliseconds |
| `links_count`      | `u32`            | Number of links on page       |
| `images_count`     | `u32`            | Number of images on page      |
| `scripts`          | `list<string>`   | Script URLs                   |
| `stylesheets`      | `list<string>`   | Stylesheet URLs               |

#### Capabilities

| Capability           | Description                        | WASM | Native            |
| -------------------- | ---------------------------------- | ---- | ----------------- |
| `log`                | Write log messages to host tracing | Yes  | N/A (full access) |
| `http_read`          | Outbound HTTP GET requests         | Yes  | N/A               |
| `db_read`            | Read-only SQL queries              | Yes  | N/A               |
| `fs_read_plugin_dir` | Read files in plugin directory     | Yes  | N/A               |

#### API Stability

- **Stable:** `PluginParsedPage` fields (append-only), WIT world names, manifest format, issue namespace (`plugin.*`), C-ABI constructor symbol (`oxide_seo_create_rule`)
- **Experimental:** UI extension slots, post-processor SQL interface, exporter data format

### Installing Plugins

Copy the plugin directory to `{app_data_dir}/plugins/{plugin-name}/`. The app discovers plugins on startup and when you click Reload in the Plugin Manager.

Plugin-generated issues are automatically namespaced: `plugin.{plugin-name}.{rule-id}`.

## Troubleshooting

### `tauri::generate_context!()` panics

Icons must exist before build. Run `npx tauri icon app-icon.png` to generate them.

### Cargo commands fail at project root

Rust commands must run from `src-tauri/`, not the project root:

```bash
cd src-tauri && cargo test
```

### Pre-commit hook rejects files

The pre-commit hook runs ESLint and Prettier on staged `.ts`/`.tsx` files. Format before committing:

```bash
npx prettier --write src/path/to/file.tsx
```

### `cargo fix` breaks formatting

Always run `cargo fmt --all` after `cargo fix --allow-dirty`.

### TypeScript path aliases not resolving

Both `tsconfig.json` (for the editor) and `vite.config.ts` (for the bundler) must define the same path aliases. The `@` alias maps to `./src`.

### `method not found` on AppHandle

Add `use tauri::Manager;` to the file. Tauri v2's `Manager` trait provides methods like `.manage()`, `.path()`, and `.emit()`, but the compiler error does not suggest the missing import.

### Build fails with `composite` error

`tsconfig.node.json` must use `emitDeclarationOnly: true`, not `noEmit: true`. These conflict when `composite: true` is set.
