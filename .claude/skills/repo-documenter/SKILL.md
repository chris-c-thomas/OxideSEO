---
name: repo-documenter
description: Generate or maintain a comprehensive, verified documentation set (README.md, ARCHITECTURE.md, DEVELOPMENT.md, CONTRIBUTING.md, DEPLOYMENT.md, ADRs) for an existing code repository. Use this skill whenever the user wants to document a repo, write or rewrite a README, generate project docs from scratch, modernize stale documentation, onboard a legacy codebase, produce architecture documentation for an existing project, or check existing documentation for drift against the codebase — even when they don't explicitly say "use the repo-documenter skill". Trigger on phrases like "document this repo", "write a README for this project", "generate docs", "this repo has no docs", "create architecture docs", "the README is outdated", "check if the docs are still accurate", or "update the docs after my refactor".
---

# Repo Documenter

Generate a comprehensive, accurate, and professional documentation set for an existing repository through a four-phase pipeline: **Discovery → Architecture → Synthesis → Verification**. The pipeline is designed to prevent the most common failure mode of AI-generated documentation: plausible-sounding claims about features, files, or commands that don't actually exist.

## Core Principles

1. **Verify before writing.** Every factual claim in the final docs must be traceable to a real file, line, or command in the repo. If it can't be verified, omit it or mark it `TODO`.
2. **Discovery before synthesis.** Do not write a single line of user-facing documentation until the discovery and architecture phases are complete and the user has confirmed the findings.
3. **Router, not encyclopedia.** The README's job is to get a new developer productive in 10 minutes and route them to deeper docs for everything else.
4. **Stop between phases.** Do not chain phases without explicit user confirmation. The user needs to validate Phase 1 and answer the Open Questions in Phase 2 before Phase 3 can produce accurate output.
5. **No marketing language, no emojis, present tense, active voice.**

## Workflow

The skill is organized into four phases, each with its own detailed instructions in `phases/`. Read the relevant phase file at the start of that phase rather than trying to hold the entire workflow in context at once.

| Phase | File | Purpose | Stops for user? |
|---|---|---|---|
| 1 | `phases/01-discovery.md` | Build a factual inventory of the repo | Yes — show inventory before proceeding |
| 2 | `phases/02-architecture.md` | Map system structure and surface Open Questions | Yes — get answers before proceeding |
| 3 | `phases/03-synthesis.md` | Generate the actual user-facing docs | No — proceed to verification |
| 4 | `phases/04-verification.md` | Cross-check every claim against the codebase | Yes — present report and final docs |
| 5 | `phases/05-maintenance.md` | Detect and fix doc drift on subsequent runs | Yes — present drift report |

Phase 5 is **not** part of the initial pipeline. It is run separately on subsequent invocations against an already-documented repo to catch drift. The first four phases are the initial pass; Phase 5 is the recurring sweep.

All scratch work goes in `docs/_working/`. Final docs go in the repo root and `docs/`.

## Initial Setup

Before starting Phase 1, do the following:

1. **Confirm the working directory** is the root of the target repo. Run `git rev-parse --show-toplevel` to verify.
2. **Check git state.** If the working tree is dirty, suggest the user commit or stash first. Then create a documentation branch:
   ```bash
   git checkout -b docs/repo-documenter-pass
   ```
3. **Create the working directory:**
   ```bash
   mkdir -p docs/_working docs/adr
   ```
4. **Read the existing README** (if any) to understand what's already claimed about the project. Note discrepancies as you discover them in Phase 1 — they often reveal what changed.
5. **Ask the user three calibration questions** before beginning:
   - What kind of project is this? (library, CLI, web app, service, monorepo, etc.)
   - Who is the primary audience for the docs? (external contributors, internal team, end users, all of the above)
   - Are there any areas of the codebase that are deprecated, experimental, or that should be excluded from documentation?

Once those are answered, begin Phase 1 by reading `phases/01-discovery.md`.

## Phase Transitions

Between each phase, present a clear summary and explicitly ask the user to confirm before proceeding. Example transition message after Phase 1:

> Phase 1 (Discovery) complete. I've written the inventory to `docs/_working/repo-inventory.md`. Here's a summary: [3-5 bullets]. Please review the inventory and let me know if anything looks wrong or missing. Once you confirm, I'll proceed to Phase 2 (Architecture Mapping).

## Output Structure

The final deliverable should look like this:

```
<repo root>/
├── README.md                          # Rewritten, structured per templates/README.template.md
├── CHANGELOG.md                       # Scaffolded if absent
├── docs/
│   ├── ARCHITECTURE.md
│   ├── DEVELOPMENT.md
│   ├── CONTRIBUTING.md
│   ├── DEPLOYMENT.md                  # Only if deploy config exists
│   ├── adr/
│   │   ├── README.md                  # ADR index
│   │   └── 0001-record-architecture-decisions.md
│   └── _working/                      # Scratch space — gitignore or commit per user preference
│       ├── repo-inventory.md
│       ├── architecture-notes.md
│       └── verification-report.md
```

## Templates

All output documents are based on templates in `templates/`. Read the relevant template at the start of Phase 3 and adapt it to the specific project — do not blindly copy. Templates are starting points, not rigid forms.

| Template | Used for |
|---|---|
| `templates/README.template.md` | The root README |
| `templates/ARCHITECTURE.template.md` | System overview and data flow |
| `templates/DEVELOPMENT.template.md` | Local setup and dev workflows |
| `templates/CONTRIBUTING.template.md` | Contribution process |
| `templates/DEPLOYMENT.template.md` | Build, release, environments |
| `templates/adr-template.md` | Individual ADR entries |

## Quality Checklist

Before declaring the work complete, run through `checklists/readme-quality.md` against the generated README. This is a hard gate — if any item fails, fix it before presenting to the user.

## Re-running the Skill

This skill is designed to be re-runnable after major refactors. When re-running:

1. Preserve any human edits to the existing docs by diffing first.
2. **Use Phase 5 (Maintenance) for routine updates.** It's a targeted drift sweep, not a full rewrite, and is the right tool 80% of the time.
3. **Use the full Phase 1-4 pipeline only when drift is severe** (>30% of docs need updating, or there's been a major architectural change). Phase 5 will recommend this when appropriate.
4. When re-running Phase 1-4, surface diffs against existing docs in Phase 3 rather than overwriting blindly. Ask the user which sections to regenerate vs. preserve.

## Stack-Specific References

When the target repo uses a known stack, read the corresponding reference file in `references/` at the start of Phase 1. Reference files contain stack-specific detection signals, additional inventory items, common footguns, and documentation conventions to honor.

| Stack | Reference File | When to Read |
|---|---|---|
| TypeScript | `references/typescript.md` | Any project with `tsconfig.json` |
| React | `references/react.md` | Any project with `react` in dependencies |
| Rust | `references/rust.md` | Any project with `Cargo.toml` |
| Tauri v2 | `references/tauri-v2.md` | Projects with `src-tauri/` and Tauri v2 |

Reference files are **additive**, not exclusive. A Tauri app written in React + TypeScript should load all four references. Read them once at the start of Phase 1 and apply their guidance throughout subsequent phases.

If the target repo uses a stack without a dedicated reference file, proceed with the phase files alone — they are framework-agnostic and will still work, just without stack-specific shortcuts.

## Constraints (Non-Negotiable)

- No emojis anywhere in generated docs.
- No marketing language ("blazing fast", "revolutionary", "powerful", "seamless", "simply").
- Present tense, active voice.
- Every code example must be copy-paste runnable against the actual repo.
- Every env var, script name, file path, and command mentioned must exist in the codebase.
- Tables for any structured data with 3+ rows (env vars, scripts, endpoints, etc.).
- Link between docs rather than duplicating content.
- If a claim cannot be verified, omit it or mark `TODO(verify)` — never guess.
