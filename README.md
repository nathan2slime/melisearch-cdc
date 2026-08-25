# Meilisearch CDC

Rust and React monorepo that keeps a Meilisearch `products` index synchronized with Postgres through Change Data Capture (CDC).

The API writes product data to Postgres. Debezium publishes product row changes to Kafka. The Rust indexer consumes those events and updates Meilisearch. Search uses Meilisearch for ranking and Postgres for the full product response.

## Repository Map

| Path                                                       | Responsibility                                                        |
| ---------------------------------------------------------- | --------------------------------------------------------------------- |
| [`apps/`](apps/README.md)                                  | Runtime programs: API service, migrations, indexer, seed, and web app |
| [`packages/`](packages/README.md)                          | Shared frontend TypeScript and Rslint configuration packages          |
| [`infra/debezium/`](infra/debezium/postgres-conector.json) | Kafka Connect connector config for Postgres CDC                       |

## Data Flow

```txt
Client/Web -> API service -> Postgres
Client/Web -> API service -> Meilisearch ranked IDs -> Postgres hydrated products
Postgres logical replication -> Debezium Connect -> Kafka -> Indexer -> Meilisearch
```

## Requirements

| Tool                    | Version / Notes                                                                             |
| ----------------------- | ------------------------------------------------------------------------------------------- |
| Rust                    | Stable toolchain with edition 2024 support                                                  |
| Node.js                 | `22`                                                                                        |
| pnpm                    | `11.21.0`                                                                                   |
| Docker + Docker Compose | Postgres, Kafka, Kafka UI, Meilisearch, Debezium Connect, app containers                    |
| Native packages         | `libcurl4-openssl-dev` and `libsasl2-dev` for local `rdkafka` builds on Ubuntu-like systems |

## Quick Start

```sh
cp .env.example .env
cp .env.production.example .env.production
pnpm install
docker compose --env-file .env.production up --build
```

`.env` is for local app development. `.env.production` is for Docker Compose and uses the same app variable names with Docker-network values, like `DATABASE_URL=postgres://postgres:melisearch@postgres:5432/melisearch`.

Docker Compose runs `migrations` as a required one-shot program after `postgres` is healthy. Every other container waits until `migrations` exits successfully.

After Debezium Connect is available, register the connector:

```sh
curl -i -X POST \
  -H "Content-Type: application/json" \
  --data @infra/debezium/postgres-conector.json \
  http://localhost:8083/connectors
```

Optional seed data:

```sh
docker compose --env-file .env.production --profile tools up seed
```

Useful local URLs:

| Service          | URL                           |
| ---------------- | ----------------------------- |
| Web              | `http://localhost:3000`       |
| API              | `http://localhost:5400/api`   |
| Swagger UI       | `http://localhost:5400/docs/` |
| Kafka UI         | `http://localhost:8282`       |
| Meilisearch      | `http://localhost:7700`       |
| Debezium Connect | `http://localhost:8083`       |

## Local Development

Use Docker for infrastructure and run apps directly when you need faster feedback:

```sh
docker compose --env-file .env.production up -d postgres kafka kafka-ui meilisearch connect
cargo run -p service
cargo run -p indexer
pnpm --filter web dev
```

That Compose command also starts `migrations` because Kafka, Meilisearch, and Connect wait for it.

See app-specific READMEs for environment variables, commands, API details, and verification:

| App         | README                                                   |
| ----------- | -------------------------------------------------------- |
| API service | [`apps/service/README.md`](apps/service/README.md)       |
| Migrations  | [`apps/migrations/README.md`](apps/migrations/README.md) |
| Indexer     | [`apps/indexer/README.md`](apps/indexer/README.md)       |
| Seed        | [`apps/seed/README.md`](apps/seed/README.md)             |
| Web         | [`apps/web/README.md`](apps/web/README.md)               |

## Verification

```sh
pnpm run check
pnpm run lint
pnpm run test
pnpm run build
```

Rust workspace checks:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

GitHub Actions runs workspace checks on pushes and pull requests targeting `master`.
