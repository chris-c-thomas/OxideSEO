# Contributing

Contributions to OxideSEO are welcome. This document describes the contribution process and expectations.

For local development setup, see [DEVELOPMENT.md](DEVELOPMENT.md).
For architecture details, see [ARCHITECTURE.md](ARCHITECTURE.md).

## Before You Start

- Check [existing issues](https://github.com/chris-c-thomas/OxideSEO/issues) and PRs to avoid duplicate work
- For non-trivial changes, open an issue first to discuss the approach
- For bug fixes, include steps to reproduce in the issue or PR description

## Development Setup

See [DEVELOPMENT.md](DEVELOPMENT.md) for prerequisites, installation, and common workflows.

## Branch Naming

Create branches from `main` following this pattern:

| Prefix      | Use                       |
| ----------- | ------------------------- |
| `feat/`     | New features              |
| `fix/`      | Bug fixes                 |
| `refactor/` | Code restructuring        |
| `docs/`     | Documentation changes     |
| `test/`     | Test additions or changes |
| `chore/`    | Tooling, dependencies, CI |
| `perf/`     | Performance improvements  |

## Commit Messages

This project follows [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <short summary>
```

**Types:** `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `perf`, `ci`

**Scopes:** `crawler`, `rules`, `storage`, `frontend`, `ipc`, `ci`, `export`, `ai`, `plugin`

Examples:

```
feat(crawler): add streaming body read with blake3 hashing
fix(rules): handle missing viewport meta on frameset pages
refactor(storage): extract query builder for paginated results
test(frontier): add property-based tests for dedup consistency
```

The changelog is generated from these messages via [git-cliff](https://git-cliff.org/). Maintainers generate the changelog; contributors do not need git-cliff installed.

## Pull Request Process

1. Create a feature branch from `main`
2. Make your changes following the code style guidelines below
3. Add or update tests for behavioral changes
4. Ensure all checks pass locally:

```bash
# Frontend
npm run lint
npm run format:check
npm run typecheck
npm run test

# Rust (from src-tauri/)
cd src-tauri
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

5. Open a PR against `main` with:
   - A clear description of what changed and why
   - Link to the related issue (if any)
   - Screenshots for UI changes

### Required CI Checks

Every PR must pass before merge:

| Check              | Command                                                    | Runs On                |
| ------------------ | ---------------------------------------------------------- | ---------------------- |
| Rust format        | `cargo fmt --all -- --check`                               | Ubuntu, macOS, Windows |
| Rust lint          | `cargo clippy --all-targets --all-features -- -D warnings` | Ubuntu, macOS, Windows |
| Rust tests         | `cargo test --lib --all-features`                          | Ubuntu, macOS, Windows |
| Frontend lint      | `npm run lint`                                             | Ubuntu                 |
| Frontend format    | `npm run format:check`                                     | Ubuntu                 |
| Frontend typecheck | `npm run typecheck`                                        | Ubuntu                 |
| Frontend tests     | `npm run test`                                             | Ubuntu                 |

## Code Style

### Rust

- All code must pass `cargo fmt` and `cargo clippy` with zero warnings
- Use `anyhow::Result<T>` with `.context()` for error propagation in internal code
- Return `Result<T, String>` at the Tauri command boundary (map with `.map_err(|e| format!("{e:#}"))`)
- No `unwrap()`, `expect()`, or `panic!()` outside of tests
- Use `///` doc comments on public items
- Import order: `std` -> external crates -> `crate::` internal imports, separated by blank lines

### TypeScript

- All code must pass ESLint (zero warnings) and Prettier
- No `any` type. Use `unknown` and narrow with type guards.
- Use function declarations for components: `export function MyComponent()`, not arrow assignments
- Use `cn()` from `lib/utils.ts` for conditional Tailwind class merging

### IPC Types

Types that cross the Tauri IPC boundary must be updated in both languages simultaneously:

- Rust: struct with `#[serde(rename_all = "camelCase")]`
- TypeScript: interface in `src/types/index.ts`

### Styling

- Use Tailwind utility classes exclusively
- Use CSS custom properties (`var(--color-*)`) for theme-aware colors
- Use shadcn/ui primitives for standard UI patterns

## Testing Expectations

- Bug fixes should include a regression test
- New rules should include tests with HTML fixture `ParsedPage` structs
- New commands should include tests for the Rust handler
- Aim for behavioral coverage, not line coverage

## Reporting Bugs

Open an issue with:

- Steps to reproduce
- Expected vs. actual behavior
- OS and app version
- Relevant error messages or screenshots

## Suggesting Features

Open an issue describing:

- The problem you are trying to solve
- Your proposed solution
- Alternatives you considered

## License

By contributing, you agree that your contributions will be licensed under the project's dual MIT / Apache 2.0 license.
