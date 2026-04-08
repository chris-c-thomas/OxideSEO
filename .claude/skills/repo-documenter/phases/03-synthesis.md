# Phase 3: Synthesis

**Goal:** Generate the actual user-facing documentation set from the verified inventory and architecture notes.

**Output:** README.md (root), docs/ARCHITECTURE.md, docs/DEVELOPMENT.md, docs/CONTRIBUTING.md, docs/DEPLOYMENT.md (conditional), docs/adr/0001-record-architecture-decisions.md

**Prerequisite:** Phase 1 inventory + Phase 2 architecture notes are complete, user-confirmed, and all blocking Open Questions have been answered.

## Workflow

For each output file, do the following in order:

1. Read the corresponding template from `templates/`.
2. Read the relevant sections of `repo-inventory.md` and `architecture-notes.md`.
3. Adapt the template to the specific project. **Do not blindly copy.** Templates are starting points — delete sections that don't apply, expand sections that do.
4. For every factual claim in the generated doc, mentally trace it back to a specific file or command in the repo. If you cannot, omit it or mark it `TODO(verify)`.
5. Write the file.

## Generation Order

Generate in this order — later files reference earlier ones:

1. `docs/ARCHITECTURE.md` first (from `architecture-notes.md`)
2. `docs/DEVELOPMENT.md` second (from inventory sections 2, 6, 7, 13, 15)
3. `docs/CONTRIBUTING.md` third
4. `docs/DEPLOYMENT.md` fourth (only if CI/CD or container config exists)
5. `docs/adr/0001-record-architecture-decisions.md` — bootstrap ADR
6. `README.md` last — it links to all the others

## File-by-File Guidance

### docs/ARCHITECTURE.md

Source: `architecture-notes.md` plus inventory sections 4, 8, 10, 11, 12.

This is the deepest doc. It should let a senior engineer understand the system in 30 minutes of reading. Include:
- The mechanical system overview (no marketing)
- A Mermaid diagram of runtime topology (5-10 nodes)
- Request lifecycle walkthrough
- Data model summary (link to schema file, don't reproduce it)
- External integrations table
- Auth model
- Hidden coupling / footguns section (high-leverage for new contributors)

Length: 400-1000 lines is reasonable for a non-trivial app. Don't pad.

### docs/DEVELOPMENT.md

Source: inventory sections 2, 6, 7, 13, 15, 16.

This is the doc a new developer reads on day one. Include:
- Prerequisites (exact versions: Node, package manager, system deps)
- Clone + install steps (verbatim runnable commands)
- Environment setup (every required env var with description)
- How to start the dev server
- How to run tests
- How to run linters / formatters / type checks
- Local services setup (docker-compose up, etc.)
- How to seed the database
- Common dev workflows (creating a migration, generating types, etc.)
- Troubleshooting section for common errors

Every command must work copy-paste against the actual repo.

### docs/CONTRIBUTING.md

Source: inventory section 14, 18 + project conventions inferred from existing code.

Include:
- How to propose a change (issue first? PR directly?)
- Branch naming convention (infer from `git branch -a`)
- Commit message convention (infer from `git log`)
- PR process and required checks (from CI config)
- Code style (linter/formatter config)
- Testing expectations
- How to add a changeset / changelog entry (if applicable)
- Code of conduct link

Be honest. If the project has no formal contribution process, say so and propose a minimal one rather than fabricating one.

### docs/DEPLOYMENT.md

**Only generate this file if there is real deployment infrastructure** (CI workflow, Dockerfile, vercel.json, fly.toml, k8s manifests, etc.). Otherwise, skip it and link to a `TODO` from the README.

Include:
- Where it deploys (which provider, which environments)
- How a deploy is triggered
- Environment promotion flow
- Required secrets and where they're configured
- Rollback procedure
- Monitoring / alerts
- Runbook for common incidents (if any)

### docs/adr/0001-record-architecture-decisions.md

Bootstrap ADR. Use `templates/adr-template.md`. The first ADR is always "we will use ADRs to record architecture decisions." This sets the precedent. Optionally create 2-3 retroactive ADRs for the most important decisions you can identify from the code (e.g., "0002-use-prisma-for-data-access", "0003-monorepo-with-pnpm-workspaces"). Only do this if the user agrees — it's high-leverage but takes time.

Also create `docs/adr/README.md` as the index.

### README.md

Source: everything. This is the router.

Use `templates/README.template.md` as the base. The structure is:

1. Title + one-sentence value prop
2. Badges (if applicable — only include badges that actually work)
3. Overview (2-4 paragraphs, mechanical)
4. Features (concrete bullets, present tense)
5. Quickstart (minimum commands to get running)
6. Requirements (exact versions)
7. Installation (step by step)
8. Usage (the 80% case)
9. Configuration (env var table)
10. Architecture (one paragraph + link to docs/ARCHITECTURE.md)
11. Project Structure (annotated tree, top 2 levels)
12. Scripts (table)
13. Testing (how to run + link to docs/DEVELOPMENT.md)
14. Deployment (one paragraph + link if applicable)
15. Contributing (link)
16. License

Critical README rules:
- The Quickstart must work copy-paste. Every single line.
- The env var table must contain every required var. Optional vars can be omitted from the README and listed in DEVELOPMENT.md.
- The script table must list every npm script (or workspace equivalent), not a curated subset.
- Length target: 300-600 lines. If it's longer, you're putting things in the README that should be in deeper docs.

## CHANGELOG.md

If `CHANGELOG.md` does not exist, scaffold one using the [Keep a Changelog](https://keepachangelog.com/) format. Don't try to reconstruct history from git log — just create the scaffold with an `[Unreleased]` section and a note that the changelog starts from this point.

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

_Changelog tracking begins from this point. For history before this, see git log._
```

## Style Rules (Non-Negotiable)

These apply to every file generated in this phase:

- **No emojis.** Anywhere.
- **No marketing language.** Banned words: blazing, lightning-fast, revolutionary, powerful, seamless, simply, just, easily, robust, cutting-edge, world-class, best-in-class, modern (as a compliment).
- **Present tense, active voice.** "The server validates the request" not "The request will be validated by the server."
- **Concrete, not abstract.** "Returns a 401 if the session cookie is missing" not "handles authentication errors."
- **Tables for structured data.** Any list of 3+ items with multiple attributes becomes a table.
- **Code blocks with language tags.** Always.
- **Link, don't duplicate.** If something is in DEVELOPMENT.md, the README links to it instead of repeating it.
- **Mark unverified claims.** `TODO(verify): ...` is acceptable as a placeholder. Silent guesses are not.

## Completion Criteria

Phase 3 is complete when:

- [ ] All required files have been generated
- [ ] Every file passes the style rules above
- [ ] No marketing language anywhere
- [ ] No emojis anywhere
- [ ] Every code example has been mentally traced to the actual repo
- [ ] Every env var, script, and file path mentioned exists in the inventory

Do not present to the user yet — proceed directly to Phase 4 (Verification). Verification is part of the same delivery.
