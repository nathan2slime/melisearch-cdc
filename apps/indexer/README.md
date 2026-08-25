# Indexer

Rust worker that consumes Debezium product events from Kafka and updates the Meilisearch `products` index.

## Responsibilities

| Area           | Details                                                                         |
| -------------- | ------------------------------------------------------------------------------- |
| Event source   | Reads `KAFKA_PRODUCTS_TOPIC` with `rdkafka`                                     |
| Event handling | Parses Debezium envelopes for product inserts, updates, deletes, and snapshots  |
| Index writes   | Batch upserts and deletes product documents in Meilisearch                      |
| Offset safety  | Commits Kafka offsets only after Meilisearch accepts the batch                  |
| Retry          | Retries Kafka and Meilisearch operations instead of exiting on transient errors |

## Ports

| Port                 | Implementation            | Purpose                                      |
| -------------------- | ------------------------- | -------------------------------------------- |
| `ProductEventSource` | `KafkaProductEventSource` | Reads product CDC events and commits offsets |
| `ProductIndex`       | `MeilisearchProductIndex` | Writes product documents to Meilisearch      |

## Environment

| Variable                           | Default / Example            | Purpose                                   |
| ---------------------------------- | ---------------------------- | ----------------------------------------- |
| `KAFKA_BOOTSTRAP_SERVERS`          | `localhost:9092`             | Kafka bootstrap server                    |
| `KAFKA_PRODUCTS_TOPIC`             | `melisearch.public.products` | Product CDC topic                         |
| `KAFKA_GROUP_ID`                   | `melisearch-indexer`         | Consumer group                            |
| `KAFKA_PRODUCTS_BATCH_SIZE`        | `5000`                       | Maximum messages before each index write  |
| `KAFKA_PRODUCTS_BATCH_MAX_WAIT_MS` | `500`                        | Maximum wait after the first batch record |
| `MEILISEARCH_URL`                  | `http://localhost:7700`      | Meilisearch base URL                      |
| `MEILISEARCH_API_KEY`              | `342143821043`               | Optional Meilisearch API key              |
| `MEILISEARCH_PRODUCTS_INDEX`       | `products`                   | Product index name                        |

For Docker Compose, `.env.production` uses the same variable names with Docker-network values, like `KAFKA_BOOTSTRAP_SERVERS=kafka:19092` and `MEILISEARCH_URL=http://meilisearch:7700`.

## Local Run

Start Postgres, migrations, Kafka, Meilisearch, and Debezium Connect first. Register the Debezium connector before expecting product events.

```sh
cargo run -p indexer
```

The indexer retries failed Kafka and Meilisearch operations every second. This keeps the container alive while Kafka or Meilisearch is still becoming ready.

Register the connector from the repository root:

```sh
curl -i -X POST \
  -H "Content-Type: application/json" \
  --data @infra/debezium/postgres-conector.json \
  http://localhost:8083/connectors
```

## Event Behavior

For each batch, the indexer keeps only the latest event per product ID.

| Debezium operation  | Indexer action                                    |
| ------------------- | ------------------------------------------------- |
| `c`                 | Upsert product document from `payload.after`      |
| `r`                 | Upsert product document from `payload.after`      |
| `u`                 | Upsert product document from `payload.after`      |
| `d`                 | Delete product document using `payload.before.id` |
| Tombstone / unknown | Ignore                                            |

Meilisearch product documents intentionally contain only search/index identity data:

```json
{
  "id": 1,
  "name": "Keyboard"
}
```

The API fetches the full product rows from Postgres after Meilisearch returns ranked IDs.

## Verification

```sh
cargo fmt --package indexer -- --check
cargo check -p indexer
cargo test -p indexer
cargo clippy -p indexer --all-targets --all-features -- -D warnings
```
