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

| Variable                   | Default / Example     | Purpose                               |
| -------------------------- | --------------------- | ------------------------------------- |
| `REACT_APP_PUBLIC_API_URL` | `/api`                | Browser-visible API base URL          |
| `SERVICE_UPSTREAM`         | `http://service:5400` | Nginx API proxy target in Docker      |
| `PORT`                     | `5400`                | Used by the dev proxy target fallback |

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

The Dockerfile does not define application environment defaults. Docker Compose passes `REACT_APP_PUBLIC_API_URL` as a build argument from `.env.production`, and the nginx runtime reads `SERVICE_UPSTREAM` from the same file.

## Verification

```sh
pnpm --filter web check
pnpm --filter web lint
pnpm --filter web test
pnpm --filter web build
```
