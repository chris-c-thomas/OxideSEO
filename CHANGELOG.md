# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-04-06

### Bug Fixes

- **ai:** Improve error handling, safety, and code quality from PR review
- **crawler:** Address Copilot PR review feedback across 6 issues
- **crawler:** Harden error handling across Phase 6 crawl features
- Fmt fix
- **export:** Improve type safety, error handling, and UI correctness
- **export:** Harden ATTACH DATABASE lifecycle and error handling
- **storage,rules,frontend:** Address PR review issues 5-16
- **crawler,storage:** Handle channel send errors and flush failures in crawl pipeline
- **storage,rules:** Propagate row deserialization errors and fix UTF-8 truncation panic
- Formatting

### Features

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
- **ui:** Implement Phase 4 frontend UI with results tables and page detail

### Miscellaneous

- Update CLAUDE.md
- Update CLAUDE.md files
- Update CLAUDE.md

### Refactoring

- **ai:** Improve type safety, JSON validation, and test coverage
- **rules:** Replace stringly-typed severity and category with enums

### Testing

- **crawler:** Add Phase 6 tests and fix sitemap discovery port handling

## [0.3.0] - 2026-04-04

### Features

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

