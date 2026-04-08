# Deployment

This document describes how OxideSEO is built, distributed, and updated.

For local development, see [DEVELOPMENT.md](DEVELOPMENT.md).
For architecture, see [ARCHITECTURE.md](ARCHITECTURE.md).

## Overview

OxideSEO is a desktop application distributed as platform-specific installers. There is no server deployment. Users download and install the app directly.

## Supported Platforms

| Platform | Architecture | Bundle Formats | Status |
|---|---|---|---|
| macOS | aarch64 (Apple Silicon) | `.dmg`, `.app` | Supported |
| macOS | x86_64 (Intel) | `.dmg`, `.app` | Supported |
| Windows | x86_64 | `.msi`, `.exe` (NSIS) | Supported |
| Linux | x86_64 | `.deb`, `.AppImage`, `.rpm` | Supported |

## Build Process

### Local Build

```bash
# Production build for the current platform
npx tauri build
```

This runs:
1. `npm run build` -- TypeScript compilation (`tsc -b`) + Vite production bundle -> `dist/`
2. `cargo build --release` -- Optimized Rust binary
3. Tauri packages `dist/` into the binary and produces platform-specific installers

Build output location: `src-tauri/target/release/bundle/`

### Debug Build

```bash
npx tauri build --debug
```

Produces an unoptimized build with debug symbols. Useful for diagnosing release-only issues.

### Cross-Platform Build (macOS Universal)

```bash
npx tauri build --target universal-apple-darwin
```

Produces a universal binary for both Apple Silicon and Intel Macs.

## Continuous Integration

CI is defined in `.github/workflows/ci.yml`. It runs on every push to `main` and every pull request targeting `main`.

### Pipeline

```
Push / PR to main
    |
    ├── rust-check (Ubuntu, macOS, Windows)
    │   ├── cargo fmt --check
    │   ├── cargo clippy (zero warnings)
    │   ├── cargo test --lib
    │   └── cargo test --test '*' (integration, continue-on-error)
    │
    ├── frontend-check (Ubuntu)
    │   ├── npm ci
    │   ├── npm run lint
    │   ├── npm run format:check
    │   ├── npm run typecheck
    │   └── npm run test
    │
    └── build (Ubuntu, macOS, Windows) [only on push/PR to main]
        ├── Depends on rust-check + frontend-check passing
        ├── npx tauri build
        └── Upload artifacts (7-day retention)
```

### CI Environment

| Variable | Value | Purpose |
|---|---|---|
| `CARGO_TERM_COLOR` | `always` | Colored Cargo output |
| `RUST_BACKTRACE` | `1` | Enable Rust backtraces |
| `TAURI_SIGNING_PRIVATE_KEY` | `""` (empty) | No code signing in CI |

## Distribution

Pre-built binaries are available from:
- [GitHub Releases](https://github.com/chris-c-thomas/OxideSEO/releases) -- all platforms
- Project website -- download links for each platform

### File Association

The app registers `.seocrawl` files (MIME: `application/x-seocrawl`). Double-clicking a `.seocrawl` file opens it in OxideSEO. These files are portable SQLite databases containing a complete crawl.

## Code Signing

Code signing is required for distribution on macOS (Gatekeeper) and recommended on Windows (SmartScreen). The signing key is stored as a CI secret and applied during the build job.

The public key for update verification is configured in `src-tauri/tauri.conf.json`. The private signing key must never be committed to the repository.

## Auto-Update

OxideSEO uses `tauri-plugin-updater` for self-updating. When a new version is available:

1. The app checks the update endpoint on launch
2. If an update is found, the user is notified
3. The update downloads and applies in the background
4. The user restarts the app to complete the update

## Bundle Identifier

The bundle identifier is `com.oxideseo.desktop`. This identifier is used by the OS for app identity, data directory paths, and update verification. Changing it after release breaks updates and data migration on macOS and Windows.

## Versioning

OxideSEO follows [Semantic Versioning](https://semver.org/). The version is declared in:
- `src-tauri/tauri.conf.json` (`version` field)
- `src-tauri/Cargo.toml` (`version` field)
- `package.json` (`version` field)

All three must stay in sync. The changelog is generated from conventional commits via [git-cliff](https://git-cliff.org/).

## System Diagnostics

For troubleshooting build or runtime issues, run:

```bash
npx tauri info
```

This prints system details (OS, Rust version, Node version, webview version) useful for bug reports.
