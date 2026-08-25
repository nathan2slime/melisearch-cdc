# Service

Rust Actix Web API for product CRUD, product search, dependency health checks, and OpenAPI docs.

## Responsibilities

| Area           | Details                                                                      |
| -------------- | ---------------------------------------------------------------------------- |
| HTTP API       | `/api/health` and `/api/products` routes                                     |
| Product writes | Creates, updates, deletes, and reads products in Postgres                    |
| Search         | Sends product queries to Meilisearch, then hydrates ranked IDs from Postgres |
| API docs       | Serves Swagger UI and OpenAPI JSON                                           |

## Architecture

The service uses Clean Architecture style boundaries.

| Layer         | Role                                                    |
| ------------- | ------------------------------------------------------- |
| `domain`      | Core product structures                                 |
| `application` | Product use cases, inputs, outputs, errors, and ports   |
| `infra`       | SeaORM Postgres repository and Meilisearch adapter      |
| `http`        | Actix handlers, request/response DTOs, and OpenAPI docs |

Product use-case files live in `src/application/products/`:

```txt
errors.rs     Product use-case errors
inputs.rs     Create/update/search inputs
outputs.rs    Search output and search-index output
ports.rs      ProductRepository and ProductSearchIndex ports
use_cases.rs  Product use cases
```

## Environment

| Variable                     | Default / Example                                          | Purpose                      |
| ---------------------------- | ---------------------------------------------------------- | ---------------------------- |
| `HOST`                       | `0.0.0.0`                                                  | API bind host                |
| `PORT`                       | `5400` locally, `8080` code fallback                       | API bind port                |
| `DATABASE_URL`               | `postgres://postgres:melisearch@localhost:5432/melisearch` | Postgres connection          |
| `MEILISEARCH_URL`            | `http://localhost:7700`                                    | Meilisearch base URL         |
| `MEILISEARCH_API_KEY`        | `342143821043`                                             | Optional Meilisearch API key |
| `MEILISEARCH_PRODUCTS_INDEX` | `products`                                                 | Product index name           |

For Docker Compose, create `.env.production` from the `Docker Compose` block in the root `.env.example`. It uses the same variable names with Docker-network hostnames, like `DATABASE_URL=postgres://postgres:melisearch@postgres:5432/melisearch` and `MEILISEARCH_URL=http://meilisearch:7700`.

## Local Run

Start the required infrastructure from the repository root, then run the service:

```sh
docker compose --env-file .env.production up -d postgres meilisearch
cargo run -p service
```

That Compose command also starts `migrations` before Meilisearch because the stack requires migrations to complete before every non-Postgres service.

Useful URLs:

| Resource     | URL                                           |
| ------------ | --------------------------------------------- |
| API base URL | `http://localhost:5400/api`                   |
| Swagger UI   | `http://localhost:5400/docs/`                 |
| OpenAPI JSON | `http://localhost:5400/api-docs/openapi.json` |

## API Reference

| Method   | Path                                          | Description                                    |
| -------- | --------------------------------------------- | ---------------------------------------------- |
| `GET`    | `/api/health`                                 | Service health report                          |
| `POST`   | `/api/products`                               | Create product                                 |
| `GET`    | `/api/products?page=1&per_page=20`            | Search products with Meilisearch default query |
| `GET`    | `/api/products?q=keyboard&page=1&per_page=20` | Search products by text                        |
| `GET`    | `/api/products/{id}`                          | Get product by ID                              |
| `PUT`    | `/api/products/{id}`                          | Update product by ID                           |
| `DELETE` | `/api/products/{id}`                          | Delete product by ID                           |

Search query params:

| Param      | Required | Default      | Rules                         |
| ---------- | -------- | ------------ | ----------------------------- |
| `q`        | No       | empty string | Trimmed before search         |
| `page`     | No       | `1`          | Must be `>= 1`                |
| `per_page` | No       | `20`         | Must be between `1` and `100` |

Create payload:

```json
{
  "name": "Keyboard",
  "description": "Mechanical",
  "price_cents": 12999,
  "stock": 7
}
```

Update payloads are partial. Omit `description` to keep it, send `null` to clear it, or send a string to replace it.

Validation rules:

| Field         | Rule                                  |
| ------------- | ------------------------------------- |
| `name`        | Must not be blank when provided       |
| `price_cents` | Must be zero or greater when provided |
| `stock`       | Must be zero or greater when provided |

## Verification

```sh
cargo fmt --package service -- --check
cargo check -p service
cargo test -p service
cargo clippy -p service --all-targets --all-features -- -D warnings
```
