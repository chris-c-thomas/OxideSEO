# Phase 4: Verification

**Goal:** Cross-check every factual claim in the generated documentation against the actual codebase. This is the phase that prevents AI-hallucinated documentation.

**Output:** `docs/_working/verification-report.md` and a corrected set of documentation files.

**Stop condition:** Present the verification report and final docs to the user.

**Prerequisite:** Phase 3 has produced a complete draft of all docs.

## Why This Phase Exists

The single biggest failure mode of AI-generated documentation is plausible-sounding claims about features, files, scripts, env vars, or commands that don't actually exist in the repository. The previous phases reduce this risk but don't eliminate it. This phase is the explicit gate that catches what slipped through.

Treat this phase as adversarial review against your own work. Be suspicious of every claim.

## Workflow

For each generated file (README.md, ARCHITECTURE.md, DEVELOPMENT.md, CONTRIBUTING.md, DEPLOYMENT.md, ADRs), do the following:

1. Re-read the file from disk (do not rely on memory of what you wrote).
2. Extract every verifiable claim into a checklist.
3. Verify each claim against the actual codebase using `bash`/grep/cat.
4. For each failed verification, fix the source doc immediately.
5. Record the result in `verification-report.md`.

## Verification Categories

### 1. File Path Claims

Every file or directory path mentioned in the docs must exist.

```bash
# Extract file paths from docs (rough heuristic)
rg -o '`[a-zA-Z0-9_/.-]+\.(ts|tsx|js|jsx|json|md|yaml|yml|toml|rs|py|go)`' docs/ README.md

# For each, verify it exists
test -e <path> && echo "OK" || echo "MISSING: <path>"
```

### 2. Command Claims

Every shell command in the docs must be runnable. You don't need to actually execute everything (some commands have side effects), but verify:

- Every `pnpm <script>` / `npm run <script>` / `yarn <script>` references a real script in package.json
- Every binary referenced (e.g., `prisma`, `next`, `eslint`) is in dependencies or devDependencies
- Every file referenced in a command exists

```bash
# Extract scripts mentioned in docs
rg -o '(pnpm|npm run|yarn) [a-z:_-]+' docs/ README.md | sort -u

# Cross-check against package.json
cat package.json | jq -r '.scripts | keys[]'
```

### 3. Environment Variable Claims

Every env var mentioned in the docs must:
- Exist in the actual code (grep for `process.env.X` or equivalent)
- Match the description (does the description match how it's actually used?)

```bash
# Extract env var names from docs
rg -o '\b[A-Z][A-Z0-9_]{2,}\b' docs/ README.md | sort -u

# Cross-check against code
rg -o 'process\.env\.[A-Z_][A-Z0-9_]*' --no-filename | sort -u
```

### 4. Dependency Claims

Every package or library named in the docs must be in package.json (or equivalent manifest).

```bash
# For each package mentioned in ARCHITECTURE.md, verify it's a real dependency
cat package.json | jq -r '.dependencies + .devDependencies | keys[]'
```

### 5. Endpoint / Route Claims

For web apps, every URL path or API endpoint mentioned in the docs must correspond to a real route file.

```bash
# Next.js app router
find app -name 'page.tsx' -o -name 'route.ts'

# Express-style
rg -o '(app|router)\.(get|post|put|delete|patch)\(["'\''][^"'\'']+["'\'']' -t ts
```

### 6. Configuration Claims

Every config file referenced (`tsconfig.json`, `next.config.js`, etc.) must exist and contain what the docs say it contains.

### 7. Architecture Claims

These are the hardest to verify mechanically. For each non-trivial architectural claim, identify the file(s) that substantiate it. If you can't, mark the claim as unverified and either remove it or replace it with `TODO(verify)`.

Examples:
- "The system uses Prisma for data access" → verify `prisma` is a dependency and `schema.prisma` exists
- "Authentication is handled by NextAuth" → verify `next-auth` is a dependency and there's an auth config file
- "Background jobs run on Inngest" → verify `inngest` is a dependency and there are job definitions

### 8. Quickstart Walkthrough

The README's Quickstart section is the highest-stakes section in the entire doc set. Mentally simulate a new developer following it line by line:

1. Does step 1 produce the state step 2 expects?
2. Are there missing prerequisites that would cause step N to fail?
3. Does the final step produce a working application?

If you have any uncertainty about any step, flag it.

## Verification Report Structure

```markdown
# Verification Report

Generated: <date>

## Summary
- Total claims checked: N
- Verified: N
- Fixed (was wrong, now corrected): N
- Marked TODO (cannot verify): N

## File-by-File Results

### README.md
| Claim | Type | Status | Source/Note |
|---|---|---|---|
| `pnpm dev` starts the dev server | command | OK | package.json scripts.dev |
| `DATABASE_URL` is required | env | OK | src/db.ts:3 |
| Uses Prisma 6.x | dependency | FIXED | Was 5.x in docs, actual is 6.1.0 |
| Has a /admin dashboard | route | TODO | No app/admin route found, asked user |
| ... | ... | ... | ... |

### docs/ARCHITECTURE.md
| Claim | Type | Status | Source/Note |
| ... | ... | ... | ... |

### docs/DEVELOPMENT.md
...

## Unresolved Issues

List any claims that could not be verified and could not be safely removed. The user must decide what to do with these.

1. README mentions an "/admin" dashboard but no admin routes found in code. User input needed.
2. ...

## Quickstart Simulation

Step-by-step trace of the README Quickstart, with notes on any potential failure points:

1. `git clone <url>` — OK
2. `pnpm install` — OK (pnpm is the declared package manager)
3. `cp .env.example .env` — OK, .env.example exists with N variables
4. `pnpm db:migrate` — POTENTIAL ISSUE: requires DATABASE_URL to be set first; should add explicit step
5. ...
```

## Useful Verification Commands

```bash
# Verify every file path mentioned in a doc exists
for f in $(rg -o '`[a-zA-Z0-9_/.-]+\.(ts|tsx|js|json)`' README.md | tr -d '`'); do
  test -e "$f" && echo "OK: $f" || echo "MISSING: $f"
done

# Verify every npm script mentioned
rg -o '(pnpm|npm run) [a-z:_-]+' README.md docs/ | awk '{print $2}' | sort -u | while read script; do
  jq -e ".scripts.\"$script\"" package.json > /dev/null && echo "OK: $script" || echo "MISSING: $script"
done

# Find docs mentions of env vars vs. code mentions
diff <(rg -o '\b[A-Z][A-Z0-9_]{2,}\b' README.md docs/ | awk -F: '{print $NF}' | sort -u) \
     <(rg -o 'process\.env\.[A-Z_][A-Z0-9_]*' --no-filename | sed 's/process.env.//' | sort -u)
```

## Completion Criteria

Phase 4 is complete when:

- [ ] Every doc file has been re-read from disk
- [ ] Every file path claim verified
- [ ] Every command claim verified
- [ ] Every env var claim verified
- [ ] Every dependency claim verified
- [ ] Quickstart has been mentally simulated end-to-end
- [ ] All discovered errors have been fixed in the source docs
- [ ] All unverifiable claims have been removed or marked `TODO(verify)`
- [ ] Verification report written to `docs/_working/verification-report.md`

## Final Quality Gate

Run through `checklists/readme-quality.md` against the README. This is a hard gate — every item must pass.

## Present to User

After verification, present the final result:

> Documentation pass complete. Here's what was generated:
>
> **Files created/modified:**
> - README.md (rewritten)
> - docs/ARCHITECTURE.md (new)
> - docs/DEVELOPMENT.md (new)
> - docs/CONTRIBUTING.md (new)
> - docs/DEPLOYMENT.md (new) [if applicable]
> - docs/adr/0001-record-architecture-decisions.md (new)
> - CHANGELOG.md (scaffolded) [if applicable]
>
> **Verification summary:**
> - N total claims checked
> - N verified, N fixed, N marked TODO
>
> **Unresolved items needing your attention:**
> [List from verification-report.md]
>
> **Recommended next steps:**
> 1. Review the docs in your editor
> 2. Resolve the TODO items
> 3. Run the Quickstart end-to-end on a clean checkout to confirm it works
> 4. Commit and open a PR from the docs/repo-documenter-pass branch
>
> The full verification report is at `docs/_working/verification-report.md`. The working files (`docs/_working/`) can be committed for future reference or added to .gitignore.
