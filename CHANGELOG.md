# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0] - 2026-04-10

### Bug Fixes

- **frontend:** Rename raw shadow vars to avoid self-referential @theme inline tokens
- **frontend:** Deduplicate tokens, fix dark mode warning/shadow visibility

### Miscellaneous

- Bump v0.7.0

### Refactoring

- **frontend:** Migrate color theme to teal-blue/amber palette with Manrope + IBM Plex Mono fonts

## [0.6.0] - 2026-04-10

### Bug Fixes

- **test:** Address Copilot PR review feedback on E2E mocks and shortcuts
- **frontend:** Address E2E test PR review findings

### Miscellaneous

- Update CHANGELOG.md
- Bump v0.6.0
- Update CLAUDE.md
- Update MIT and Apache-2.0 licenses
- Update package.json information
- Add SECURITY.md

### Testing

- **frontend:** Add IPC error simulation, IssuesView spec, and coverage gaps
- **frontend:** Add Playwright E2E tests with Tauri IPC mocking

## [0.5.0] - 2026-04-10

### Bug Fixes

- Address GitHub Copilot PR review findings
- Cargo fmt fix
- Address PR review findings for crawl lifecycle management
- **frontend:** Address GitHub Copilot PR review findings
- **frontend:** Address PR review suggestions
- **frontend:** Resolve PR review findings for error handling and token collision
- **ui:** Add delete crawl button
- **frontend:** Resolve styling failures and UI issues
- GitHub Copilot PR Review issues addressed

### Dependencies

- **deps:** Bump the cargo group across 1 directory with 2 updates

### Documentation

- Fix inaccuracies and reduce duplication in project documentation
- Add comprehensive user facing documentation for OxideSEO

### Features

- **crawl:** Add delete, re-run, and full lifecycle management

### Miscellaneous

- Bump v.0.5.0
- Update CLAUDE.md
- Update CLAUDE.md
- **skills:** Add ai agent skills
- Update .gitignore
- Update package-lock.json
- **skill:** Repo-documenter skill
- Refine CLAUDE.md

### Refactoring

- **frontend:** Improve Dashboard type safety and remove redundant comment

## [0.4.0] - 2026-04-08

### Bug Fixes

- GitHub Copilot PR Review issues
- **results:** Address PR review issues in comparison queries, types, and error handling
- **deps:** Update vite to 6.4.2 and vitest to 3.2.4 to resolve esbuild vulnerability
- **plugin:** Address PR review feedback on persistence, ordering, a11y, and log spam
- **plugin:** Fix async deadlock, path traversal, SQL validation, and type safety issues
- **ai:** Fix false-positive connection test, error handling, and input validation
- Vitest issues
- **ai:** Improve error handling, safety, and code quality from PR review
- **crawler:** Address Copilot PR review feedback across 6 issues
- **crawler:** Harden error handling across Phase 6 crawl features
- Fmt fix
- **export:** Improve type safety, error handling, and UI correctness
- **export:** Harden ATTACH DATABASE lifecycle and error handling
- **storage,rules,frontend:** Address PR review issues 5-16

### Documentation

- Update CHANGELOG.md for v0.4.0

### Features

- **results:** Add crawl comparison / diff view
- **results:** Add site tree visualization tab (D-7)
- **export:** Add PDF report and Excel (XLSX) export formats (D-5, D-6)
- **crawler:** Add render-blocking rule, AI prompts, and resource meter (D-1–D-4)
- **ai:** Add summary regeneration, cost estimation, Ollama model discovery, and frontend tests
- **ai:** Phase 7 - export integration and tests
- **ai:** Implement Phase 7 - AI integration with BYOK provider support
- **frontend:** Add Sitemap and External Links result tabs
- **crawler:** Add JavaScript rendering pipeline with hidden Tauri webviews
- **crawler:** Add external link checker and custom CSS extraction
- **crawler:** Add cookie auth, URL pattern filtering, and advanced config UI
- **crawler:** Add Phase 6 schema migration and advanced CrawlConfig fields
- **frontend:** Add export dialog, results export button, and dashboard crawl management
- **export:** Implement .seocrawl file save and open with ATTACH DATABASE
- **settings:** Implement settings and rule config persistence
- **export:** Implement HTML report generation with summary stats and top issues
- **export:** Implement NDJSON export with streaming and column selection
- **export:** Implement CSV export with streaming and column selection
- **storage:** Add streaming export queries, report aggregates, and settings persistence
- **export:** Scaffold Phase 5 export module with IPC types and command stubs

### Miscellaneous

- Update CLAUDE.md
- Update CLAUDE.md
- Update CLAUDE.md files
- Update CLAUDE.md

### Refactoring

- Replace positional tuples with named structs and add error logging
- **plugin:** Improve type safety, error specificity, validation schemas, and load error surfacing
- **ai:** Extract cost helper, name constants, improve error messages and test coverage
- **ai:** Improve type safety, JSON validation, and test coverage
- **rules:** Replace stringly-typed severity and category with enums

### Testing

- **crawler:** Add Phase 6 tests and fix sitemap discovery port handling

### Update

- Conventions.md

## [0.3.0] - 2026-04-05

### Bug Fixes

- **crawler,storage:** Handle channel send errors and flush failures in crawl pipeline
- **storage,rules:** Propagate row deserialization errors and fix UTF-8 truncation panic
- Formatting

### Features

- **ui:** Implement Phase 4 frontend UI with results tables and page detail
- **rules:** Implement Phase 3 SEO rule engine

### Miscellaneous

- Update CLAUDE.md

## [0.2.0] - 2026-04-04

### Bug Fixes

- **build:** Add @types/node and fix bundle identifier

### Features

- **crawler:** Implement Phase 2 core crawl engine
- Wire tauri commands for crawl lifecycle
- Implement crawl engine orchestrator
- Implement frontier SQLite overflow
- Implement storage writer thread + query execution
- Integrate texting_robots for robots.txt
- Implement fetcher streaming body with blake3 hash
- **parser:** Implement HTML parser with lol_html and scraper fallback

## [0.1.0] - 2026-04-03

### Features

- Git-cliff
- **oxideseo:** Initial project scaffold

### Miscellaneous

- Update CLAUDE.md
- Phase 1
- Update .gitignore
- Update CLAUDE.md
- Fix initial scaffold; build app
- Change README.md; add Cargo.lock

