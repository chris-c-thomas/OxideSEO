# Reference: Tauri v2 Projects

Stack-specific guidance for documenting Tauri v2 desktop and mobile applications. Read this in addition to the Rust reference (Tauri apps are also Rust projects) when the target repo is a Tauri project.

## Detection

The repo is a Tauri v2 project if:

- A `src-tauri/` directory exists with `Cargo.toml`
- `src-tauri/tauri.conf.json` exists
- `@tauri-apps/api` v2.x is in the frontend `package.json`
- `tauri` v2.x is in `src-tauri/Cargo.toml`

Check the version specifically — Tauri v1 and v2 have substantially different architectures, configuration formats, and security models.

```bash
# Frontend version
jq '.dependencies["@tauri-apps/api"]' package.json

# Backend version
grep '^tauri' src-tauri/Cargo.toml
```

## What Makes Tauri Different

Tauri apps have **two halves** that need to be documented together but treated as distinct subsystems:

1. **Frontend** — A web app (any framework) running in a system webview
2. **Backend** — A Rust binary that hosts the webview, exposes commands, and accesses native APIs

The boundary between them is the IPC layer. Most of the unique documentation surface is here.

## Phase 1 Additions: Discovery

### Project Layout

Standard Tauri v2 layout:

```
.
├── src/                    # Frontend (your web app)
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   └── commands/
│   ├── capabilities/       # v2 capability files (permissions)
│   ├── icons/
│   ├── tauri.conf.json
│   ├── Cargo.toml
│   └── build.rs
├── package.json
└── (frontend build config)
```

### Tauri Configuration

Read and summarize `src-tauri/tauri.conf.json`:

```bash
jq '{
  productName,
  version,
  identifier,
  build: .build,
  app: {
    windows: .app.windows,
    security: .app.security,
    trayIcon: .app.trayIcon
  },
  bundle: {
    active: .bundle.active,
    targets: .bundle.targets,
    category: .bundle.category
  },
  plugins: .plugins
}' src-tauri/tauri.conf.json
```

Capture:

- Product name and bundle identifier
- Build commands (`beforeDevCommand`, `beforeBuildCommand`, `devUrl`, `frontendDist`)
- Window definitions (count, sizing, decorations, transparency)
- Bundle targets (deb, msi, dmg, app, appimage, etc.)
- Updater configuration
- Plugin list

### Frontend Stack

Detect the frontend framework as you would for any web project. Common Tauri pairings:

- React + Vite
- Svelte/SvelteKit
- Solid
- Vue + Vite
- Vanilla TS

Note: Next.js with Tauri requires static export (`output: 'export'`) and has its own caveats.

### Tauri Plugins

List enabled plugins from both manifests:

```bash
# Backend plugins
grep 'tauri-plugin-' src-tauri/Cargo.toml

# Frontend plugin bindings
jq '.dependencies | to_entries[] | select(.key | startswith("@tauri-apps/plugin-")) | .key' package.json
```

Common plugins to recognize:

- `tauri-plugin-fs` — Filesystem access
- `tauri-plugin-dialog` — Native dialogs
- `tauri-plugin-shell` — Shell command execution
- `tauri-plugin-store` — Persistent key-value store
- `tauri-plugin-sql` — Database access
- `tauri-plugin-http` — HTTP client (with CORS bypass)
- `tauri-plugin-notification` — System notifications
- `tauri-plugin-updater` — Auto-updates
- `tauri-plugin-window-state` — Window position persistence
- `tauri-plugin-deep-link` — Deep linking
- `tauri-plugin-single-instance` — Single instance enforcement
- `tauri-plugin-log` — Structured logging

### Capabilities (v2 Security Model)

Tauri v2's defining feature is its capability-based permission system. List all capability files:

```bash
ls src-tauri/capabilities/
cat src-tauri/capabilities/*.json
```

Each capability file declares:

- `identifier` — capability name
- `windows` — which windows it applies to
- `permissions` — list of granular permissions granted

This is critical to document — it's the security boundary.

### Custom Commands

Find every Tauri command exposed to the frontend:

```bash
rg '#\[tauri::command\]' src-tauri/src/ -A 2
```

Each `#[tauri::command]` is an IPC endpoint. Inventory them all.

### Frontend → Backend Calls

Find every `invoke` call:

```bash
rg "invoke\(['\"]" src/
```

These should pair 1:1 with the commands above. Mismatches are bugs or dead code.

### Events

Find event emitters and listeners:

```bash
# Backend emitters
rg 'emit\(' src-tauri/src/

# Frontend listeners
rg "listen\(['\"]" src/
```

### Mobile Targets (v2)

Tauri v2 supports iOS and Android. Check:

```bash
ls src-tauri/gen/ 2>/dev/null
ls src-tauri/gen/apple/ 2>/dev/null
ls src-tauri/gen/android/ 2>/dev/null
```

If mobile is configured, document the additional targets and their build requirements.

## Phase 2 Additions: Architecture

### IPC Architecture

This is the most important section for any Tauri app. Document:

1. **Command surface** — every `#[tauri::command]`, what it does, what permissions it requires
2. **Event surface** — every event the backend emits, every event the frontend listens to
3. **State management** — how `tauri::State` is used, what's stored in app state
4. **Async patterns** — which commands are async, which spawn background tasks

### Capability Model

Document the capability files explicitly:

| Capability File | Applies To   | Key Permissions                              |
| --------------- | ------------ | -------------------------------------------- |
| `default.json`  | main window  | `core:default`, `fs:allow-read-text-file`    |
| `admin.json`    | admin window | `shell:allow-execute`, `fs:allow-write-file` |

For each non-trivial permission, note _why_ it's granted and what feature requires it.

### Process Model

- Main process (Rust) — what it owns
- Webview process(es) — how many, what each renders
- Worker threads — any `tauri::async_runtime::spawn` patterns
- Sidecars — external binaries bundled with the app (`externalBin`)

### State and Persistence

- Where app state lives in memory (`tauri::State`)
- Where state persists to disk (tauri-plugin-store, custom files, SQLite via plugin-sql)
- Where logs go (`tauri-plugin-log` or custom)
- App data directory paths per platform (`app_data_dir`, `app_config_dir`, `app_log_dir`)

### Window Management

- Window count and roles
- Window-to-window communication (events, shared state)
- Tray integration
- Menu integration

### Update Strategy

If `tauri-plugin-updater` is configured:

- Update endpoint
- Signing key location (public key in `tauri.conf.json`, private key in CI secrets)
- Update channel strategy (stable, beta, etc.)
- User notification flow

## Phase 3 Additions: Synthesis

### Tauri-Specific README Sections

Adapt the README template to include:

1. **Platforms** — table of supported OS/architecture combinations
2. **Installation** — both "build from source" and "download release" paths
3. **Permissions** — high-level summary of what the app can access (links to capability docs)
4. **Auto-update** — does the app self-update? From where?

### Platforms Table

| Platform | Architecture | Bundle Format         | Status    |
| -------- | ------------ | --------------------- | --------- |
| macOS    | aarch64      | .dmg, .app            | Supported |
| macOS    | x86_64       | .dmg, .app            | Supported |
| Windows  | x86_64       | .msi, .exe            | Supported |
| Linux    | x86_64       | .deb, .AppImage, .rpm | Supported |
| iOS      | aarch64      | .ipa                  | Beta      |
| Android  | aarch64      | .apk, .aab            | Beta      |

### Installation Section

```bash
# Prerequisites
# - Rust (see rust-toolchain.toml)
# - Node.js 20+
# - Platform-specific deps:
#   - macOS: Xcode Command Line Tools
#   - Windows: WebView2 (preinstalled on Win11), Microsoft C++ Build Tools
#   - Linux: see https://v2.tauri.app/start/prerequisites/

# Install dependencies
pnpm install

# Run in dev mode
pnpm tauri dev

# Build for current platform
pnpm tauri build
```

### Architecture Section in README

A short paragraph explaining the two-process model, then link to ARCHITECTURE.md for the full IPC surface.

### Commands Reference

In `ARCHITECTURE.md`, include a table of every command:

| Command       | Permission Required                        | Purpose                       | Async |
| ------------- | ------------------------------------------ | ----------------------------- | ----- |
| `read_config` | `fs:allow-read-text-file`                  | Loads user config from disk   | Yes   |
| `save_image`  | `dialog:allow-save`, `fs:allow-write-file` | Saves image via native dialog | Yes   |
| ...           | ...                                        | ...                           | ...   |

## Phase 4 Additions: Verification

### Tauri-Specific Checks

```bash
# Verify every documented command exists
rg '#\[tauri::command\]' src-tauri/src/ -A 1 | rg 'fn \w+' -o | sort -u

# Verify every documented permission exists in a capability file
rg '"permissions"' src-tauri/capabilities/ -A 50 | rg '"[a-z-]+:[a-z-]+"' -o | sort -u

# Verify the bundle identifier matches what's documented
jq '.identifier' src-tauri/tauri.conf.json

# Verify icons exist for all platforms claimed in docs
ls src-tauri/icons/

# Verify the tauri version matches what's documented
grep '^tauri ' src-tauri/Cargo.toml
jq '.dependencies["@tauri-apps/api"]' package.json

# Verify the dev URL matches the frontend dev server config
jq '.build.devUrl' src-tauri/tauri.conf.json
```

### Build Verification (optional but high-value)

If feasible, run a debug build to verify the documented build process actually works:

```bash
pnpm tauri build --debug
```

Drift in the build instructions is the most common form of Tauri doc rot.

## Common Footguns to Document

These belong in the "Hidden Coupling and Footguns" section of ARCHITECTURE.md:

- **Permission scoping** — Permissions in v2 are per-capability, per-window. A permission granted to one window does not apply to another. New windows need explicit capability assignments.
- **CSP and `withGlobalTauri`** — If `withGlobalTauri: true`, the global `__TAURI__` object is exposed but CSP must allow it.
- **`devUrl` mismatch** — `tauri.conf.json` `build.devUrl` must exactly match the frontend dev server URL or `tauri dev` will hang.
- **Static export requirement** — Frontends that need a runtime (Next.js without `output: 'export'`, Remix in server mode) cannot be bundled into a Tauri app.
- **Path separators** — File paths from `path` plugin are platform-native. Frontend code must not assume POSIX separators.
- **Updater key management** — The private signing key must never be committed; the public key in `tauri.conf.json` must match.
- **Mobile build prerequisites** — iOS requires macOS + Xcode; Android requires JDK + Android SDK + NDK. Document required versions.
- **Bundle identifier immutability** — Changing `identifier` after release breaks updates and migrations on macOS and Windows.
- **App data directory differences** — `app_data_dir` differs per platform; migrations between installations need explicit handling.
- **Single-instance plugin ordering** — Must be registered first in the builder chain or it doesn't work.
- **File system scopes** — `fs` plugin permissions are scoped to specific directories; granting `fs:allow-read-text-file` without a scope grants nothing useful.
- **Inter-process state** — `tauri::State` is per-process; data must be cloneable and `Send + Sync`.

## Useful Commands Quick Reference

```bash
# Dev
pnpm tauri dev
pnpm tauri dev --no-watch

# Build
pnpm tauri build
pnpm tauri build --debug
pnpm tauri build --target universal-apple-darwin

# Info
pnpm tauri info  # System diagnostics — useful for bug reports

# Plugin add
pnpm tauri add <plugin-name>

# Permission inspection
pnpm tauri permission ls
pnpm tauri permission add <permission>
```

## Documentation Conventions to Honor

- **Link to v2 docs explicitly.** A lot of stale Tauri v1 documentation exists on the web. README links should point at https://v2.tauri.app/, not v1 docs.
- **Capability files are documentation.** They are the security model. Reference them directly from ARCHITECTURE.md rather than restating their contents.
- **Plugin docs live on tauri.app.** Don't duplicate plugin API documentation; link to the official plugin docs.
- **Bundle identifiers are versioned commitments.** Document the chosen identifier prominently and warn against changes.
- **Platform support matrix should be honest.** If iOS is "configured but untested," say so.
