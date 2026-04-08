# README Quality Checklist

This is a hard gate. Run through every item against the generated README before declaring the documentation pass complete. If any item fails, fix it before presenting to the user.

## Structure

- [ ] Title is the project name (not a tagline)
- [ ] First line after title is a one-sentence value proposition
- [ ] Sections appear in this order: Overview, Features, Quickstart, Requirements, Installation, Usage, Configuration, Architecture, Project Structure, Scripts, Testing, Deployment, Contributing, License
- [ ] Sections that don't apply to the project are omitted (not left empty)
- [ ] Total length is between 200 and 800 lines

## Quickstart Section

- [ ] Quickstart appears within the first 50 lines of the README
- [ ] Every command is in a code block with a language tag
- [ ] Commands are copy-paste runnable on a clean machine that meets the Requirements
- [ ] No prerequisites are assumed beyond what's listed in Requirements
- [ ] The final command produces a working application or visible output
- [ ] Local URL or expected output is shown

## Requirements Section

- [ ] Every required tool is listed with a version
- [ ] Versions match what's actually declared in the repo (`.nvmrc`, `engines`, `.tool-versions`, etc.)
- [ ] System dependencies (databases, brokers, etc.) are listed
- [ ] Optional tools are clearly marked as optional

## Configuration Section

- [ ] Every required environment variable is in the table
- [ ] Every variable in the table is actually referenced in the code
- [ ] Each variable has a description that matches its actual use
- [ ] Default values are shown where they exist
- [ ] The table includes a "Required?" column

## Scripts Section

- [ ] Every script in `package.json` (or equivalent) is listed
- [ ] Each script has a one-line purpose description
- [ ] No fabricated scripts that don't exist in the manifest

## Project Structure Section

- [ ] Tree shows only top 2 levels
- [ ] Every directory shown actually exists
- [ ] Each directory has a one-line annotation
- [ ] Generated/build directories are excluded (`node_modules`, `dist`, `.next`, etc.)

## Architecture Section

- [ ] Contains a one-paragraph summary, not the full architecture
- [ ] Links to `docs/ARCHITECTURE.md` for details

## Style

- [ ] No emojis anywhere
- [ ] No banned marketing words: blazing, lightning-fast, revolutionary, powerful, seamless, simply, just, easily, robust, cutting-edge, world-class, best-in-class
- [ ] Present tense, active voice throughout
- [ ] No vague claims ("handles X well", "provides good Y")
- [ ] Code blocks have language tags
- [ ] Tables used for structured data with 3+ rows
- [ ] Consistent capitalization and punctuation

## Verification

- [ ] Every file path mentioned exists in the repo
- [ ] Every command mentioned references a real script or binary
- [ ] Every env var mentioned is referenced in the code
- [ ] Every package mentioned is in the dependency manifest
- [ ] No `TODO(verify)` markers remain unresolved (or they're explicitly flagged to the user)

## Links

- [ ] All internal links use relative paths
- [ ] All internal links resolve to files that exist
- [ ] External links are HTTPS
- [ ] Badges (if present) point to URLs that actually return a badge image

## Final Check

- [ ] A new developer following only the README can get the app running
- [ ] A returning developer can find any common task without leaving the README (or can find a clear pointer to the right doc)
- [ ] The README does not duplicate content that lives in `docs/` — it links instead
