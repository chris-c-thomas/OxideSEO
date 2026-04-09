# Phase 1: Discovery

**Goal:** Build a complete, factual inventory of the repository before writing any documentation. This phase is read-only — do not modify any source files.

**Output:** `docs/_working/repo-inventory.md`

**Stop condition:** Present the inventory to the user and wait for confirmation before proceeding to Phase 2.

## Workflow

Work through each section below in order. Use `bash` (find, grep, cat, jq, ls) and the file viewer aggressively. Do not skip sections — even sections that turn up empty are useful signal (e.g., "no test runner detected" tells the user something).

For each section, write findings directly into `docs/_working/repo-inventory.md` as you go. Use the structure shown below.

## Inventory Structure

````markdown
# Repository Inventory

Generated: <date>
Branch: <branch>
Commit: <short sha>

## 1. Project Identity

- Name (from package.json/Cargo.toml/pyproject.toml/go.mod)
- Version
- License
- Declared description
- Repository URL
- Homepage / docs URL (if any)

## 2. Stack and Runtime

- Language(s) and version(s)
- Runtime (Node version from .nvmrc/.tool-versions/engines, Python version, etc.)
- Package manager (pnpm/npm/yarn/uv/poetry/cargo) — detected from lockfile
- Framework(s) (Next.js, Express, Hono, Django, etc.) with version
- Build tooling (tsup, vite, webpack, turbopack, esbuild, cargo, etc.)
- Monorepo tooling (turborepo, nx, lerna, pnpm workspaces) — if applicable

## 3. Workspace Topology (monorepos only)

- Workspace root config
- List of packages/apps with their purpose (one line each)
- Dependency relationships between workspaces

## 4. Entry Points

- `main`, `module`, `exports`, `bin` from package.json
- Server entry files (e.g., `src/server.ts`, `app/api/*`)
- CLI entry files
- Next.js app/ or pages/ routes (top-level only)
- Background workers, cron jobs

## 5. Directory Tree (annotated, 2 levels)

A tree -L 2 -I 'node_modules|dist|.next|.git|coverage' output, with one-line annotations on each top-level directory.

## 6. Scripts Inventory

Table of every package.json script (and equivalents for other package managers):

| Script | Command  | Inferred Purpose |
| ------ | -------- | ---------------- |
| dev    | next dev | Local dev server |
| ...    | ...      | ...              |

## 7. Environment Variables

Grep the codebase for env var references and list every one found:

```bash
# Recommended search:
rg -o 'process\.env\.[A-Z_]+' --no-filename | sort -u
rg -o 'env::var\("[A-Z_]+"\)' --no-filename | sort -u
rg -o 'os\.environ\[?\.get\(?"[A-Z_]+"' --no-filename | sort -u
```
````

| Variable     | Files Referenced | Required? | Default | Inferred Purpose           |
| ------------ | ---------------- | --------- | ------- | -------------------------- |
| DATABASE_URL | src/db.ts        | Yes       | -       | Postgres connection string |
| ...          | ...              | ...       | ...     | ...                        |

Cross-reference against `.env.example`, `.env.template`, or similar — flag any variables in code but not in example, and vice versa.

## 8. Configuration Surface

- Config files (`tsconfig.json`, `next.config.js`, `vite.config.ts`, etc.)
- Feature flags (search for common patterns)
- Runtime config files

## 9. Dependencies (Top-Level Only)

Group production dependencies by purpose. Skip transitive deps. Format:

### Framework

- next@15.x — React framework
- react@19.x

### Data

- prisma@x — ORM
- ...

### Auth

- ...

### UI

- ...

### Build / Dev

- ...

### Testing

- ...

## 10. External Services and Integrations

Anything that calls out to a third party. Detect by:

- Imported SDK packages (@aws-sdk/\*, stripe, twilio, openai, etc.)
- HTTP client calls to known service URLs (grep for `https://api.`)
- Webhook handlers
- Database connections (Postgres, Redis, etc.)

| Service | How It's Used      | Where (file)       |
| ------- | ------------------ | ------------------ |
| Stripe  | Payment processing | src/lib/billing.ts |
| ...     | ...                | ...                |

## 11. Data Layer

- Database(s) in use
- ORM/query builder
- Schema location
- Migration tool and migration directory
- Seed data location

## 12. Authentication and Authorization

- Auth library (NextAuth, Lucia, Clerk, custom, etc.)
- Session model (JWT, cookie, etc.)
- Authorization patterns (RBAC, ABAC, route guards)
- Where auth config lives

## 13. Testing

- Test runner (vitest, jest, pytest, cargo test, etc.)
- Test file locations and conventions
- Coverage tool and current coverage (if accessible)
- E2E framework (Playwright, Cypress, etc.) — if any
- Fixtures location

## 14. CI/CD

- CI provider (GitHub Actions, GitLab CI, CircleCI, etc.)
- Workflow files and what each does
- Deployment target(s) (Vercel, Fly, AWS, self-hosted)
- Branch protection / required checks (if visible)

## 15. Containerization and Local Services

- Dockerfile(s) — what each builds
- docker-compose.yml — what services it provides
- Devcontainer config — if any

## 16. Observability

- Logging library
- Metrics / tracing (OpenTelemetry, Datadog, Sentry, etc.)
- Health check endpoints

## 17. Existing Documentation

- Current README.md — summarize and note what's stale or wrong
- Any files in docs/, doc/, documentation/
- Inline JSDoc / TSDoc / docstrings — sample density
- README.md files in subdirectories

## 18. Git Signal

- Recent commit messages (last 20)
- Active branches
- Number of contributors (`git shortlog -sn | wc -l`)
- First and last commit dates

## 19. Notable Files in Root

Any file in the root directory not yet covered above. This often surfaces things like `.editorconfig`, `Makefile`, `Justfile`, custom scripts, etc.

## 20. Anomalies and Red Flags

Anything that looks unusual, broken, or contradictory:

- Stale lockfiles
- Dependencies imported but not declared
- Dead code that looks intentional vs. accidental
- TODO/FIXME density (rough count)
- Files in the README that don't exist
- Scripts that reference removed files

````

## Useful Commands

```bash
# Project identity
cat package.json | jq '{name, version, license, description, repository, homepage}'

# Detect package manager
ls -1 | grep -E '(pnpm-lock|yarn.lock|package-lock|Cargo.lock|uv.lock|poetry.lock|go.sum)'

# Annotated tree
tree -L 2 -I 'node_modules|dist|.next|.git|coverage|target|__pycache__'

# All env vars in TypeScript/JavaScript
rg -o 'process\.env\.[A-Z_][A-Z0-9_]*' --no-filename | sort -u

# All scripts
cat package.json | jq -r '.scripts | to_entries[] | "\(.key): \(.value)"'

# Top-level dependencies grouped
cat package.json | jq '{dependencies, devDependencies}'

# Find Dockerfiles
find . -name 'Dockerfile*' -not -path '*/node_modules/*'

# Find CI workflows
ls -la .github/workflows/ 2>/dev/null || ls -la .gitlab-ci.yml 2>/dev/null

# Recent commits
git log --oneline -20

# Contributor count
git shortlog -sn | wc -l
````

## Completion Criteria

Phase 1 is complete when:

- [ ] Every section above has been filled in (or explicitly marked "N/A" with a one-line reason)
- [ ] Every env var reference in the code has been catalogued
- [ ] Every package.json script has an inferred purpose
- [ ] Anomalies and red flags section is populated (even if empty, state "no anomalies found")
- [ ] The inventory file has been written to `docs/_working/repo-inventory.md`

## Stop and Present

After writing the inventory, present a summary to the user in the following form:

> Phase 1 (Discovery) is complete. The full inventory is at `docs/_working/repo-inventory.md`.
>
> **Key findings:**
>
> - Stack: [one line]
> - [3-4 bullets of the most important / surprising findings]
>
> **Anomalies to flag:**
>
> - [Any red flags, or "none"]
>
> Please review the inventory and confirm before I proceed to Phase 2 (Architecture Mapping). If anything is wrong or missing, let me know and I'll correct it before moving on.

**Do not proceed to Phase 2 until the user confirms.**
