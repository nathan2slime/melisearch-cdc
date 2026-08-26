# Meilisearch CDC

A Rust + React monorepo demonstrating how to keep a **Meilisearch** index synchronized with **PostgreSQL** using **Change Data Capture (CDC)**.

The API stores product data in PostgreSQL, **Debezium** captures database changes through logical replication and publishes them to **Kafka**, and a Rust indexer consumes those events to keep the Meilisearch `products` index up to date.

Search queries are ranked by Meilisearch and hydrated from PostgreSQL before being returned to the client.

## Architecture

```mermaid
flowchart LR
    Client[Client / React Web App]
    API[Rust API]
    Postgres[(PostgreSQL)]
    Debezium[Debezium Connect]
    Kafka[(Kafka)]
    Indexer[Rust Indexer]
    Meili[(Meilisearch)]

    Client -->|CRUD| API
    Client -->|Search| API

    API -->|Write / Read| Postgres
    API -->|Search query| Meili
    Meili -->|Ranked product IDs| API
    API -->|Hydrate products| Postgres

    Postgres -->|Logical replication| Debezium
    Debezium -->|CDC events| Kafka
    Kafka -->|Consume events| Indexer
    Indexer -->|Upsert / Delete| Meili
```

## Tech Stack

| Technology     | Purpose                                          |
| -------------- | ------------------------------------------------ |
| Rust           | API, migrations, seed and CDC indexer            |
| React          | Web application                                  |
| PostgreSQL     | Primary database and source of truth             |
| Meilisearch    | Full-text search and ranking                     |
| Apache Kafka   | CDC event transport                              |
| Debezium       | PostgreSQL Change Data Capture                   |
| Docker Compose | Local infrastructure and container orchestration |
| pnpm           | JavaScript workspace and package management      |

## Repository Structure

```text
.
├── apps/
│   ├── service/       # Rust HTTP API
│   ├── migrations/    # Database migrations
│   ├── indexer/       # Kafka consumer -> Meilisearch
│   ├── seed/          # Development seed data
│   └── web/           # React frontend
│
├── packages/          # Shared frontend packages and configuration
│
└── infra/
    └── debezium/      # Debezium / Kafka Connect configuration
```

| Path                                                       | Responsibility                                               |
| ---------------------------------------------------------- | ------------------------------------------------------------ |
| [`apps/`](apps/README.md)                                  | Runtime applications: API, migrations, indexer, seed and web |
| [`packages/`](packages/README.md)                          | Shared TypeScript packages and Rslint configuration          |
| [`infra/debezium/`](infra/debezium/postgres-conector.json) | PostgreSQL Debezium connector configuration                  |

## Requirements

Make sure the following tools are installed before running the project:

| Tool            | Version / Notes                                                                                   |
| --------------- | ------------------------------------------------------------------------------------------------- |
| Rust            | Stable toolchain with Rust 2024 edition support                                                   |
| Node.js         | `22`                                                                                              |
| pnpm            | `11.21.0`                                                                                         |
| Docker          | Docker Engine or Docker Desktop                                                                   |
| Docker Compose  | Compose v2                                                                                        |
| Native packages | `libcurl4-openssl-dev` and `libsasl2-dev` when building `rdkafka` locally on Ubuntu-based systems |

On Ubuntu/Debian:

```sh
sudo apt install libcurl4-openssl-dev libsasl2-dev
```

## Getting Started

### 1. Install dependencies

```sh
pnpm install
```

### 2. Configure the environment

The repository contains two environment configurations:

* `.env` — local application development
* `.env.production` — Docker Compose

Copy the corresponding sections from `.env.example`.

```sh
cp .env.example .env
```

Create `.env.production` using the **Docker Compose** configuration from `.env.example`.

Docker containers use Docker network hostnames instead of `localhost`.

Rust application logs use `RUST_LOG`, which defaults to `info` in `.env.example`.

For example:

```env
DATABASE_URL=postgres://postgres:melisearch@postgres:5432/melisearch
```

The Compose environment also exposes configurable ports such as:

```text
WEB_PORT
SERVICE_PORT
POSTGRES_PORT
KAFKA_EXTERNAL_PORT
KAFKA_UI_PORT
MEILISEARCH_PORT
DEBEZIUM_CONNECT_PORT
```

### 3. Start the stack

```sh
docker compose --env-file .env.production up --build
```

Docker Compose starts the infrastructure and application containers.

The `migrations` service runs as a required one-shot container after PostgreSQL becomes healthy.

Services that depend on the database wait until migrations complete successfully before starting.

### 4. Register the Debezium connector

Once Debezium Connect is available:

```sh
curl -i -X POST \
  -H "Content-Type: application/json" \
  --data @infra/debezium/postgres-conector.json \
  http://localhost:8083/connectors
```

Verify registered connectors:

```sh
curl http://localhost:8083/connectors
```

### 5. Seed development data

Seed data is optional and available through the `tools` Compose profile:

```sh
docker compose \
  --env-file .env.production \
  --profile tools \
  up seed
```

## Local Development

For faster development cycles, run only the infrastructure with Docker and start the applications directly on the host.

### Start infrastructure

```sh
docker compose \
  --env-file .env.production \
  up -d postgres kafka kafka-ui meilisearch connect
```

The required `migrations` service is also executed automatically.

### Start the API

```sh
cargo run -p service
```

### Start the CDC indexer

```sh
cargo run -p indexer
```

### Start the web application

```sh
pnpm --filter web dev
```

## Services

| Service          | URL                         |
| ---------------- | --------------------------- |
| Web              | http://localhost:3000       |
| API              | http://localhost:5400/api   |
| Swagger UI       | http://localhost:5400/docs/ |
| Kafka UI         | http://localhost:8282       |
| Meilisearch      | http://localhost:7700       |
| Debezium Connect | http://localhost:8083       |

## How CDC Synchronization Works

A product mutation starts in PostgreSQL.

For example:

```text
POST /products
      |
      v
 PostgreSQL
      |
      | WAL / logical replication
      v
  Debezium
      |
      v
    Kafka
      |
      v
 Rust Indexer
      |
      v
 Meilisearch
```

Debezium reads PostgreSQL's logical replication stream and converts row-level changes into Kafka events.

The Rust indexer consumes those events and translates them into Meilisearch operations:

```text
INSERT  -> add/update document
UPDATE  -> add/update document
DELETE  -> delete document
```

The API therefore does **not** need to synchronously update Meilisearch whenever a product changes.

This keeps the write path decoupled from the search infrastructure.

## Search Strategy

Meilisearch is responsible for:

* full-text search
* relevance ranking
* filtering
* returning ordered product IDs

PostgreSQL remains responsible for the canonical product data.

Conceptually:

```text
GET /search?q=keyboard

        |
        v

   Meilisearch
        |
        | ranked IDs
        v

[42, 18, 91, 7]

        |
        v

   PostgreSQL
        |
        | hydrated products
        v

   API Response
```

This prevents the search index from becoming the system's source of truth.

## Application Documentation

Each application contains its own documentation with commands, configuration and implementation details.

| Application | Documentation                                            |
| ----------- | -------------------------------------------------------- |
| API Service | [`apps/service/README.md`](apps/service/README.md)       |
| Migrations  | [`apps/migrations/README.md`](apps/migrations/README.md) |
| Indexer     | [`apps/indexer/README.md`](apps/indexer/README.md)       |
| Seed        | [`apps/seed/README.md`](apps/seed/README.md)             |
| Web         | [`apps/web/README.md`](apps/web/README.md)               |

## Useful Commands

Start everything:

```sh
docker compose --env-file .env.production up --build
```

Start in the background:

```sh
docker compose --env-file .env.production up -d
```

Stop the stack:

```sh
docker compose --env-file .env.production down
```

Rebuild containers:

```sh
docker compose --env-file .env.production build
```

View logs:

```sh
docker compose --env-file .env.production logs -f
```

Increase Rust app log verbosity by setting `RUST_LOG`, for example `RUST_LOG=debug`.

View indexer logs:

```sh
docker compose --env-file .env.production logs -f indexer
```

Check Debezium connectors:

```sh
curl http://localhost:8083/connectors
```

Check connector status:

```sh
curl http://localhost:8083/connectors/postgres-products/status
```

## Project Goals

This repository is primarily a study and reference project for experimenting with:

* Change Data Capture
* PostgreSQL logical replication
* event-driven architectures
* Kafka consumers
* eventual consistency
* search indexing
* Rust backend development
* React applications
* Docker-based local environments
* synchronization between transactional databases and search engines

The main architectural principle is simple:

> PostgreSQL owns the data. Meilisearch indexes it. CDC keeps both synchronized.
