# Reference: React Projects

Stack-specific guidance for documenting React applications and component libraries. Read this in addition to the TypeScript reference (most React projects are TypeScript) when the target repo is React-based.

## Detection

The repo is a React project if:
- `react` is in `package.json` dependencies
- `.tsx` or `.jsx` files are present in significant numbers
- A React-based framework is in use (Next.js, Remix, Vite + React, Astro, etc.)

Determine the **shape** of the React project — this drives documentation structure:
- **Standalone app** (Vite, CRA, custom): SPA, document like a normal app
- **Next.js app**: Has its own conventions (see below)
- **Remix app**: Has its own conventions
- **Component library**: Document like a Rust library — public API surface matters more than runtime
- **Monorepo with multiple React surfaces**: Document each app separately

## Phase 1 Additions: Discovery

### React Version and Mode
```bash
jq '.dependencies.react, .dependencies["react-dom"]' package.json
```

Capture:
- React major version (17, 18, 19) — affects available features
- Whether the app uses Server Components (Next.js 13+ App Router, or experimental setups)
- Concurrent features in use (`useTransition`, `useDeferredValue`, Suspense for data)

### Framework Detection
| Signal | Framework |
|---|---|
| `next` in deps + `app/` or `pages/` directory | Next.js |
| `@remix-run/*` in deps | Remix |
| `vite` + `@vitejs/plugin-react` | Vite |
| `react-scripts` | Create React App (legacy) |
| `astro` + React integration | Astro |
| `gatsby` | Gatsby |
| `@tanstack/react-start` or `@tanstack/start` | TanStack Start |

Note: Tauri apps use one of the above as the frontend.

### Routing
Determine the routing mechanism:
- File-system routing (Next.js App Router, Next.js Pages Router, Remix, TanStack Router file routes)
- Code-based routing (`react-router-dom`, TanStack Router code routes)
- No routing (single-page tools)

```bash
# Next.js App Router
find app -name 'page.tsx' -o -name 'route.ts' 2>/dev/null

# Next.js Pages Router
find pages -name '*.tsx' -not -name '_*' 2>/dev/null

# Remix
find app/routes -name '*.tsx' 2>/dev/null

# react-router
rg 'createBrowserRouter|<Routes>' src/
```

### State Management
Detect the state strategy. Check `package.json` and grep for usage:

| Library | Pattern to grep |
|---|---|
| Redux Toolkit | `@reduxjs/toolkit`, `createSlice` |
| Zustand | `zustand`, `create<` |
| Jotai | `jotai`, `atom(` |
| Recoil | `recoil`, `atom(` |
| TanStack Query | `@tanstack/react-query`, `useQuery` |
| SWR | `swr`, `useSWR` |
| MobX | `mobx`, `observer(` |
| Apollo Client | `@apollo/client`, `useQuery` |
| URQL | `urql`, `useQuery` |
| Context only | `createContext`, no external store |

Most apps use a combination — e.g., TanStack Query for server state + Zustand for client state. Document both explicitly.

### Styling
| Library | Detection |
|---|---|
| Tailwind | `tailwindcss` in deps, `tailwind.config.*` |
| CSS Modules | `*.module.css` files |
| Styled Components | `styled-components` in deps |
| Emotion | `@emotion/react` in deps |
| Vanilla Extract | `@vanilla-extract/css` |
| Stitches | `@stitches/react` |
| shadcn/ui | `components/ui/` directory + Tailwind + Radix |
| Plain CSS | `*.css` imports without modules |

### Component Libraries
| Library | Detection |
|---|---|
| shadcn/ui | `components/ui/` + Radix primitives |
| Radix UI | `@radix-ui/*` |
| Headless UI | `@headlessui/react` |
| Material UI | `@mui/material` |
| Chakra UI | `@chakra-ui/react` |
| Mantine | `@mantine/core` |
| Ant Design | `antd` |
| Park UI | `@park-ui/*` |

Note: shadcn/ui is special — components are vendored into the project, not installed. Look in `components/ui/` for evidence.

### Data Fetching
- TanStack Query (configuration in `lib/query-client.ts` or similar)
- SWR
- Native `fetch` in components or hooks
- Server Components (Next.js App Router) — fetched at render time
- tRPC (`@trpc/*`)

### Forms
- React Hook Form (`react-hook-form`)
- Formik (`formik`)
- TanStack Form (`@tanstack/react-form`)
- Conform (`@conform-to/react`)
- Native form elements only

### Validation
- Zod (`zod`) — most common in modern React projects
- Yup (`yup`)
- Valibot (`valibot`)
- Joi (`joi`)

### Testing
- Vitest (`vitest`) — increasingly the default
- Jest (`jest`)
- React Testing Library (`@testing-library/react`)
- Playwright (`@playwright/test`) for E2E
- Cypress (`cypress`) for E2E
- Storybook (`@storybook/*`) for component dev

## Phase 2 Additions: Architecture

### Component Architecture
Document the project's component organization. Common patterns:

**Atomic structure:**
```
components/
├── atoms/
├── molecules/
├── organisms/
├── templates/
└── pages/
```

**Feature-based:**
```
features/
├── auth/
│   ├── components/
│   ├── hooks/
│   └── api.ts
├── dashboard/
└── settings/
```

**Flat with shadcn pattern:**
```
components/
├── ui/             # shadcn primitives
├── layout/
└── (feature components at root)
```

Document which one is in use, with examples.

### Server vs. Client Components (Next.js App Router)
For App Router projects, the boundary is critical:

```bash
# Find client components
rg "^['\"]use client['\"]" --no-filename | wc -l

# Find server components (everything else in app/)
find app -name 'page.tsx' -o -name 'layout.tsx' | xargs grep -L "use client"
```

Document the project's default and exceptions:
- Default: server components
- Client components used for: forms, interactive UI, browser APIs, third-party libraries that need client context

### State Layers
Most non-trivial React apps have multiple state layers. Document each:

| Layer | Tool | Lifetime | Used For |
|---|---|---|---|
| Server cache | TanStack Query | Until invalidated | API data |
| URL state | Next.js router / nuqs | Page lifetime | Filters, pagination, dialog open state |
| Global client state | Zustand | App lifetime | User preferences, theme |
| Local component state | useState | Component lifetime | Form inputs, UI toggles |
| Form state | React Hook Form | Form lifetime | Form values and validation |

### Hooks Architecture
Custom hooks are the React equivalent of services. Document:
- Where shared hooks live (`hooks/`, `lib/hooks/`, etc.)
- Naming convention (`use-*` vs `use*`)
- The few "load-bearing" hooks that everything depends on (e.g., `useAuth`, `useUser`)

### Boundaries
- **Server boundary**: Where server-only code lives (`server/`, `lib/server/`) and how it's enforced (`server-only` package)
- **Client boundary**: Where client-only code lives and how client-only deps are kept out of server bundles
- **Public API** (component libraries): What's exported from `src/index.ts`

## Phase 3 Additions: Synthesis

### README for an Application

Adapt the standard template. Add or emphasize:
- A screenshot or GIF (note: don't generate one — flag for the user to add)
- Browser support statement
- Whether it's a SPA, SSR, or static
- Bundle size if non-trivial (use `next build` output or bundle analyzer)

### README for a Component Library

Different structure, closer to a Rust library:

1. Title + one-sentence purpose
2. Demo / Storybook link
3. Installation
4. Quick example (one component, ~15 lines)
5. Components list with brief descriptions
6. Theming approach
7. Browser support
8. Peer dependency requirements (React version, etc.)
9. Bundle size
10. License

```tsx
import { Button } from 'your-library'

export function App() {
  return <Button variant="primary">Click me</Button>
}
```

### Component Inventory (for libraries)

| Component | Purpose | Status |
|---|---|---|
| `Button` | Standard button with variants | Stable |
| `Dialog` | Modal dialog | Stable |
| `DataTable` | Sortable, filterable table | Beta |

### Routes Table (for apps with file-system routing)

For Next.js App Router:

| Route | File | Auth | Purpose |
|---|---|---|---|
| `/` | `app/page.tsx` | Public | Landing page |
| `/dashboard` | `app/(app)/dashboard/page.tsx` | Required | User dashboard |
| `/api/auth/[...nextauth]` | `app/api/auth/[...nextauth]/route.ts` | N/A | NextAuth handler |

Document route groups (`(group)`), parallel routes (`@slot`), and intercepting routes (`(.)`) explicitly — they confuse new contributors.

### Performance Considerations Section

For non-trivial apps, include in ARCHITECTURE.md:
- Code splitting strategy (`dynamic`, route-level, component-level)
- Image optimization (`next/image`, custom)
- Font loading (`next/font`, system fonts)
- Streaming and Suspense boundaries
- Cache strategy (Next.js fetch cache, TanStack Query cache, browser cache)

## Phase 4 Additions: Verification

### React-Specific Checks

```bash
# Verify documented routes exist (Next.js App Router)
find app -name 'page.tsx' -o -name 'route.ts' | sort

# Verify documented components exist
rg -l "export (default )?function \w+" components/

# Check for "use client" / "use server" misuse
rg -l '"use client"' app/  # Server components shouldn't have this

# Verify documented hooks exist
rg "^export function use\w+" -t ts hooks/ lib/hooks/ 2>/dev/null

# Verify Tailwind config matches documented design tokens
cat tailwind.config.* 2>/dev/null
```

### Bundle and Build Verification

```bash
# For Next.js
pnpm build  # Note the route summary at the end

# For Vite
pnpm build  # Note the chunk sizes
```

If the build output mentions routes or chunks not in the docs, flag drift.

## Common Footguns to Document

These belong in the "Hidden Coupling and Footguns" section:

- **`"use client"` is viral** — A server component cannot import a client component's children's children if any link in the chain is a client component without crossing a serialization boundary
- **`async` server components** — Cannot be used inside client components except via children prop
- **Hydration mismatches** — Any non-deterministic render output (Date.now(), Math.random(), localStorage access) causes hydration errors
- **Effect dependencies** — `useEffect` dependency arrays must include every reactive value used inside; ESLint rule `react-hooks/exhaustive-deps` is critical
- **Stale closures** — Event handlers and effects capture values at definition time; refs are needed to escape this
- **Strict mode double-renders** — In dev, components mount/unmount/mount; effects run twice. Cleanup must be idempotent.
- **Suspense + data fetching** — Suspense boundaries swallow errors; need `ErrorBoundary` companion
- **TanStack Query keys** — Cache keys must be serializable and stable; objects must be referentially stable or memoized
- **Server actions and form state** — Form state resets on action submission unless explicitly preserved
- **Image domains** (Next.js) — Remote images require `next.config.js` `images.remotePatterns` configuration
- **Environment variable scoping** — `NEXT_PUBLIC_*` prefix for client-exposed vars; everything else is server-only and will be `undefined` in client bundles
- **CSS layer ordering** (Tailwind) — `@layer` directives matter; custom CSS that doesn't use layers can lose to or override Tailwind unexpectedly
- **Server component cache** (Next.js 14+) — `fetch` is cached by default; opt-out via `cache: 'no-store'` or `revalidate`

## Useful Commands Quick Reference

```bash
# Bundle analysis (Next.js)
ANALYZE=true pnpm build  # requires @next/bundle-analyzer

# Bundle analysis (Vite)
pnpm build && pnpm vite-bundle-visualizer

# Type check without build
pnpm tsc --noEmit

# React DevTools (browser extension, no command)

# Storybook
pnpm storybook
pnpm build-storybook

# Component generation (shadcn)
pnpm dlx shadcn@latest add button
```

## Documentation Conventions to Honor

- **Storybook is documentation.** If Storybook is set up, link to it from the README. Don't duplicate component examples that already live in stories.
- **TypeScript types are documentation.** For component libraries, the exported prop types are part of the public API. Generate API docs from them (typedoc, react-docgen) rather than hand-writing them.
- **Browser support matters.** State it explicitly. Modern projects assume evergreen browsers; document if you support more.
- **Server Component boundaries deserve a diagram.** For Next.js App Router projects, a small Mermaid diagram of the component tree showing the server/client split is high-value.
- **Performance budgets** (if any) belong in ARCHITECTURE.md or a dedicated PERFORMANCE.md.
