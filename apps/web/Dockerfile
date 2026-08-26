# syntax=docker/dockerfile:1

FROM node:24-bookworm-slim AS builder

RUN corepack enable

WORKDIR /app

COPY package.json pnpm-lock.yaml pnpm-workspace.yaml turbo.json ./
COPY apps/web/package.json apps/web/package.json
COPY apps/service/package.json apps/service/package.json
COPY apps/indexer/package.json apps/indexer/package.json
COPY apps/migrations/package.json apps/migrations/package.json
COPY apps/seed/package.json apps/seed/package.json
COPY packages/typescript-config/package.json packages/typescript-config/package.json
COPY packages/rslint-config/package.json packages/rslint-config/package.json

RUN pnpm install --frozen-lockfile

COPY apps/web apps/web
COPY packages packages

ARG REACT_APP_PUBLIC_API_URL

RUN pnpm --filter web build

FROM nginx:1.29-alpine AS runtime

COPY apps/web/nginx.conf /etc/nginx/templates/default.conf.template
COPY --from=builder /app/apps/web/dist /usr/share/nginx/html

EXPOSE 80
