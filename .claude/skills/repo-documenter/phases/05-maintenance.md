# Phase 5: Maintenance

**Goal:** Detect documentation drift over time and keep the doc set in sync with the codebase as it evolves. This phase is run periodically (manually or in CI), not as part of the initial pipeline.

**Output:** `docs/_working/drift-report.md` and a set of targeted fixes.

**When to run:**
- After a major refactor or feature merge
- On a schedule (monthly or quarterly)
- As part of CI on PRs that touch high-signal files
- Before a release

## Why This Phase Exists

Documentation rots. Phases 1-4 produce a verified doc set at a single point in time, but every commit after that is a chance for the docs to drift from reality. Phase 5 is the recurring sweep that catches drift early, before the docs become stale enough to require another full pass.

The work here is intentionally narrower than Phases 1-4. This is a diff, not a rewrite.

## Workflow

Maintenance is a three-step loop: **detect drift → assess severity → apply targeted fixes**. Run each step in order. Do not jump to fixes before assessing severity, and do not assess severity before running the full detection sweep.

### Step 1: Detect Drift

Run the drift detection checks below. Write findings to `docs/_working/drift-report.md` as you go. Do not fix anything yet — just catalogue.

### Step 2: Assess Severity

Categorize each finding:
- **Critical** — README Quickstart is broken, a documented command no longer exists, a required env var is missing from docs
- **Important** — A documented feature has changed shape, a script's behavior has changed, an integration was added or removed
- **Minor** — Cosmetic, version bumps, dependency list staleness
- **Informational** — Things to flag but not necessarily fix (e.g., new contributor in `git shortlog` not yet credited)

### Step 3: Apply Targeted Fixes

Fix critical and important items. For minor items, batch them. For informational items, just note them in the drift report.

If the drift is severe enough that more than ~30% of the docs need updating, do not patch — recommend re-running the full pipeline (Phases 1-4) instead.

## Drift Detection Checks

### Check 1: Script Drift

Compare scripts in `package.json` (or equivalent) against scripts mentioned in README and DEVELOPMENT.md.

```bash
# Scripts currently in the manifest
cat package.json | jq -r '.scripts | keys[]' | sort > /tmp/manifest-scripts.txt

# Scripts mentioned in docs
rg -o '(pnpm|npm run|yarn) [a-z:_-]+' README.md docs/ | awk '{print $2}' | sort -u > /tmp/doc-scripts.txt

# What's in docs but not in manifest (broken docs)
comm -23 /tmp/doc-scripts.txt /tmp/manifest-scripts.txt

# What's in manifest but not in docs (undocumented scripts)
comm -13 /tmp/doc-scripts.txt /tmp/manifest-scripts.txt
```

### Check 2: Environment Variable Drift

Compare env vars in code against env vars documented in README and DEVELOPMENT.md.

```bash
# Env vars in code
rg -o 'process\.env\.[A-Z_][A-Z0-9_]*' --no-filename | sed 's/process.env.//' | sort -u > /tmp/code-env.txt

# Env vars in docs (rough heuristic — adjust per project)
rg -o '`[A-Z][A-Z0-9_]{2,}`' README.md docs/ | tr -d '`' | sort -u > /tmp/doc-env.txt

# In code but not documented
comm -23 /tmp/code-env.txt /tmp/doc-env.txt

# In docs but not in code (stale)
comm -13 /tmp/code-env.txt /tmp/doc-env.txt
```

Also cross-check `.env.example` against the code reference list — they should match exactly.

### Check 3: File Path Drift

Verify every file path mentioned in the docs still exists.

```bash
# Extract file paths from docs
rg -o '`[a-zA-Z0-9_/.-]+\.(ts|tsx|js|jsx|json|md|yaml|yml|toml|rs|py|go|prisma)`' README.md docs/ \
  | awk -F: '{print $NF}' | tr -d '`' | sort -u > /tmp/doc-paths.txt

# Check each
while read path; do
  test -e "$path" || echo "MISSING: $path"
done < /tmp/doc-paths.txt
```

### Check 4: Dependency Drift

Compare top-level dependencies in the manifest against those mentioned in the architecture docs.

```bash
# Current top-level deps
cat package.json | jq -r '.dependencies + .devDependencies | keys[]' | sort > /tmp/current-deps.txt

# Deps mentioned in architecture docs
rg -o '`[a-z0-9@/_-]+`' docs/ARCHITECTURE.md | tr -d '`' | sort -u > /tmp/doc-deps.txt
```

Look for major version bumps that change behavior (e.g., Next.js 14 → 15, React 18 → 19) and verify the architecture doc still describes the system accurately.

### Check 5: Route Drift (web apps)

Compare actual routes against routes mentioned in docs.

```bash
# Next.js app router
find app -name 'page.tsx' -o -name 'route.ts' | sed 's|app||;s|/page.tsx||;s|/route.ts||' | sort > /tmp/current-routes.txt
```

Look for new routes that aren't in docs and documented routes that no longer exist.

### Check 6: Schema Drift

For projects with a database schema:

```bash
# Last-modified time of schema vs. ARCHITECTURE.md
stat -c '%Y %n' prisma/schema.prisma docs/ARCHITECTURE.md 2>/dev/null
```

If the schema is newer than the architecture doc, the data model section is suspect. Diff against the last documented state if possible (look for schema entities mentioned in `architecture-notes.md` and check whether they still exist).

### Check 7: CI Workflow Drift

Compare CI workflow files against what's documented in CONTRIBUTING.md and DEPLOYMENT.md.

```bash
# Currently configured CI jobs
find .github/workflows -name '*.yml' -o -name '*.yaml' 2>/dev/null
```

Check for new required checks, removed steps, or changed deployment targets.

### Check 8: Quickstart Smoke Test

The most important check. Mentally walk through the README Quickstart one more time:
1. Are the prerequisites still accurate?
2. Does the install command still work?
3. Does the env file template match what the code expects?
4. Does the dev server command still exist?
5. Is there a new required setup step missing from the Quickstart?

If any answer is no, that's a critical drift.

### Check 9: Open Questions Recheck

Re-read `docs/_working/architecture-notes.md` Open Questions section. Have any been answered by recent commits? Have any new questions emerged?

### Check 10: TODO(verify) Sweep

Search the docs for any unresolved verification markers from previous runs.

```bash
rg 'TODO\(verify\)' README.md docs/
```

Each one is a chance to either verify and remove the marker, or finally cut the unverifiable claim.

## Drift Report Structure

```markdown
# Drift Report

Generated: <date>
Last documentation pass: <date>
Commits since last pass: <N>

## Summary
- Critical issues: N
- Important issues: N
- Minor issues: N
- Informational: N

## Critical Issues

### 1. README Quickstart broken
- File: README.md, Quickstart section
- Issue: `pnpm db:setup` no longer exists; renamed to `pnpm db:init` in commit abc123
- Fix: Update line 47 of README.md

### 2. ...

## Important Issues

### 1. New environment variable not documented
- Variable: `INNGEST_SIGNING_KEY`
- Added in: commit def456
- Fix: Add to README env table and DEVELOPMENT.md env section

## Minor Issues
- 3 stale dependency versions in ARCHITECTURE.md table
- ...

## Informational
- 2 new contributors since last pass
- ...

## Recommendation
<patch | full re-run | no action needed>
```

## When to Patch vs. Re-Run

| Situation | Action |
|---|---|
| 0-5 drift items, all minor | Skip — note in report |
| 5-15 drift items | Patch in place |
| 15+ drift items, or any structural changes | Recommend full Phase 1-4 re-run |
| Major refactor with new architecture | Recommend full Phase 1-4 re-run |
| New top-level subsystem added | Patch ARCHITECTURE.md, recommend ADR |

## CI Integration

To make this skill cheap to run continuously, the detection checks can be turned into a CI script. A minimal example:

```bash
#!/usr/bin/env bash
# scripts/check-docs.sh
# Exits non-zero if docs drift is detected

set -e

# Check 1: scripts referenced in docs that don't exist
manifest_scripts=$(jq -r '.scripts | keys[]' package.json | sort)
doc_scripts=$(rg -o '(pnpm|npm run) [a-z:_-]+' README.md docs/ | awk '{print $2}' | sort -u)

missing=$(comm -23 <(echo "$doc_scripts") <(echo "$manifest_scripts"))
if [ -n "$missing" ]; then
  echo "Scripts referenced in docs but not in package.json:"
  echo "$missing"
  exit 1
fi

# Check 2: env vars in docs but not in code
# ... etc
```

Run on PRs that touch `package.json`, `.env.example`, schema files, or any file under `docs/`.

## Completion Criteria

Phase 5 is complete when:

- [ ] All drift detection checks have run
- [ ] Findings are catalogued in `docs/_working/drift-report.md`
- [ ] Each finding has a severity assignment
- [ ] Critical and important items have been fixed (or escalated to a full re-run)
- [ ] The drift report includes a clear recommendation

## Stop and Present

> Phase 5 (Maintenance) complete. The full drift report is at `docs/_working/drift-report.md`.
>
> **Drift summary since last pass (<date>):**
> - Critical: N (fixed)
> - Important: N (fixed)
> - Minor: N (batched)
> - Informational: N (noted)
>
> **Recommendation:** [patch applied | full re-run recommended | no action needed]
>
> **Files modified in this pass:**
> - [list]
>
> Please review the changes and the drift report before committing.
