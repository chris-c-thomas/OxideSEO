# 0003. WASM Component Model for Plugin Sandboxing

Date: 2026-04-07

## Status

Accepted

## Context

OxideSEO needs a plugin system that allows third-party code to extend SEO rules, export formats, and post-crawl processors. Plugins from untrusted sources must not be able to crash the host, access arbitrary files, or exfiltrate data. At the same time, first-party plugins need full performance and system access.

## Decision

We will support two plugin runtimes:

1. **WASM plugins** via wasmtime's Component Model for untrusted/community plugins, with fuel metering, memory limits, and capability-based permissions
2. **Native plugins** via `libloading` for trusted/first-party plugins, with no sandboxing but a mandatory `trusted = true` flag in the manifest

The WASM interface is defined using WIT (WebAssembly Interface Types) in `src-tauri/wit/`. Plugin capabilities (log, http_read, db_read, fs_read_plugin_dir) are declared in the plugin manifest and enforced at runtime.

## Consequences

### Positive

- Community plugins run in a sandboxed environment with deterministic resource limits
- Fuel metering prevents infinite loops and denial-of-service
- Capability declarations make the security model explicit and auditable
- Native plugins provide full performance for first-party use cases
- The WIT interface provides a stable ABI that survives Rust compiler upgrades

### Negative

- wasmtime adds ~5-10MB to the binary size (gated behind the `plugin-wasm` feature)
- WASM plugins have higher per-call overhead than native plugins
- Native plugins require the same Rust toolchain version as the host due to vtable instability
- Two plugin runtimes increase implementation and documentation surface

### Neutral

- The `plugin-wasm` feature is default-enabled but can be disabled for minimal builds

## Alternatives Considered

### Alternative 1: WASM Only

Require all plugins to be WASM.

Rejected because some first-party use cases (e.g., native library integrations) need full system access and cannot tolerate WASM overhead or capability restrictions.

### Alternative 2: Lua/JavaScript Scripting

Embed a scripting runtime (mlua, boa, deno_core).

Rejected because scripting runtimes lack the sandboxing guarantees of WASM's linear memory model and capability system. Fuel metering is also harder to implement correctly in a scripting VM.

## References

- [WebAssembly Component Model](https://component-model.bytecodealliance.org/)
- [wasmtime documentation](https://docs.wasmtime.dev/)
