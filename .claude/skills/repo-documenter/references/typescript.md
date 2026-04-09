# Reference: TypeScript Projects

Stack-specific guidance for documenting TypeScript projects. Read this in addition to any framework-specific reference (React, Next.js, Tauri, etc.) when the target repo uses TypeScript.

## Detection

The repo is a TypeScript project if:

- `tsconfig.json` exists at the root or in workspace packages
- `typescript` is in `devDependencies`
- `.ts` or `.tsx` files dominate the source

Determine the **shape** of the TypeScript project:

- **Application** (private, deployed): Strictness is internal policy
- **Published library** (npm package): Strictness is part of the public contract
- **Monorepo** (multiple tsconfigs): Project references and path mapping matter
- **Type-only package**: `.d.ts` files, no runtime code

## Phase 1 Additions: Discovery

### TypeScript Version

```bash
jq '.devDependencies.typescript' package.json
```

Capture the major version (5.0+, 5.4+, 5.5+, 5.6+, 5.7+) — features and behavior differ meaningfully.

### tsconfig Analysis

Read every `tsconfig*.json` and capture:

```bash
find . -name 'tsconfig*.json' -not -path '*/node_modules/*'
```

For each:

- `extends` chain — what base config is in use (`@tsconfig/strictest`, `@tsconfig/node20`, custom)
- `compilerOptions.strict` — true means all strict flags on
- `compilerOptions.target` and `module` — output format
- `compilerOptions.moduleResolution` — `node`, `node16`, `nodenext`, `bundler`
- `compilerOptions.paths` — path aliases
- `compilerOptions.lib` — DOM, ES2022, etc.
- `references` — project reference graph (monorepos)
- `include` / `exclude` — which files are checked

The most important question: **is `noUncheckedIndexedAccess` enabled?** This single flag has more impact on code style than almost any other.

### Strictness Audit

Capture the actual strictness level explicitly. Run:

```bash
jq '.compilerOptions | {
  strict,
  noImplicitAny,
  strictNullChecks,
  strictFunctionTypes,
  strictBindCallApply,
  strictPropertyInitialization,
  noImplicitThis,
  alwaysStrict,
  noUncheckedIndexedAccess,
  noImplicitOverride,
  noPropertyAccessFromIndexSignature,
  exactOptionalPropertyTypes,
  verbatimModuleSyntax,
  isolatedModules
}' tsconfig.json
```

Document the chosen level. "Strict mode" means different things to different teams.

### Module System

Detect the module strategy:

- **CJS** (`"type": "commonjs"` or absent, `module: "commonjs"`)
- **ESM** (`"type": "module"`, `module: "esnext"`/`"node16"`/`"nodenext"`)
- **Dual** (separate builds via `exports` field)
- **Bundler** (`moduleResolution: "bundler"`, no Node compatibility)

```bash
jq '{type, exports, main, module, types}' package.json
```

The `exports` field is critical for libraries — document it explicitly.

### Build Tooling

| Tool                | Detection         | Purpose                                  |
| ------------------- | ----------------- | ---------------------------------------- |
| `tsc`               | Just `typescript` | Type checking, sometimes builds          |
| `tsup`              | `tsup` in deps    | Library bundler (esbuild-based)          |
| `unbuild`           | `unbuild` in deps | Library bundler (rollup-based)           |
| `tsx`               | `tsx` in deps     | Run TS files directly (replaces ts-node) |
| `ts-node`           | `ts-node` in deps | Run TS files (legacy)                    |
| `swc` / `@swc/core` | In deps           | Fast TS compilation                      |
| `esbuild`           | In deps           | Fast TS compilation/bundling             |
| `vite`              | In deps           | Dev server + bundler                     |
| `rollup`            | In deps           | Bundler (often for libraries)            |
| `tsc-alias`         | In deps           | Resolves path aliases in tsc output      |

Document which tool does what — it's common to have `tsc` for type checking + `tsup` for building + `tsx` for scripts.

### Monorepo Project References

For monorepos using TypeScript project references:

```bash
# Map the reference graph
find . -name 'tsconfig.json' -not -path '*/node_modules/*' -exec sh -c '
  echo "=== $1 ==="
  jq -r ".references[]?.path // empty" "$1"
' _ {} \;
```

Document the build order implied by references, and whether builds use `tsc -b` (build mode) or per-package builds.

### Runtime Validation

TypeScript only enforces types at compile time. Detect runtime validation libraries:

- **Zod** (`zod`) — most common
- **Valibot** (`valibot`) — newer, smaller
- **ArkType** (`arktype`) — newer, very fast
- **Effect Schema** (`@effect/schema`)
- **io-ts** (`io-ts`)
- **TypeBox** (`@sinclair/typebox`)
- **Yup** (`yup`)

Document which is used and where the trust boundaries are (API request parsing, env var parsing, config file parsing, third-party data).

### Linting and Formatting

| Tool                   | Detection                          |
| ---------------------- | ---------------------------------- |
| ESLint (legacy config) | `.eslintrc*`                       |
| ESLint (flat config)   | `eslint.config.js`/`.ts`/`.mjs`    |
| typescript-eslint      | `@typescript-eslint/*`             |
| Biome                  | `biome.json`                       |
| oxlint                 | `.oxlintrc.json`                   |
| Prettier               | `.prettierrc*`, `prettier` in deps |
| dprint                 | `dprint.json`                      |

Note: ESLint flat config is the modern default; legacy `.eslintrc` is deprecated.

### Test Framework

| Tool             | Detection                                    |
| ---------------- | -------------------------------------------- |
| Vitest           | `vitest` in deps, `vitest.config.*`          |
| Jest             | `jest` in deps                               |
| Node test runner | `node:test` imports, no other test framework |
| Bun test         | `bun` runtime + `bun:test` imports           |
| uvu              | `uvu` in deps                                |
| ava              | `ava` in deps                                |

### Type Generation Sources

Document any code that generates types (these are documentation maintenance hazards):

- Prisma (`prisma generate` → `@prisma/client`)
- Drizzle (`drizzle-kit` introspection)
- GraphQL (`graphql-codegen`)
- OpenAPI (`openapi-typescript`, `oazapfts`)
- tRPC (inferred from router, no codegen step)
- Protobuf (`@bufbuild/protoc-gen-es`)
- Database introspection (`kysely-codegen`)

## Phase 2 Additions: Architecture

### Type System Architecture

For non-trivial TypeScript projects, document:

- **Where types live** — Centralized in `types/`, colocated with code, or split by domain
- **Branded types** — If used (e.g., `type UserId = string & { __brand: 'UserId' }`)
- **Discriminated unions** — Major domain models that use them
- **Generic constraints** — Library code with non-trivial generics
- **Type-level programming** — Any heavy use of conditional types, mapped types, or template literal types deserves explicit documentation

### Validation Boundaries

For each external boundary, document the validation strategy:

| Boundary                  | Validation        | Schema Location         |
| ------------------------- | ----------------- | ----------------------- |
| HTTP request body         | Zod               | `lib/schemas/api.ts`    |
| Environment variables     | Zod               | `lib/env.ts`            |
| Config files              | Zod               | `lib/config.ts`         |
| Third-party API responses | Zod               | `lib/integrations/*.ts` |
| Database results          | Inferred from ORM | (none — trusted)        |

The pattern to surface: where TypeScript types are _trusted_ vs. _enforced_.

### Public API Surface (libraries)

For published libraries, the public API is everything exported from the entry point(s) declared in `package.json` `exports`. Document:

```bash
# Find the entry point(s)
jq '.exports' package.json

# List public exports
rg '^export ' src/index.ts
```

Anything reachable from these entry points is part of the public contract and is subject to semver. Anything not reachable is implementation detail.

### Path Aliases

If `compilerOptions.paths` is configured, document:

- The aliases (`@/*`, `~/*`, `@app/*`, etc.)
- Whether the bundler/runtime also resolves them (TypeScript paths alone do nothing at runtime; you need `tsc-alias`, a bundler, or runtime resolution)
- Any aliases that point to internal-only modules

## Phase 3 Additions: Synthesis

### README Additions

For TypeScript projects, the README should mention:

- TypeScript version requirement (especially for libraries)
- Whether the package ships types (`"types"` field) and whether they're handwritten or generated
- Module format (ESM-only, CJS, dual)
- Node.js version requirement

For libraries:

```markdown
## Requirements

- Node.js 20+
- TypeScript 5.5+ (for full type support)
```

### Module Format Statement (libraries)

````markdown
## Module Format

This package is published as ESM only. It does not provide a CommonJS build.
If you need to use it from CommonJS, use a dynamic import:

\```js
const { thing } = await import('the-package')
\```
````

Or:

```markdown
## Module Format

This package ships dual ESM and CJS builds via the `exports` field.
Both formats are first-class.
```

### tsconfig Documentation

In `DEVELOPMENT.md`, include a section explaining the project's tsconfig setup:

```markdown
## TypeScript Configuration

This project uses strict TypeScript with the following non-default options:

- `noUncheckedIndexedAccess: true` — array and object access returns `T | undefined`
- `exactOptionalPropertyTypes: true` — `?` and `| undefined` are not equivalent
- `verbatimModuleSyntax: true` — `import type` is required for type-only imports

The base config is in `tsconfig.json`. Additional configs:

- `tsconfig.build.json` — production build (excludes tests)
- `tsconfig.test.json` — test compilation
```

### Type Generation Steps

In `DEVELOPMENT.md`, document every code generation step:

```markdown
## Generated Code

The following code is generated and must be regenerated after changes:

| Source                 | Generator               | Output                            | When to Run          |
| ---------------------- | ----------------------- | --------------------------------- | -------------------- |
| `prisma/schema.prisma` | `pnpm db:generate`      | `node_modules/.prisma/client`     | After schema changes |
| `openapi.yaml`         | `pnpm openapi:generate` | `src/generated/api.ts`            | After spec changes   |
| `src/server/router.ts` | (none — inferred)       | (tRPC types inferred at use site) | Never                |
```

## Phase 4 Additions: Verification

### TypeScript-Specific Checks

```bash
# Verify the project type-checks
pnpm tsc --noEmit

# Verify the public API is what the docs say (library)
# This requires api-extractor or similar; manual check otherwise:
rg '^export ' src/index.ts

# Verify documented strictness flags match actual config
jq '.compilerOptions.strict, .compilerOptions.noUncheckedIndexedAccess' tsconfig.json

# Verify all path aliases mentioned in docs exist in tsconfig
jq '.compilerOptions.paths' tsconfig.json

# Verify the module format claim
jq '.type, .main, .module, .exports' package.json
```

### Cross-Reference: Generated vs. Hand-Written

If the project has generated code, verify the docs don't claim hand-written status for generated files (and vice versa).

## Common Footguns to Document

These belong in the "Hidden Coupling and Footguns" section:

- **`any` is contagious** — A single `any` in a hot path defeats type safety for everything downstream
- **Type assertions (`as`) are unsafe** — They bypass the type checker without runtime checks; flag heavy `as` usage
- **Index signatures vs. `Record`** — `Record<string, T>` returns `T`, not `T | undefined`, even when the key doesn't exist (without `noUncheckedIndexedAccess`)
- **`unknown` vs `any`** — `unknown` requires narrowing; `any` doesn't. Prefer `unknown` at boundaries.
- **`satisfies` vs annotation** — `const x: T = ...` widens; `const x = ... satisfies T` preserves the literal type
- **Module resolution mode** — `node` is legacy; `bundler` for bundled apps; `node16`/`nodenext` for Node ESM with explicit extensions
- **`.js` extensions in imports** — Required by `node16`/`nodenext`/`bundler` mode for relative imports, even though the source file is `.ts`
- **`import type` ordering** — `verbatimModuleSyntax: true` requires `import type` for type-only imports; runtime imports of types-only modules will break
- **Project reference build order** — `tsc -b` respects references; `tsc` does not. Editor errors that don't match CLI errors usually mean reference misconfiguration.
- **Path aliases at runtime** — TypeScript paths only inform the type checker. Runtime needs a bundler, `tsc-alias`, or `tsconfig-paths`.
- **Declaration file generation** — `declaration: true` is required for libraries; without it, consumers get `any` types
- **Const enums** — Don't work with `isolatedModules: true` (which most modern setups use)
- **Triple-slash directives** — Legacy; should be replaced with imports in modern code
- **`@types/*` version mismatches** — `@types/node` should match the runtime Node version; mismatches cause subtle API surface drift
- **Strict mode toggles in test files** — Some projects relax strictness for tests via a separate tsconfig; document this if so

## Useful Commands Quick Reference

```bash
# Type check without emitting
pnpm tsc --noEmit

# Type check in watch mode
pnpm tsc --noEmit --watch

# Build with project references
pnpm tsc -b

# Clean build artifacts
pnpm tsc -b --clean

# Generate declaration files only
pnpm tsc --emitDeclarationOnly

# Trace module resolution (debugging)
pnpm tsc --traceResolution > resolution.log

# List files included in compilation
pnpm tsc --listFiles

# Find what's importing a specific file
rg "from ['\"].*lib/foo['\"]" -t ts

# API surface inspection
pnpm dlx api-extractor run  # requires setup
pnpm dlx @microsoft/api-extractor run

# Type coverage
pnpm dlx type-coverage --strict
```

## Documentation Conventions to Honor

- **Types are documentation.** For libraries especially, exported type signatures should be self-explanatory. Hand-written API docs that duplicate type signatures are doomed to drift.
- **Generate API reference from types.** Use `typedoc`, `api-extractor`, or `tsdoc` for libraries rather than hand-writing API docs.
- **TSDoc comments matter.** `/** */` comments on exported items appear in editor tooltips for consumers. Document them like public API.
- **Strictness is part of the contract** for libraries. A library compiled with loose settings can break consumers using stricter settings.
- **Module format is part of the contract.** Switching from CJS to ESM (or vice versa) is a breaking change.
- **`@deprecated` JSDoc tag** is honored by editors and should be used instead of comments when removing API surface gradually.
