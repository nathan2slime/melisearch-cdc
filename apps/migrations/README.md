# Migrations

SeaORM migration CLI for the Postgres schema used by the product service, seed binary, Debezium connector, and indexer pipeline.

## Responsibility

`migrations` is the first program that must run after the database starts. It creates the source schema before any other container starts.

Docker Compose enforces this with:

```txt
postgres healthcheck -> migrations exits 0 -> every other service starts
```

If migrations fail, dependent services do not start.

## Commands

From the repository root:

```sh
cargo run -p migration -- up
cargo run -p migration -- down
cargo run -p migration -- status
```

The Docker image runs `migrations up` by default:

Copy the root `.env.example` to `.env.production` before running this command.

```sh
docker compose --env-file .env.production up migrations
```

## Schema

The current migration creates `public.products`.

| Column        | Type    | Notes                       |
| ------------- | ------- | --------------------------- |
| `id`          | integer | Primary key, auto-increment |
| `name`        | string  | Required                    |
| `description` | text    | Nullable                    |
| `price_cents` | integer | Required, defaults to `0`   |
| `stock`       | integer | Required, defaults to `0`   |

## Docker Compose Contract

Keep this service as a one-shot dependency for the rest of the stack. It should depend only on `postgres` being healthy; every other Compose service should depend on `migrations` completing successfully.

## Verification

```sh
cargo fmt --package migration -- --check
cargo check -p migration
cargo test -p migration
cargo clippy -p migration --all-targets --all-features -- -D warnings
```
