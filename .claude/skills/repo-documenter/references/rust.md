# Reference: Rust Projects

Stack-specific guidance for documenting Rust projects. Read this in addition to the main phase files when the target repo is a Rust project (binary, library, or workspace).

## Detection

The repo is a Rust project if any of these are present:

- `Cargo.toml` at the root
- `rust-toolchain.toml` or `rust-toolchain` file
- `.rs` files as the dominant source language

Determine the project shape from `Cargo.toml`:

- `[package]` only → single crate
- `[workspace]` with `members` → workspace (multi-crate)
- Both `[package]` and `[lib]`/`[[bin]]` → library and/or binary

## Phase 1 Additions: Discovery

Add these to the inventory:

### Crate Identity

```bash
cargo metadata --format-version 1 --no-deps | jq '.packages[] | {name, version, edition, rust_version}'
```

Capture:

- Crate name(s) and version(s)
- Rust edition (2018, 2021, 2024)
- MSRV (`rust-version` field)
- Whether it's a `lib`, `bin`, both, or workspace
- Workspace member count and shape

### Toolchain

- Pinned toolchain (`rust-toolchain.toml`)
- Required components (rustfmt, clippy, miri, etc.)
- Required targets (cross-compilation)

```bash
cat rust-toolchain.toml 2>/dev/null
```

### Cargo Features

List all features and their dependencies:

```bash
cargo metadata --format-version 1 --no-deps | jq '.packages[] | {name, features}'
```

Document which features are default, which are mutually exclusive, and which gate platform-specific code.

### Dependencies (grouped)

```bash
cargo tree --depth 1 --edges normal
```

Group by purpose in the inventory:

- **Async runtime**: tokio, async-std, smol
- **Web framework**: axum, actix-web, rocket, warp, actix
- **Serialization**: serde, serde_json, rmp-serde, bincode
- **Error handling**: thiserror, anyhow, eyre, snafu
- **Database**: sqlx, diesel, sea-orm, rusqlite
- **HTTP client**: reqwest, ureq, hyper
- **CLI**: clap, structopt, argh
- **Logging**: tracing, log, env_logger, slog
- **Testing**: proptest, quickcheck, insta, mockall, rstest

### Build Configuration

- `[profile.*]` overrides in Cargo.toml
- `build.rs` presence and what it does
- `.cargo/config.toml` settings (custom targets, linker config, registry overrides)

### Binaries and Examples

```bash
find . -path ./target -prune -o -name 'main.rs' -print
ls examples/ 2>/dev/null
```

### Tests

```bash
find . -path ./target -prune -o -name 'tests' -type d -print
find tests -name '*.rs' 2>/dev/null
```

Distinguish unit tests (in `src/`), integration tests (in `tests/`), doctests, and benchmarks (`benches/`).

## Phase 2 Additions: Architecture

### Module Boundaries

Rust's module system is the architecture. Document:

- Top-level modules (`src/lib.rs` or `src/main.rs` `mod` declarations)
- Public API surface (`pub use` re-exports in `lib.rs`)
- Internal vs. public modules

```bash
rg '^(pub )?mod ' src/lib.rs src/main.rs 2>/dev/null
rg '^pub use ' src/lib.rs 2>/dev/null
```

### Error Handling Strategy

Determine the project's error model:

- `thiserror`-based custom error enums (typed, library-style)
- `anyhow`/`eyre` (dynamic, application-style)
- Mixed (libraries use thiserror, binaries use anyhow)
- Custom Result type alias

Find the canonical error type:

```bash
rg 'pub (enum|struct) \w*Error' src/
rg 'pub type Result' src/
```

### Async Model

- Which runtime
- Whether the library is runtime-agnostic or tokio-specific
- Whether `Send`/`Sync` bounds are enforced
- Spawn vs. block_on patterns

### Unsafe Surface

```bash
rg -c 'unsafe' src/ | sort -t: -k2 -n -r
```

Document any unsafe blocks and their justification. This is high-leverage for new contributors.

### FFI Boundaries

- `extern "C"` declarations
- `#[no_mangle]` exports
- C header generation (cbindgen, cxx)
- Bindings (`bindgen` build scripts)

## Phase 3 Additions: Synthesis

### README for a Library Crate

Library crates need a different README structure than apps. Adapt the template:

1. Title + one-sentence purpose
2. crates.io and docs.rs badges (if published)
3. Overview
4. **Add to your project** (Cargo.toml snippet)
5. **Quick example** (10-30 lines, runnable)
6. Feature flags table
7. MSRV statement
8. Links to docs.rs for full API reference
9. License (typically dual MIT/Apache-2.0)

```toml
[dependencies]
your-crate = "1.0"
```

### README for a Binary Crate

Closer to the standard application README, but add:

- Installation via `cargo install`
- Pre-built binaries (if released)
- Shell completion setup

### Cargo Features Table

| Feature | Default | Pulls In              | Purpose               |
| ------- | ------- | --------------------- | --------------------- |
| `tokio` | yes     | `tokio`, `tokio-util` | Async runtime support |
| `serde` | no      | `serde`               | Serialization derives |
| `tls`   | no      | `rustls`              | HTTPS support         |

### Workspace Structure (for workspace projects)

```
crates/
├── core/         # Pure logic, no I/O
├── api/          # HTTP layer
├── cli/          # Binary entry point
└── macros/       # Proc macros
```

### Architecture Notes Specific to Rust

In `ARCHITECTURE.md`, include:

- **Lifetimes**: Any non-trivial lifetime relationships at API boundaries
- **Type-state pattern**: If used, document the state transitions
- **Trait hierarchy**: For libraries with significant generic code
- **Send/Sync**: Document any `!Send` or `!Sync` types
- **Panic policy**: Does the library panic? Under what conditions? Or does it return Results?

## Phase 4 Additions: Verification

### Cargo-Specific Checks

```bash
# Verify documented features actually exist
cargo metadata --format-version 1 --no-deps | jq -r '.packages[].features | keys[]' | sort -u

# Verify documented binaries exist
cargo metadata --format-version 1 --no-deps | jq -r '.packages[].targets[] | select(.kind[0] == "bin") | .name'

# Verify the documented MSRV builds
rustup install 1.XX.0
cargo +1.XX.0 check

# Verify doctests pass (they're part of the docs!)
cargo test --doc

# Verify no broken intra-doc links
cargo doc --no-deps --document-private-items -- -D rustdoc::broken-intra-doc-links
```

### Doc Comment Coverage

For libraries, check that every public item has a doc comment:

```bash
cargo +nightly rustdoc --lib -- -D missing-docs
```

If the project has missing docs, surface this in the verification report — it's drift between intent and reality.

## Common Footguns to Document

These are high-value for the "Hidden Coupling and Footguns" section of ARCHITECTURE.md:

- **Feature unification**: cargo features are additive across the dep graph; mutually-exclusive features must be enforced at compile time via `compile_error!`
- **Default features in dev-dependencies**: enabling a feature for a dev-dep can leak into the main build
- **Workspace inheritance**: `workspace = true` in `Cargo.toml` requires the corresponding entry in the workspace root
- **`cargo build` vs `cargo build --release`**: profile differences (overflow checks, debug assertions, panic strategy)
- **`build.rs` rerun-if directives**: missing `cargo:rerun-if-changed=` lines cause stale builds
- **Async runtime mismatches**: spawning a `tokio::task` from a `smol` context will panic
- **`#[cfg]` drift**: code that only compiles on certain targets needs clear docs and `--all-targets` testing

## Useful Commands Quick Reference

```bash
# Project metadata
cargo metadata --format-version 1 --no-deps | jq

# Dependency tree
cargo tree
cargo tree -e features
cargo tree -d  # duplicates

# Outdated deps
cargo outdated  # requires cargo-outdated

# Unused deps
cargo +nightly udeps  # requires cargo-udeps

# Public API surface
cargo public-api  # requires cargo-public-api

# Audit
cargo audit

# Doc generation
cargo doc --open --no-deps --document-private-items

# Clippy
cargo clippy --all-targets --all-features -- -D warnings

# Format check
cargo fmt --all -- --check

# Bloat analysis
cargo bloat --release  # requires cargo-bloat
```

## Documentation Conventions to Honor

Rust has strong existing conventions. The generated docs should align with them, not fight them:

- **rustdoc is the primary API documentation.** README and ARCHITECTURE docs should link to docs.rs for API details, not duplicate them.
- **Doctests are documentation.** Every public function should have a usage example in its doc comment, and those examples must compile.
- **The crate-level doc comment in `lib.rs`** is what shows on docs.rs. It should mirror the README for published crates (use `#![doc = include_str!("../README.md")]`).
- **CHANGELOG.md** for libraries should follow Keep a Changelog and align with semver releases.
- **MSRV bumps** are breaking changes for some users — document policy explicitly.
