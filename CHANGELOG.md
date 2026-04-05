# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

