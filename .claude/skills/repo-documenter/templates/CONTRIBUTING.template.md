# Contributing

Thank you for considering a contribution to this project. This document describes the contribution process and expectations.

## Before You Start

- Check existing issues and PRs to avoid duplicate work
- For non-trivial changes, open an issue first to discuss the approach
- For bug fixes, include a reproduction case in the issue or PR description

## Development Setup

See [DEVELOPMENT.md](DEVELOPMENT.md) for local environment setup.

## Branch Naming

Branches should follow this pattern:

- `feat/<short-description>` for new features
- `fix/<short-description>` for bug fixes
- `docs/<short-description>` for documentation changes
- `chore/<short-description>` for tooling, dependencies, refactors
- `test/<short-description>` for test-only changes

## Commit Messages

This project follows [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <short summary>

<optional body>

<optional footer>
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `perf`, `build`, `ci`

Examples:

```
feat(auth): add OAuth login via GitHub
fix(api): handle null user in session check
docs(readme): update quickstart for Node 20
```

## Pull Request Process

1. Fork the repo and create a branch from `main`
2. Make your changes following the code style guidelines below
3. Add or update tests for any behavioral changes
4. Update documentation if your change affects user-facing behavior
5. Ensure all CI checks pass locally:
   ```bash
   pnpm check
   pnpm test
   ```
6. Open a PR against `main` with:
   - A clear description of what changed and why
   - Link to the related issue (if any)
   - Screenshots for UI changes
   - Notes on any breaking changes or migration steps

### Required Checks

Every PR must pass the following before merge:

- Linting (`pnpm lint`)
- Type checking (`pnpm typecheck`)
- Unit and integration tests (`pnpm test`)
- E2E tests (if applicable)
- At least one approving review

## Code Style

### TypeScript

- Strict mode is required — no `any`, no implicit returns
- Prefer explicit return types on exported functions
- Use Zod for runtime validation at trust boundaries
- Avoid default exports except where required by the framework

### React

- Server components by default; use client components only when needed
- Colocate components with their styles and tests
- Follow the existing component patterns in `components/`

### File Organization

- Keep files focused. Split when a file exceeds ~300 lines or covers multiple concerns
- Group by feature, not by file type
- See [ARCHITECTURE.md](ARCHITECTURE.md#module-boundaries) for layer rules

### Imports

- Use absolute imports via the configured path alias (e.g., `@/lib/...`)
- Group imports: external, internal, types, styles
- Let the linter handle import ordering

## Testing Expectations

- Bug fixes should include a regression test
- New features should include unit tests for logic and integration tests for endpoints
- Aim for behavioral coverage, not line coverage
- See [DEVELOPMENT.md#testing](DEVELOPMENT.md#testing) for how to run and write tests

## Documentation

- User-facing changes require README or docs updates
- New environment variables must be added to `.env.example`, `lib/env.ts`, and the env table in [DEVELOPMENT.md](DEVELOPMENT.md#environment-variables)
- Architectural changes should be recorded as an ADR in `docs/adr/`
- Inline comments should explain _why_, not _what_

## Reporting Bugs

Open an issue with:

- A clear, descriptive title
- Steps to reproduce
- Expected vs. actual behavior
- Environment details (OS, Node version, browser if applicable)
- Relevant logs or error messages
- A minimal reproduction if possible

## Suggesting Features

Open an issue describing:

- The problem you're trying to solve
- Your proposed solution
- Alternatives you considered
- Any prior art or relevant references

## Code of Conduct

By participating in this project, you agree to abide by the project's Code of Conduct. See [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md) if present.

## License

By contributing, you agree that your contributions will be licensed under the project's license.
