# Seed

Rust binary that inserts fake products into Postgres for local development and search testing.

## Responsibility

The seed program creates `40,000` products in batches of `1,000`. It requires the `products` table, so migrations must run first.

## Environment

| Variable       | Example                                                    | Purpose             |
| -------------- | ---------------------------------------------------------- | ------------------- |
| `DATABASE_URL` | `postgres://postgres:melisearch@localhost:5432/melisearch` | Postgres connection |
| `RUST_LOG`     | `info`                                                     | Log filter          |

For Docker Compose, copy the root `.env.example` to `.env.production`. The template uses the Docker-network hostname: `DATABASE_URL=postgres://postgres:melisearch@postgres:5432/melisearch`.

## Commands

From the repository root:

```sh
cargo run -p seed
```

With Docker Compose:

```sh
docker compose --env-file .env.production --profile tools up seed
```

Seeded fields:

| Field         | Source                             |
| ------------- | ---------------------------------- |
| `name`        | Faker commerce product             |
| `description` | Faker commerce product description |
| `price_cents` | Faker commerce price range         |
| `stock`       | Random `0..500`                    |

## Verification

```sh
cargo fmt --package seed -- --check
cargo check -p seed
cargo test -p seed
cargo clippy -p seed --all-targets --all-features -- -D warnings
```
