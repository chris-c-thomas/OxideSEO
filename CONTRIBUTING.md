# Contributing to OxideSEO

## Development Setup

1. Install prerequisites: Rust (1.85+), Node.js (22 LTS), platform dependencies (see README)
2. Clone the repo and run `npm install`
3. Run `npx tauri dev` to start development mode

## Code Style

**Rust**: `rustfmt` + `clippy`. Run `cargo fmt` and `cargo clippy` before committing. CI enforces zero warnings.

**TypeScript**: ESLint + Prettier. Run `npm run lint` and `npm run format:check`. CI enforces zero warnings.

## Pull Request Process

1. Create a feature branch from `main`
2. Write tests for new functionality
3. Ensure all checks pass: `cargo test`, `cargo clippy`, `npm run lint`, `npm run test`, `npm run typecheck`
4. Submit a PR with a clear description of changes

## Module Ownership

| Area | Directory | Description |
|---|---|---|
| Crawl Engine | `src-tauri/src/crawler/` | URL frontier, HTTP fetcher, HTML parser, politeness |
| Rule Engine | `src-tauri/src/rules/` | SeoRule trait, registry, built-in rules |
| Storage | `src-tauri/src/storage/` | SQLite schema, queries, models |
| IPC Commands | `src-tauri/src/commands/` | Tauri command handlers |
| Frontend | `src/` | React components, hooks, stores, types |

## Adding New Rules

See the "Adding a new SEO rule" section in CLAUDE.md for step-by-step instructions.
