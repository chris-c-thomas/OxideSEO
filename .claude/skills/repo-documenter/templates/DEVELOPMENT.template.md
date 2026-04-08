# Development

This document covers everything needed to develop the project locally: prerequisites, setup, common workflows, and troubleshooting.

For the project overview, see [README.md](../README.md).
For architecture details, see [ARCHITECTURE.md](ARCHITECTURE.md).

## Prerequisites

| Tool | Version | Notes |
|---|---|---|
| Node.js | 20.x | See `.nvmrc` |
| pnpm | 9.x | Install via `npm install -g pnpm` or corepack |
| Postgres | 16.x | Local install or Docker |
| Docker | 24+ | Required for local services |

Optional tooling:
- `direnv` for automatic env loading
- `mkcert` for local HTTPS

## Initial Setup

```bash
# Clone the repository
git clone <repo-url>
cd <repo-name>

# Install dependencies
pnpm install

# Set up environment variables
cp .env.example .env
# Edit .env with your local values (see Environment Variables section)

# Start local services (Postgres, Redis, etc.)
docker compose up -d

# Run database migrations
pnpm db:migrate

# Generate Prisma client (or equivalent)
pnpm db:generate

# Seed the database with initial data
pnpm db:seed

# Start the dev server
pnpm dev
```

The app should now be running at <http://localhost:3000>.

## Environment Variables

### Required

| Variable | Description | Example |
|---|---|---|
| `DATABASE_URL` | Postgres connection string | `postgresql://user:pass@localhost:5432/dbname` |
| `NEXTAUTH_SECRET` | Session signing secret | Generate with `openssl rand -base64 32` |
| `NEXTAUTH_URL` | Canonical app URL | `http://localhost:3000` |

### Optional

| Variable | Default | Description |
|---|---|---|
| `LOG_LEVEL` | `info` | Logger verbosity (`debug`, `info`, `warn`, `error`) |
| `RATE_LIMIT_ENABLED` | `true` | Toggle rate limiting in development |

### Service Credentials

| Variable | Required In | Description |
|---|---|---|
| `STRIPE_SECRET_KEY` | Production | Stripe API secret |
| `STRIPE_WEBHOOK_SECRET` | Production | For verifying Stripe webhooks |
| `OPENAI_API_KEY` | Production | LLM features |

## Common Workflows

### Creating a Database Migration

```bash
# After editing schema.prisma:
pnpm db:migrate-dev --name <descriptive-name>

# This generates a migration file and applies it to your local DB.
# Commit both the migration file and updated schema.
```

### Adding a New Dependency

```bash
pnpm add <package>           # Production dependency
pnpm add -D <package>        # Dev dependency
pnpm add -F <workspace> <pkg> # Add to a specific workspace package
```

### Regenerating Generated Code

```bash
pnpm db:generate     # Prisma client
pnpm openapi:generate # OpenAPI types (if applicable)
pnpm icons:generate   # Icon sprite (if applicable)
```

### Adding a New Route

1. Create the route file in `app/<path>/page.tsx`
2. If protected, ensure the path is matched by middleware
3. If it fetches data, add a server component or use a server action
4. Add the route to the navigation if applicable

### Adding a New Environment Variable

1. Add it to `.env.example` with a placeholder
2. Add it to `lib/env.ts` with a Zod validator
3. Document it in this file (table above)
4. If required in production, add it to the deployment platform secrets

## Code Quality

```bash
pnpm lint            # Run ESLint
pnpm lint:fix        # Fix auto-fixable issues
pnpm typecheck       # Run TypeScript in --noEmit mode
pnpm format          # Run Prettier
pnpm format:check    # Check formatting without modifying
```

All checks run in CI on every PR. To run them all locally before pushing:

```bash
pnpm check
```

## Testing

### Test Layout

- Unit tests: colocated as `*.test.ts` next to source files
- Integration tests: `tests/integration/`
- E2E tests: `tests/e2e/`
- Fixtures: `tests/fixtures/`

### Running Tests

```bash
pnpm test                  # Run all unit + integration tests
pnpm test:watch            # Watch mode
pnpm test:coverage         # With coverage report
pnpm test:e2e              # End-to-end tests (requires app running)
pnpm test <pattern>        # Run tests matching a pattern
```

### Writing Tests

- Use the existing test utilities in `tests/helpers/`
- For database-dependent tests, use the test database fixtures
- For API tests, use the in-memory test server in `tests/helpers/server.ts`

## Debugging

### Server-Side

- Set `LOG_LEVEL=debug` in `.env` for verbose logs
- Use the Node inspector: `pnpm dev:inspect` then attach via Chrome or VS Code
- Check the logs at `<location>` if applicable

### Client-Side

- React DevTools (browser extension)
- Network tab for inspecting API calls
- `localStorage.debug = '*'` for debug logs (if applicable)

## Local Services

The `docker-compose.yml` provides local Postgres, Redis, and any other services required.

```bash
docker compose up -d         # Start all services in background
docker compose ps            # Check status
docker compose logs -f       # Tail logs
docker compose down          # Stop all services
docker compose down -v       # Stop and remove volumes (full reset)
```

## Troubleshooting

### "Cannot find module" after pulling latest

```bash
pnpm install
pnpm db:generate
```

### Database connection refused

Check that Postgres is running:
```bash
docker compose ps
docker compose logs postgres
```

### Port already in use

```bash
lsof -i :3000
kill -9 <pid>
```

### Prisma client out of date

```bash
pnpm db:generate
```

### Type errors after pulling latest

```bash
rm -rf node_modules/.cache
pnpm install
pnpm db:generate
pnpm typecheck
```

## Editor Setup

### VS Code

Recommended extensions are listed in `.vscode/extensions.json`. The project includes workspace settings in `.vscode/settings.json`.

### Other Editors

- Use the project's `.editorconfig` for basic formatting
- Configure your editor to use the local Prettier and ESLint installations
- TypeScript should use the workspace version, not the editor's bundled version
