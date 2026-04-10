# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.5.x   | Yes                |
| < 0.5   | No                 |

Only the latest release receives security updates. It's recommended to always be running the most recent version.

## Reporting a Vulnerability

The security of OxideSEO is taken seriously. If you discover a security vulnerability, please report it responsibly.

**Do not open a public GitHub issue for security vulnerabilities.**

Instead, please send a detailed report to:

**[security@oxideseo.app](mailto:security@oxideseo.app)**

### What to Include

- A description of the vulnerability and its potential impact
- Steps to reproduce the issue or a proof-of-concept
- The affected component (e.g., crawler, storage, plugin runtime, IPC boundary, frontend)
- Your suggested severity (Critical, High, Medium, Low)
- Any suggested fixes, if applicable

### Response Timeline

| Stage                | Target          |
| -------------------- | --------------- |
| Acknowledgment       | Within 48 hours |
| Initial assessment   | Within 7 days   |
| Fix or mitigation    | Within 30 days  |
| Public disclosure    | After fix ships  |

Disclosure will be coordinated with you. If a fix requires more than 30 days, expect regular status updates and an agreed-upon disclosure timeline.

## Security Architecture

OxideSEO is a desktop application built with Tauri v2 (Rust backend + React frontend). All data processing runs locally on the user's machine. There is no server component, telemetry, or phone-home behavior.

### Content Security Policy

The Tauri webview enforces a strict CSP:

```
default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'
```

All scripts are loaded from the application bundle. No remote code execution is permitted. Inline styles are allowed only for Tailwind CSS utility classes.

### Data Storage

- **Crawl data** is stored in a local SQLite database (`{app_data_dir}/oxide-seo.db`) with bundled SQLite (no system dependency).
- **AI API keys** are stored in OS-native credential managers (macOS Keychain, Windows Credential Manager, Linux Secret Service) via the `keyring` crate. Keys are never written to plaintext files, logs, or configuration.
- **No data leaves the machine** unless the user explicitly triggers an export or opts into AI analysis with their own API keys.

### Input Validation

- **SQL injection prevention:** All database queries use parameterized statements. LIKE patterns escape metacharacters. No string interpolation in SQL.
- **HTML sanitization:** Exported HTML reports escape all special characters (`&`, `<`, `>`, `"`) to prevent injection.
- **Crawl config validation:** Both the frontend (Zod schemas) and backend (Rust) independently validate configuration values with strict bounds (e.g., max depth 1-100, concurrency 1-200, timeout 5-120s).
- **IPC boundary:** All Tauri `invoke()` calls use typed wrappers. The frontend enforces TypeScript strict mode with `noUncheckedIndexedAccess`.

### Web Crawling Safety

- **robots.txt compliance:** RFC 9309 compliant parsing. Crawl-Delay directives are respected.
- **Redirect limits:** Manual redirect tracking with a maximum of 10 hops to prevent infinite loops and open redirect abuse.
- **Response size limits:** Maximum 10 MB response body to prevent out-of-memory conditions.
- **Rate limiting:** Per-host concurrency limits (default 2) and configurable crawl delays to avoid overwhelming target servers.
- **TLS:** HTTP requests use `rustls` (pure-Rust TLS). No dependency on OpenSSL.

### Plugin Security

OxideSEO supports two plugin runtimes with different trust models:

- **WASM plugins** (community/untrusted): Sandboxed via wasmtime Component Model with fuel metering (10M instruction limit per call), 64 MB memory cap, and an explicit capability system. Plugins must declare required capabilities (e.g., `log`, `http_read`, `db_read`) in their manifest. Access beyond declared capabilities is denied at runtime.
- **Native plugins** (trusted/first-party): Loaded via dynamic linking with full system access. Require `trusted = true` in their manifest. Native plugins must be compiled with the same Rust toolchain as the host application.

### Error Handling

- Production paths never panic. `unwrap()` and `expect()` are prohibited outside of tests.
- IPC errors are mapped to `Result<T, String>` at the command boundary. Internal error context (via `anyhow`) does not leak to the frontend.
- Structured logging via `tracing` with configurable log levels. Sensitive data is not logged.

## Scope

The following are considered in-scope for security reports:

- Remote code execution or sandbox escape (especially WASM plugin boundary)
- SQL injection or data corruption in the local database
- Cross-site scripting (XSS) in the Tauri webview or exported reports
- Credential leakage (API keys exposed in logs, exports, or plaintext files)
- Path traversal in export, import, or plugin file operations
- Denial of service via crafted HTML, sitemaps, or robots.txt during crawls
- IPC boundary violations (frontend bypassing backend validation)
- SSRF via redirect chains or crafted URLs

The following are out of scope:

- Vulnerabilities requiring physical access to the user's machine
- Social engineering attacks
- Denial of service against websites being crawled (this is a local tool under user control)
- Issues in third-party dependencies without a demonstrated exploit path in OxideSEO

## Dependencies

OxideSEO pins dependency versions in both `Cargo.lock` and `package-lock.json`. Known vulnerabilities are monitored using `cargo audit` and `npm audit`. If you discover a vulnerability in a dependency that affects OxideSEO, please report it so the impact can be assessed and patched promptly.

## Acknowledgments

The security research community's efforts to make open-source software safer is greatly appreciated. Reporters who follow responsible disclosure will be credited in our release notes (unless they prefer to remain anonymous).
