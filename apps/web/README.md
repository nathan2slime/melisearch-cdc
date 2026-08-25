# Web

React frontend built with Rsbuild. It talks to the API through `/api` by default.

## Responsibilities

| Area          | Details                                              |
| ------------- | ---------------------------------------------------- |
| UI            | Product search and product interactions              |
| Routing       | TanStack Router                                      |
| Data fetching | Axios and TanStack Query                             |
| Build         | Rsbuild, React, TypeScript, Tailwind CSS, Ant Design |

## Environment

| Variable                   | Default | Purpose                               |
| -------------------------- | ------- | ------------------------------------- |
| `REACT_APP_PUBLIC_API_URL` | `/api`  | Browser-visible API base URL          |
| `PORT`                     | `5400`  | Used by the dev proxy target fallback |

`rsbuild.config.ts` loads environment variables from the repository root and proxies `/api` to `http://localhost:${PORT}` during development.

## Commands

From the repository root:

```sh
pnpm --filter web dev
pnpm --filter web build
pnpm --filter web check
pnpm --filter web lint
```

## Docker

The Docker image builds the static app with `REACT_APP_PUBLIC_API_URL` from `.env.production`, which should be created from the `Docker Compose` block in the root `.env.example`. It defaults to `/api`. The nginx runtime proxies API traffic to `SERVICE_UPSTREAM`, which defaults to `http://service:5400`.

## Verification

```sh
pnpm --filter web check
pnpm --filter web lint
pnpm --filter web test
pnpm --filter web build
```
