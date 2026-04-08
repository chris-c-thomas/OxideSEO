# Deployment

This document describes how the project is built, deployed, and operated in production.

For local development, see [DEVELOPMENT.md](DEVELOPMENT.md).
For architecture, see [ARCHITECTURE.md](ARCHITECTURE.md).

## Environments

| Environment | URL | Branch | Auto-Deploy |
|---|---|---|---|
| Production | <https://example.com> | `main` | Yes |
| Staging | <https://staging.example.com> | `staging` | Yes |
| Preview | per-PR URLs | PR branches | Yes |

## Hosting

| Component | Provider | Notes |
|---|---|---|
| Web app | Vercel | Auto-deploys on push |
| Database | Supabase (Postgres) | Managed, daily backups |
| Cache | Upstash (Redis) | Serverless |
| File storage | Cloudflare R2 | S3-compatible |
| Queue | Inngest | Hosted |

## Build Process

The build is triggered by:
- Push to `main` → production deploy
- Push to `staging` → staging deploy
- Pull request → preview deploy

Build steps (defined in `<ci-config-path>`):
1. Install dependencies
2. Run linter and type checker
3. Run test suite
4. Generate Prisma client
5. Build the Next.js application
6. Upload artifacts

If any step fails, the deploy is aborted.

## Required Secrets

The following secrets must be configured in the deployment platform for each environment:

| Secret | Where | Purpose |
|---|---|---|
| `DATABASE_URL` | Vercel env vars | Database connection |
| `NEXTAUTH_SECRET` | Vercel env vars | Session signing |
| `STRIPE_SECRET_KEY` | Vercel env vars | Stripe API |
| `STRIPE_WEBHOOK_SECRET` | Vercel env vars | Webhook signature verification |
| `OPENAI_API_KEY` | Vercel env vars | LLM features |
| `SENTRY_DSN` | Vercel env vars | Error tracking |

Secrets are managed via `<tooling>` and rotated `<frequency>`.

## Database Migrations

Migrations run automatically as part of the deploy via `pnpm db:migrate:deploy`.

### Manual Migration

If a migration must be run manually:

```bash
pnpm db:migrate:deploy
```

### Rolling Back a Migration

Prisma does not support automatic rollback. To revert a schema change:
1. Create a new migration that reverses the previous change
2. Deploy as normal

## Deployment Process

### Standard Deploy

1. Open a PR against `main`
2. CI runs all checks
3. Reviewer approves
4. Merge to `main`
5. Vercel triggers production deploy automatically
6. Health checks run (see Monitoring section)
7. If health checks pass, the deploy is promoted to live

### Hotfix

1. Branch from `main` as `fix/<description>`
2. Make the minimal change
3. Open PR with `hotfix` label
4. Expedited review
5. Merge → auto-deploy

### Manual Deploy

```bash
# Trigger a deploy without a code change
vercel --prod
```

## Rollback

### Vercel Rollback

1. Go to the Vercel project dashboard
2. Find the previous successful deployment
3. Click "Promote to Production"

### Database Rollback

If a deploy includes a destructive migration that needs reverting:
1. Roll back the application first (see above)
2. Apply a corrective migration as a hotfix
3. Do not manually edit production data unless absolutely necessary

## Monitoring

| Concern | Tool | Where |
|---|---|---|
| Errors | Sentry | <https://sentry.io/...> |
| Logs | Vercel logs | Vercel dashboard |
| Uptime | <service> | <link> |
| Performance | Vercel Analytics | Vercel dashboard |

### Health Checks

The app exposes a health check at `/api/health` which:
- Verifies database connectivity
- Verifies cache connectivity
- Returns 200 OK if all checks pass

This endpoint is hit by `<monitoring tool>` every <interval>.

### Alerts

Alerts are configured for:
- Error rate spike
- 5xx response rate above threshold
- Database connection failures
- Health check failures

Alert routing: <where alerts go — Slack channel, PagerDuty, etc.>

## Common Operations

### Tail Production Logs

```bash
vercel logs --prod --follow
```

### Connect to Production Database

```bash
# Use the read-replica URL for safety
psql $DATABASE_REPLICA_URL
```

For destructive operations, use the primary URL and require sign-off.

### Run a One-Off Script in Production

```bash
# Example: backfill script
vercel env pull .env.production.local
NODE_ENV=production node scripts/backfill.js
```

## Incident Response

For incidents, see the runbook at `<location>` (or note that no formal runbook exists yet).

### Quick Reference

| Symptom | First Steps |
|---|---|
| Site is down | Check Vercel dashboard, check Sentry, check `/api/health` |
| Database errors | Check Supabase status, check connection pool metrics |
| Slow responses | Check Vercel Analytics, check Sentry transactions |
| Webhook failures | Check Stripe webhook logs, check `/api/webhooks/stripe` logs |

## Cost Monitoring

| Service | Monthly Cost | Notes |
|---|---|---|
| Vercel | $X | Pro plan |
| Supabase | $X | Pro plan |
| Upstash | $X | Pay-per-request |

Review monthly. Set billing alerts on each provider.
