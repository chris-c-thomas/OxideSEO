# OxideSEO

Open-source, cross-platform desktop application for SEO site crawling and technical auditing. Built with Rust, Tauri v2, and React.

<!-- TODO: Add screenshot here -->
<!-- ![OxideSEO screenshot](docs/assets/screenshot.png) -->

[![CI](https://github.com/chris-c-thomas/OxideSEO/actions/workflows/ci.yml/badge.svg)](https://github.com/chris-c-thomas/OxideSEO/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

## Overview

OxideSEO crawls websites and evaluates them against 21 built-in SEO rules covering meta tags, content quality, links, images, performance, and security. It runs entirely on your machine with no external service dependencies, no URL limits, and no data leaving your computer.

The Rust backend handles HTTP fetching, HTML parsing, and rule evaluation across concurrent workers. A channel-based engine coordinates tokio (async I/O), rayon (CPU-bound parsing), and a dedicated SQLite writer thread for high-throughput crawling. The React frontend provides a virtualized data explorer that handles datasets of 100k+ rows at 60fps.

Optional AI analysis integrates with OpenAI, Anthropic, or local Ollama models via user-provided API keys. A plugin system supports WASM (sandboxed) and native extensions for custom rules, export formats, and post-crawl processors.

## Features

- Crawls websites with configurable concurrency, depth limits, and URL pattern filtering
- Evaluates pages against 21 built-in SEO rules across 6 categories (meta, content, links, images, performance, security)
- Cross-page analysis detects duplicate titles, orphan pages, and broken internal links
- Respects robots.txt (RFC 9309) with per-domain rate limiting and crawl delay
- Discovers and cross-references XML sitemaps
- Checks external links via HEAD requests
- Renders JavaScript-heavy pages via headless webview for SPA support
- Exports to CSV, NDJSON, HTML report, PDF report, and Excel (XLSX)
- Saves and opens portable `.seocrawl` project files
- Compares two crawls with page-level, issue-level, and metadata diffs
- Hierarchical site tree visualization
- Real-time crawl monitoring with memory and throughput gauges
- AI-powered content analysis, meta description generation, and structured data recommendations (BYOK: OpenAI, Anthropic, Ollama)
- Plugin system with WASM sandboxing and native plugin support
- Dark mode with system preference detection
- Keyboard shortcuts for navigation and actions
- Privacy-first: no telemetry, no analytics, no data leaves your machine

## Quickstart

```bash
git clone https://github.com/chris-c-thomas/OxideSEO.git
cd OxideSEO
npm install
npx tauri dev
```

The app window opens automatically with the dashboard view.

## Requirements

| Tool | Version | Notes |
|---|---|---|
| Rust | 1.85+ (stable) | Install via [rustup](https://rustup.rs/) |
| Node.js | 22 LTS | Install via [nodejs.org](https://nodejs.org/) |
| npm | Included with Node.js | |

Platform-specific dependencies:

- **macOS:** Xcode Command Line Tools (`xcode-select --install`)
- **Windows:** [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/), WebView2
- **Linux (Debian/Ubuntu):** `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`

For other Linux distributions, see the [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/).

## Installation

### Pre-built Binaries

Download the installer for your platform from [GitHub Releases](https://github.com/chris-c-thomas/OxideSEO/releases).

| Platform | Formats |
|---|---|
| macOS | `.dmg` |
| Windows | `.msi`, `.exe` |
| Linux | `.deb`, `.AppImage` |

### Build from Source

```bash
# Clone the repository
git clone https://github.com/chris-c-thomas/OxideSEO.git
cd OxideSEO

# Install frontend dependencies
npm install

# Build for your current platform
npx tauri build
```

Installers are output to `src-tauri/target/release/bundle/`.

## Usage

1. **Start a crawl:** Click "New Crawl" from the dashboard, enter a URL, configure options, and click Start.
2. **Monitor progress:** The crawl monitor shows real-time stats: URLs crawled, throughput, memory usage, and recent URLs.
3. **Explore results:** After completion, browse results across tabs: Pages, Issues, Links, Images, Sitemap, External Links, Site Tree, and AI Insights.
4. **Export data:** Open the export dialog to save results as CSV, NDJSON, HTML, PDF, or XLSX.
5. **Compare crawls:** Select two completed crawls from the dashboard to view page-level, issue-level, and metadata diffs.

For development workflows, see [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

## Architecture

OxideSEO runs as a single Tauri v2 process with a React frontend in a system webview and a Rust backend. The crawl engine uses a channel-based actor model: a tokio orchestrator dispatches URLs to async fetch workers, which hand off to a rayon parse and rules pool, which sends batched write commands to a dedicated SQLite writer thread. 47 Tauri IPC commands handle all frontend-backend communication. All data processing, sorting, filtering, and pagination happen server-side in Rust.

For the full architecture documentation, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Project Structure

```
OxideSEO/
├── src/                         # React frontend (TypeScript)
│   ├── components/              # UI components by feature domain
│   ├── hooks/                   # Custom React hooks
│   ├── lib/                     # IPC wrappers, validation, utilities
│   ├── stores/                  # Zustand state management
│   └── types/                   # TypeScript types matching Rust IPC
├── src-tauri/                   # Rust backend
│   ├── src/
│   │   ├── commands/            # Tauri IPC handlers (47 commands)
│   │   ├── crawler/             # Crawl engine (frontier, fetcher, parser)
│   │   ├── rules/               # SEO rule engine + 21 built-in rules
│   │   ├── storage/             # SQLite database layer
│   │   ├── ai/                  # LLM provider adapters
│   │   └── plugin/              # Plugin system (WASM + native)
│   └── migrations/              # SQL migration files
├── docs/                        # Project documentation
│   ├── ARCHITECTURE.md          # System design and data flow
│   ├── DEVELOPMENT.md           # Setup, workflows, troubleshooting
│   ├── CONTRIBUTING.md          # Contribution process and code style
│   ├── DEPLOYMENT.md            # Build, CI, and distribution
│   └── adr/                     # Architecture Decision Records
├── plugins/examples/            # Example plugins
└── tests/                       # Frontend test setup and fixtures
```

## Scripts

| Script | Purpose |
|---|---|
| `npx tauri dev` | Start dev mode (frontend + Rust backend with hot reload) |
| `npx tauri build` | Production build for current platform |
| `npm run dev` | Frontend dev server only (port 1420) |
| `npm run build` | TypeScript check + Vite production bundle |
| `npm run lint` | ESLint (zero-warning policy) |
| `npm run format` | Prettier auto-format |
| `npm run format:check` | Check Prettier formatting |
| `npm run test` | Frontend tests (Vitest) |
| `npm run test:watch` | Frontend tests in watch mode |
| `npm run typecheck` | TypeScript type checking |
| `cd src-tauri && cargo test` | Rust unit and integration tests |
| `cd src-tauri && cargo clippy --all-targets -- -D warnings` | Rust lint |
| `cd src-tauri && cargo fmt --all` | Rust auto-format |

## Testing

```bash
# Frontend
npm run test

# Rust (from src-tauri/)
cd src-tauri && cargo test
```

For testing details, writing tests, and CI configuration, see [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md#testing).

## Contributing

Contributions are welcome. See [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) for the contribution process, code style guidelines, and PR requirements.

## License

Dual licensed under [MIT](LICENSE-MIT) and [Apache 2.0](LICENSE-APACHE). Choose whichever you prefer.
