# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for OxideSEO. ADRs document significant technical decisions, their context, and their consequences.

## Index

| ADR | Title | Status | Date |
|---|---|---|---|
| [0001](0001-record-architecture-decisions.md) | Record architecture decisions | Accepted | 2026-04-07 |
| [0002](0002-channel-based-crawl-engine.md) | Channel-based crawl engine with dedicated storage writer | Accepted | 2026-04-07 |
| [0003](0003-wasm-plugin-sandboxing.md) | WASM Component Model for plugin sandboxing | Accepted | 2026-04-07 |

## Creating a New ADR

1. Copy the template: `cp docs/adr/0001-record-architecture-decisions.md docs/adr/NNNN-short-title.md`
2. Fill in the context, decision, and consequences
3. Add the entry to the index table above
4. Submit as part of the PR that implements the decision
