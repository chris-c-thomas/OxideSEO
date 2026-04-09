# <Project Name>

> One sentence describing what this is and who it's for. No marketing language.

<!-- Badges: only include badges that actually work. Delete this section if none apply. -->
<!-- [![CI](https://github.com/org/repo/actions/workflows/ci.yml/badge.svg)](https://github.com/org/repo/actions/workflows/ci.yml) -->
<!-- [![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE) -->

## Overview

Two to four paragraphs describing what the project does in mechanical terms. Cover:

- What problem it solves
- The high-level approach
- What's notable or different about the implementation
- Current status (production, beta, experimental)

No marketing language. Be concrete.

## Features

- Concrete capability 1, present tense
- Concrete capability 2
- Concrete capability 3

Avoid vague claims. "Authenticates users via email/password and OAuth (Google, GitHub)" is good. "Powerful authentication system" is not.

## Quickstart

The minimum commands to get the project running locally. Must work copy-paste on a clean machine that meets the requirements below.

```bash
git clone <repo-url>
cd <repo-name>
<install-command>
cp .env.example .env
# edit .env to fill in required values
<dev-command>
```

The app should now be available at <http://localhost:3000>.

## Requirements

- <Language> <version> (`.tool-versions` / `.nvmrc` / `engines` field)
- <Package manager> <version>
- <Database> <version> (if applicable)
- <Other system deps>

## Installation

Step-by-step installation, expanding on Quickstart. Include:

1. Clone the repository
2. Install dependencies
3. Set up environment variables (link to env section below)
4. Initialize the database (if applicable)
5. Run any required code generation
6. Start the dev server

## Usage

The 80% case for users of this project. For a library, this is import + minimal example. For an application, this is how to run the dev server and reach the first meaningful page or endpoint.

```ts
// Example code
```

For deeper usage, see [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

## Configuration

Required environment variables:

| Variable            | Required | Default | Description                        |
| ------------------- | -------- | ------- | ---------------------------------- |
| `DATABASE_URL`      | Yes      | —       | Postgres connection string         |
| `NEXTAUTH_SECRET`   | Yes      | —       | Secret used to sign session tokens |
| `STRIPE_SECRET_KEY` | Yes      | —       | Stripe API key for billing         |

Optional environment variables are documented in [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md#environment-variables).

## Architecture

One paragraph summary. Covers the runtime topology in plain language: what processes run, what data store(s) are used, what external services are integrated.

For full architecture documentation including request lifecycle, data model, and integration details, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Project Structure

```
.
├── app/                # Next.js app router routes
├── components/         # Shared React components
├── lib/                # Business logic and utilities
│   ├── db/             # Database client and queries
│   ├── auth/           # Authentication helpers
│   └── integrations/   # External service clients
├── prisma/             # Database schema and migrations
├── public/             # Static assets
├── docs/               # Project documentation
└── tests/              # Test suites
```

## Scripts

| Script             | Purpose                                              |
| ------------------ | ---------------------------------------------------- |
| `pnpm dev`         | Start the dev server with HMR                        |
| `pnpm build`       | Build the production bundle                          |
| `pnpm start`       | Start the production server (requires `build` first) |
| `pnpm test`        | Run the test suite                                   |
| `pnpm lint`        | Run ESLint                                           |
| `pnpm typecheck`   | Run TypeScript in --noEmit mode                      |
| `pnpm db:migrate`  | Apply pending Prisma migrations                      |
| `pnpm db:generate` | Regenerate the Prisma client                         |

## Testing

```bash
pnpm test           # Run all tests
pnpm test:watch     # Watch mode
pnpm test:e2e       # End-to-end tests
```

For detailed testing guidance, see [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md#testing).

## Deployment

Brief deployment summary (one paragraph). For details, see [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md).

## Contributing

Contributions are welcome. See [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) for the contribution process, code style, and PR requirements.

## License

<License name>. See [LICENSE](LICENSE).
