# Melisearch CDC

Rust and React monorepo that keeps a Meilisearch `products` index synchronized with Postgres through Change Data Capture (CDC).

> The API writes product data to Postgres. Debezium publishes product row changes to Kafka. The Rust indexer consumes those events and updates Meilisearch. Search uses Meilisearch for ranking and Postgres for the full product response.

## Contents

- [Overview](#overview)
- [Data Flow](#data-flow)
- [Repository Layout](#repository-layout)
- [Architecture](#architecture)
- [Requirements](#requirements)
- [Environment](#environment)
- [Local Setup](#local-setup)
- [API Reference](#api-reference)
- [Indexer](#indexer)
- [Web App](#web-app)
- [Verification](#verification)
- [CI](#ci)

## Overview

| Area        | Stack                   | Responsibility                                               |
| ----------- | ----------------------- | ------------------------------------------------------------ |
| API service | Rust, Actix Web, SeaORM | Product CRUD, search endpoint, Postgres access, OpenAPI docs |
| Database    | Postgres                | Source of truth for product data                             |
| CDC         | Debezium, Kafka         | Publishes product table changes as events                    |
| Indexer     | Rust, rdkafka, reqwest  | Consumes product events and updates Meilisearch              |
| Search      | Meilisearch             | Product search ranking and product index document count      |
| Web         | React, Rsbuild          | Frontend workspace using `/api` as the service base URL      |

## Data Flow

```mermaid
flowchart LR
  client[Client / Web App]

  subgraph service[API Service]
    api[Actix Web<br/>/api]
    repo[Product Repository<br/>SeaORM]
    search[Product Search Adapter<br/>Meilisearch HTTP]
  end

  subgraph storage[Source Of Truth]
    pg[(Postgres<br/>products table)]
  end

  subgraph cdc[CDC Pipeline]
    debezium[Debezium Connector]
    kafka[(Kafka<br/>melisearch.public.products)]
    indexer[Rust Indexer]
  end

  subgraph index[Search Index]
    meili[(Meilisearch<br/>products index)]
  end

  client -->|CRUD requests| api
  client -->|Search requests| api
  api -->|Use cases| repo
  api -->|Search query + pagination| search
  repo -->|Read / write products| pg
  search -->|Ranked product IDs + index stats| meili
  api -->|Hydrate ranked IDs| repo

  pg -.->|Logical replication| debezium
  debezium -.->|Product events| kafka
  kafka -.->|Consume after commit| indexer
  indexer -.->|Upsert / delete documents| meili
```

## Repository Layout

```txt
apps/
  service/      Actix Web API, product use cases, SeaORM repositories, Meilisearch search adapter
  indexer/      Kafka consumer and Meilisearch writer for product CDC events
  migrations/   SeaORM migration CLI and product table migration
  web/          React app built with Rsbuild
infra/
  debezium/     Kafka Connect connector config for Postgres CDC
packages/       Shared frontend config packages
```

## Architecture

The Rust service and indexer use Clean Architecture style boundaries.

| Layer         | Role                                                    |
| ------------- | ------------------------------------------------------- |
| `domain`      | Core data structures                                    |
| `application` | Use cases, inputs, outputs, errors, and ports           |
| `infra`       | Adapters for Postgres, Kafka, and Meilisearch           |
| `http`        | Actix handlers, request/response DTOs, and OpenAPI docs |

### Service Product Modules

```txt
apps/service/src/application/products/
  errors.rs     Product use-case errors
  inputs.rs     Create/update/search inputs
  outputs.rs    Search output and search-index output
  ports.rs      ProductRepository and ProductSearchIndex ports
  use_cases.rs  Product use cases
```

### Indexer Ports

| Port                 | Implementation            | Purpose                                      |
| -------------------- | ------------------------- | -------------------------------------------- |
| `ProductEventSource` | `KafkaProductEventSource` | Reads product CDC events and commits offsets |
| `ProductIndex`       | `MeilisearchProductIndex` | Writes product documents to Meilisearch      |

Kafka offsets are committed only after the event has been applied to Meilisearch.

## Requirements

| Tool                    | Version / Notes                                          |
| ----------------------- | -------------------------------------------------------- |
| Rust                    | Stable toolchain with edition 2024 support               |
| Node.js                 | `22`                                                     |
| pnpm                    | `11.21.0`                                                |
| Docker + Docker Compose | Postgres, Kafka, Kafka UI, Meilisearch, Debezium Connect |
| Native packages         | Required by `rdkafka` on Ubuntu-like systems             |

```sh
sudo apt-get install -y libcurl4-openssl-dev libsasl2-dev
```

## Environment

`.env.example` contains local development values and Docker Compose values. `.env` is ignored by Git.

### Local Service Variables

| Variable                     | Example                                                    | Purpose                   |
| ---------------------------- | ---------------------------------------------------------- | ------------------------- |
| `HOST`                       | `0.0.0.0`                                                  | API bind host             |
| `PORT`                       | `5400`                                                     | API bind port             |
| `DATABASE_URL`               | `postgres://postgres:melisearch@localhost:5432/melisearch` | Local Postgres connection |
| `MEILISEARCH_URL`            | `http://localhost:7700`                                    | Local Meilisearch URL     |
| `MEILISEARCH_API_KEY`        | `342143821043`                                             | Meilisearch API key       |
| `MEILISEARCH_PRODUCTS_INDEX` | `products`                                                 | Product index name        |

### Indexer Variables

| Variable                  | Example                      | Purpose                                       |
| ------------------------- | ---------------------------- | --------------------------------------------- |
| `KAFKA_BOOTSTRAP_SERVERS` | `localhost:9092`             | Kafka bootstrap server for local indexer runs |
| `KAFKA_PRODUCTS_TOPIC`    | `melisearch.public.products` | Product CDC topic                             |
| `KAFKA_GROUP_ID`          | `melisearch-indexer`         | Indexer consumer group                        |

### Docker Compose Variables

| Variable                         | Example                                                   | Purpose                                          |
| -------------------------------- | --------------------------------------------------------- | ------------------------------------------------ |
| `SERVICE_DATABASE_URL`           | `postgres://postgres:melisearch@postgres:5432/melisearch` | Service-to-Postgres URL inside Docker network    |
| `MEILISEARCH_DOCKER_URL`         | `http://meilisearch:7700`                                 | Service-to-Meilisearch URL inside Docker network |
| `KAFKA_DOCKER_BOOTSTRAP_SERVERS` | `kafka:19092`                                             | Kafka URL inside Docker network                  |
| `REACT_APP_PUBLIC_API_URL`       | `/api`                                                    | Frontend API base URL                            |

## Local Setup

### 1. Install Dependencies

```sh
pnpm install
```

### 2. Start Infrastructure

```sh
docker compose up -d postgres kafka kafka-ui meilisearch connect
```

Useful local URLs:

| Service          | URL                     |
| ---------------- | ----------------------- |
| Kafka UI         | `http://localhost:8282` |
| Meilisearch      | `http://localhost:7700` |
| Debezium Connect | `http://localhost:8083` |

### 3. Run Migrations

```sh
cargo run -p migration -- up
```

The product table shape is:

| Column        | Type    | Notes                       |
| ------------- | ------- | --------------------------- |
| `id`          | integer | Primary key, auto-increment |
| `name`        | string  | Required                    |
| `description` | text    | Nullable                    |
| `price_cents` | integer | Required, defaults to `0`   |
| `stock`       | integer | Required, defaults to `0`   |

### 4. Register Debezium Connector

```sh
curl -i -X POST \
  -H "Content-Type: application/json" \
  --data @infra/debezium/postgres-conector.json \
  http://localhost:8083/connectors
```

Connector behavior:

| Source table      | Topic prefix | Product topic                |
| ----------------- | ------------ | ---------------------------- |
| `public.products` | `melisearch` | `melisearch.public.products` |

### 5. Run API Service

```sh
cargo run -p service
```

| Resource     | URL                                           |
| ------------ | --------------------------------------------- |
| API base URL | `http://localhost:5400/api`                   |
| Swagger UI   | `http://localhost:5400/docs/`                 |
| OpenAPI JSON | `http://localhost:5400/api-docs/openapi.json` |

### 6. Run Indexer

```sh
cargo run -p indexer
```

### 7. Run Web App

```sh
pnpm --filter web dev
```

The web app uses `/api` as its default API base URL. Rsbuild proxies `/api` requests to the local service during development.

## API Reference

### Endpoints

| Method   | Path                                          | Description                                    |
| -------- | --------------------------------------------- | ---------------------------------------------- |
| `GET`    | `/api/health`                                 | Service health report                          |
| `POST`   | `/api/products`                               | Create product                                 |
| `GET`    | `/api/products?page=1&per_page=20`            | Search products with Meilisearch default query |
| `GET`    | `/api/products?q=keyboard&page=1&per_page=20` | Search products by text                        |
| `GET`    | `/api/products/{id}`                          | Get product by ID                              |
| `PUT`    | `/api/products/{id}`                          | Update product by ID                           |
| `DELETE` | `/api/products/{id}`                          | Delete product by ID                           |

### Search Products

`GET /api/products`

| Query param | Required | Default      | Rules                         | Description                                                                |
| ----------- | -------- | ------------ | ----------------------------- | -------------------------------------------------------------------------- |
| `q`         | No       | empty string | Trimmed before search         | Text sent to Meilisearch. Empty query returns Meilisearch default results. |
| `page`      | No       | `1`          | Must be `>= 1`                | Search result page                                                         |
| `per_page`  | No       | `20`         | Must be between `1` and `100` | Results per page                                                           |

Search response:

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

Search metadata:

| Field         | Source                                 | Meaning                                                      |
| ------------- | -------------------------------------- | ------------------------------------------------------------ |
| `items`       | Meilisearch IDs hydrated from Postgres | Full product records in Meilisearch result order             |
| `page`        | Meilisearch search response            | Current search result page                                   |
| `per_page`    | Meilisearch search response            | Page size used by Meilisearch                                |
| `total_items` | Meilisearch `products` index stats     | Total number of documents in the product index, ignoring `q` |
| `total_pages` | Meilisearch search response            | Total pages for the current query and page size              |

### Create Product

`POST /api/products`

```json
{
  "name": "Keyboard",
  "description": "Mechanical",
  "price_cents": 12999,
  "stock": 7
}
```

### Update Product

`PUT /api/products/{id}`

Update payloads are partial. Only provided fields are changed.

```json
{
  "name": "Wireless Keyboard"
}
```

For `description`:

| Payload shape                         | Behavior                      |
| ------------------------------------- | ----------------------------- |
| omitted                               | Keeps the current description |
| `"description": null`                 | Clears the description        |
| `"description": "Wireless accessory"` | Sets a new description        |

```json
{
  "description": null
}
```

### Product Validation

| Field         | Rule                                  |
| ------------- | ------------------------------------- |
| `name`        | Must not be blank when provided       |
| `price_cents` | Must be zero or greater when provided |
| `stock`       | Must be zero or greater when provided |

## Indexer

The indexer consumes Debezium envelopes from `KAFKA_PRODUCTS_TOPIC`.

| Debezium operation  | Indexer action                                    |
| ------------------- | ------------------------------------------------- |
| `c`                 | Upsert product document from `payload.after`      |
| `r`                 | Upsert product document from `payload.after`      |
| `u`                 | Upsert product document from `payload.after`      |
| `d`                 | Delete product document using `payload.before.id` |
| Tombstone / unknown | Ignore                                            |

Meilisearch product documents contain only searchable/index identity data:

```json
{
  "id": 1,
  "name": "Keyboard"
}
```

The API returns full product data by using Meilisearch result IDs to fetch product rows from Postgres.

## Web App

| Command                   | Description                   |
| ------------------------- | ----------------------------- |
| `pnpm --filter web dev`   | Starts the Rsbuild dev server |
| `pnpm --filter web build` | Builds the React app          |
| `pnpm --filter web check` | Runs TypeScript checks        |

## Verification

### Workspace Checks

```sh
pnpm run check
pnpm run lint
pnpm run test
pnpm run build
```

### Rust Checks

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --workspace
```

### Service Checks

```sh
cargo fmt --package service
cargo check -p service
cargo test -p service
cargo clippy -p service --all-targets --all-features -- -D warnings
```

### Indexer Checks

```sh
cargo fmt --package indexer
cargo check -p indexer
cargo test -p indexer
cargo clippy -p indexer --all-targets --all-features -- -D warnings
```

## CI

GitHub Actions runs on pushes and pull requests targeting `master`.

| Check      | Command          |
| ---------- | ---------------- |
| Type/check | `pnpm run check` |
| Lint       | `pnpm run lint`  |
| Test       | `pnpm run test`  |
| Build      | `pnpm run build` |
