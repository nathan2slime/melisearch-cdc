# Melisearch CDC

Rust and React monorepo that keeps a Meilisearch product index in sync with Postgres by using Change Data Capture (CDC).

The main data flow is:

```txt
Actix Web service
-> Postgres products table
-> Debezium Postgres connector
-> Kafka topic melisearch.public.products
-> Rust indexer
-> Meilisearch products index
```

## What This Project Does

- Exposes a product CRUD HTTP API with Actix Web.
- Stores products in Postgres through SeaORM.
- Uses SeaORM migrations to create the `products` table.
- Runs Debezium against Postgres logical replication to publish row changes to Kafka.
- Runs a Rust indexer that consumes Debezium product events from Kafka.
- Indexes only `id` and `name` in Meilisearch.
- Deletes Meilisearch documents when products are deleted in Postgres.
- Provides a small React app workspace, currently with an empty shell.

## Repository Layout

```txt
apps/
  service/      Actix Web API, product use cases, SeaORM repositories
  indexer/      Kafka consumer and Meilisearch writer for product CDC events
  migrations/   SeaORM migration CLI and product table migration
  web/          React app built with Rsbuild
infra/
  debezium/     Kafka Connect connector config for Postgres CDC
packages/       Shared frontend config packages
```

## Architecture

The Rust service and indexer are organized with Clean Architecture style boundaries:

- `domain`: core data structures.
- `application`: use cases and traits owned by the application layer.
- `infra`: adapters for external systems such as Postgres, Kafka, and Meilisearch.
- `http`: Actix Web handlers, API DTOs, and OpenAPI docs for the service.

The indexer application layer depends on two traits:

- `ProductEventSource`: reads product events and commits offsets.
- `ProductIndex`: writes product documents to a search index.

The infrastructure layer provides:

- `KafkaProductEventSource`, backed by `rdkafka`.
- `MeilisearchProductIndex`, backed by `reqwest`.

Kafka offsets are committed only after the event has been applied to Meilisearch.

## Requirements

- Rust stable with edition 2024 support.
- Node.js 22.
- pnpm 11.21.0.
- Docker and Docker Compose for Postgres, Kafka, Kafka UI, Meilisearch, and Debezium Connect.
- Native packages needed by `rdkafka` on Ubuntu-like systems:

```sh
sudo apt-get install -y libcurl4-openssl-dev libsasl2-dev
```

## Environment

`.env.example` contains local development values and Docker Compose values.

Important local variables:

```txt
HOST=0.0.0.0
PORT=5400
DATABASE_URL=postgres://postgres:melisearch@localhost:5432/melisearch
KAFKA_BOOTSTRAP_SERVERS=localhost:9092
KAFKA_PRODUCTS_TOPIC=melisearch.public.products
KAFKA_GROUP_ID=melisearch-indexer
MEILISEARCH_URL=http://localhost:7700
MEILISEARCH_API_KEY=342143821043
MEILISEARCH_PRODUCTS_INDEX=products
```

Important Docker Compose variables:

```txt
SERVICE_DATABASE_URL=postgres://postgres:melisearch@postgres:5432/melisearch
KAFKA_DOCKER_BOOTSTRAP_SERVERS=kafka:19092
MEILISEARCH_URL=http://meilisearch:7700
```

`.env` is intentionally ignored by Git.

## Install Dependencies

```sh
pnpm install
```

## Start Infrastructure

The compose file can start the shared infrastructure services:

```sh
docker compose up -d postgres kafka kafka-ui meilisearch connect
```

Useful local URLs:

- Kafka UI: `http://localhost:8282`
- Meilisearch: `http://localhost:7700`
- Debezium Connect: `http://localhost:8083`

Current Docker caveat: `docker-compose.yaml` references app Dockerfiles for `web` and `service`, but those Dockerfiles are not present yet. Start the app processes locally with Cargo and pnpm until those images are added.

## Run Migrations

With `DATABASE_URL` pointing to the local Postgres container:

```sh
cargo run -p migration -- up
```

The current migration creates:

```txt
products
  id integer primary key autoincrement
  name string not null
  description text nullable
  price_cents integer not null default 0
  stock integer not null default 0
```

## Register Debezium Connector

Postgres is configured in Docker Compose with logical replication enabled:

```txt
wal_level=logical
max_replication_slots=10
max_wal_senders=10
```

After Postgres, Kafka, and Debezium Connect are running, register the connector:

```sh
curl -i -X POST \
  -H "Content-Type: application/json" \
  --data @infra/debezium/postgres-conector.json \
  http://localhost:8083/connectors
```

The connector watches only `public.products` and publishes events under the `melisearch` topic prefix. For this table, the product topic is:

```txt
melisearch.public.products
```

## Run The API Service

```sh
cargo run -p service
```

Default local base URL:

```txt
http://localhost:5400/api
```

OpenAPI and Swagger UI:

```txt
http://localhost:5400/docs/
http://localhost:5400/api-docs/openapi.json
```

## API Endpoints

```txt
GET    /api/health
POST   /api/products
GET    /api/products?page=1&per_page=20
GET    /api/products?q=keyboard&page=1&per_page=20
GET    /api/products/{id}
PUT    /api/products/{id}
DELETE /api/products/{id}
```

Search responses are paginated:

```json
{
  "items": [
    {
      "id": 1,
      "name": "Keyboard",
      "description": "Mechanical",
      "price_cents": 12999,
      "stock": 7
    }
  ],
  "page": 1,
  "per_page": 20,
  "total_items": 1,
  "total_pages": 1
}
```

`total_items` is the total number of documents in the Meilisearch `products` index, ignoring the search query.

Create product payload:

```json
{
  "name": "Keyboard",
  "description": "Mechanical",
  "price_cents": 12999,
  "stock": 7
}
```

Update product payloads are partial. Only provided fields are changed:

```json
{
  "name": "Wireless Keyboard"
}
```

For `description`, omitted means keep the current value, `null` means clear the value, and a string means set a new value:

```json
{
  "description": null
}
```

Validation rules:

- `name` must not be blank when provided.
- `price_cents` must be zero or greater when provided.
- `stock` must be zero or greater when provided.

## Run The Indexer

```sh
cargo run -p indexer
```

The indexer consumes Debezium envelopes from `KAFKA_PRODUCTS_TOPIC`.

Supported Debezium operations:

- `c`, `r`, `u`: upsert a Meilisearch document from `payload.after`.
- `d`: delete a Meilisearch document using `payload.before.id`.
- tombstone or unknown events: ignore.

The Meilisearch document intentionally contains only:

```json
{
  "id": 1,
  "name": "Keyboard"
}
```

## Run The Web App

```sh
pnpm --filter web dev
```

The web app is currently an empty React shell.

## Verification

Run all workspace checks through pnpm and Turbo:

```sh
pnpm run check
pnpm run lint
pnpm run test
pnpm run build
```

Run Rust checks directly:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --workspace
```

Run service-only checks:

```sh
cargo fmt --package service
cargo check -p service
cargo test -p service
cargo clippy -p service --all-targets --all-features -- -D warnings
```

Run indexer-only checks:

```sh
cargo fmt --package indexer
cargo check -p indexer
cargo test -p indexer
cargo clippy -p indexer --all-targets --all-features -- -D warnings
```

## CI

GitHub Actions runs on pushes and pull requests targeting `master`.

The CI matrix runs:

- `pnpm run check`
- `pnpm run lint`
- `pnpm run test`
- `pnpm run build`

CI installs the native dependencies required by `rdkafka` before running workspace tasks.

## Current Gaps

- Dockerfiles for `apps/web` and `apps/service` are referenced by compose but not present yet.
- The indexer is not wired as a Docker Compose service yet.
- Debezium connector registration is manual.
- The web app is still an empty shell.
