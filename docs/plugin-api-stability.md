# Plugin API Stability Contract

## Overview

This document defines the stability guarantees for the OxideSEO plugin API. Plugin authors can rely on these guarantees when building and distributing plugins.

## Stability Levels

### Stable (v1.0+)

- **PluginParsedPage fields** — Fields are append-only. Existing fields will never be renamed or removed. New optional fields may be added.
- **WIT world names** — `seo-rule-plugin` and `seo-exporter-plugin` are stable identifiers.
- **Manifest format** — The `plugin.toml` schema is backward-compatible. New optional fields may be added.
- **Issue rule_id namespace** — `plugin.*` prefix is reserved for plugin-generated issues.
- **C-ABI constructor symbol** — `oxide_seo_create_rule` is the stable entry point for native rule plugins.

### Experimental (subject to change)

- **UI extension slots** — The `results-tab` slot and the `PluginSlot` component may change.
- **Post-processor SQL query interface** — The read-only SQL callback API may evolve.
- **Exporter data format** — The JSON structure passed to exporter plugins may change.

## Versioning

The WIT interface follows semver: `oxide-seo:plugin@0.1.0`. Breaking changes increment the major version. Additive changes increment the minor version.

## Deprecation Policy

- Deprecated features will be marked with a `@deprecated` annotation.
- Deprecated features remain functional for at least 2 minor releases.
- Removal of deprecated features is announced in release notes.

## Native Plugin ABI Compatibility

Native plugins use Rust `dyn` trait objects. The vtable layout is **not stable across Rust compiler versions**. Native plugins must be compiled with the same Rust toolchain version as the host application. This is documented and acceptable for the "trusted/first-party" scope of native plugins.
