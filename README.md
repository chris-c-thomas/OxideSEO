# OxideSEO

Open-source, cross-platform desktop application for SEO site crawling and technical auditing. Built with Tauri v2, Rust, and React.

## Features

- **Zero cost, no limits** — No URL caps, no seat licenses, no feature gating
- **Native performance** — Rust crawl engine with configurable resource allocation
- **Modern UI** — React + Tailwind + shadcn/ui with virtualized tables (500k+ rows at 60fps)
- **Comprehensive auditing** — 25+ built-in SEO rules across meta, content, links, images, performance, and security
- **AI-augmented analysis** — Optional LLM integration via BYOK (user-provided API keys)
- **Extensible** — Plugin architecture for custom rules, exporters, and integrations

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable, 1.85+)
- [Node.js](https://nodejs.org/) (22 LTS)
- Platform-specific dependencies for Tauri v2:
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Visual Studio Build Tools, WebView2
  - **Linux**: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`

## Quick Start

```bash
# Clone the repository
git clone https://github.com/oxide-seo/oxide-seo.git
cd oxide-seo

# Install frontend dependencies
npm install

# Run in development mode (starts both Vite dev server and Rust backend)
npx tauri dev

# Build for production
npx tauri build
```

## Development Commands

| Command | Description |
|---|---|
| `npx tauri dev` | Start dev mode (frontend + Rust backend with hot reload) |
| `npx tauri build` | Production build for current platform |
| `npm run dev` | Frontend dev server only (no Rust backend) |
| `npm run lint` | Run ESLint |
| `npm run format` | Run Prettier |
| `npm run test` | Run frontend tests (Vitest) |
| `npm run typecheck` | TypeScript type checking |
| `cd src-tauri && cargo test` | Run Rust unit and integration tests |
| `cd src-tauri && cargo clippy` | Rust lint |
| `cd src-tauri && cargo fmt` | Rust format |

## Architecture

```
┌─────────────────────────────────────────────────────┐
│  FRONTEND (React + TypeScript + Tailwind)           │
│  Dashboard │ Crawl Config │ Monitor │ Results       │
└──────────────────────┬──────────────────────────────┘
                       │ Tauri IPC (serde JSON)
┌──────────────────────┴──────────────────────────────┐
│  RUST BACKEND (Tauri main process)                  │
│  ┌─────────────────────────────────────────────┐    │
│  │  Crawl Engine                                │    │
│  │  Frontier → Fetcher → Parser → Rule Engine   │    │
│  └─────────────────────────────────────────────┘    │
│  ┌──────────────┐ ┌─────────────┐ ┌────────────┐   │
│  │ Storage      │ │ SEO Rules   │ │ AI (BYOK)  │   │
│  │ (SQLite)     │ │ (25+ rules) │ │ (Phase 7)  │   │
│  └──────────────┘ └─────────────┘ └────────────┘   │
└─────────────────────────────────────────────────────┘
```

## Project Structure

```
oxide-seo/
├── src-tauri/               # Rust backend
│   ├── src/
│   │   ├── commands/        # Tauri IPC handlers
│   │   ├── crawler/         # Crawl engine (frontier, fetcher, parser)
│   │   ├── rules/           # SEO rule engine + built-in rules
│   │   ├── storage/         # SQLite database layer
│   │   └── ai/              # LLM provider adapters (Phase 7)
│   └── migrations/          # SQL migration files
├── src/                     # React frontend
│   ├── components/          # UI components by feature area
│   ├── hooks/               # Custom React hooks
│   ├── lib/                 # Tauri IPC wrappers, validation, utilities
│   ├── stores/              # Zustand state management
│   └── types/               # Shared TypeScript types
└── tests/                   # Test setup and fixtures
```

## Development Phases

| Phase | Description | Status |
|---|---|---|
| 1 | Project Foundation & Scaffolding | **Current** |
| 2 | Core Crawl Engine | Planned |
| 3 | SEO Rule Engine | Planned |
| 4 | Frontend UI & Data Presentation (MVP) | Planned |
| 5 | Export, Reporting & Crawl Management | Planned |
| 6 | Advanced Crawl Features | Planned |
| 7 | AI Integration (BYOK) | Planned |
| 8 | Plugin Architecture | Planned |

## License

Dual licensed under MIT and Apache 2.0. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).
