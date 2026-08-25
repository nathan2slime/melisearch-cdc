# Apps

Runtime programs in this workspace live under `apps/`. Each app owns its own Dockerfile, package scripts, environment needs, and local verification commands.

| App          | Responsibility                                                                                    | README                             |
| ------------ | ------------------------------------------------------------------------------------------------- | ---------------------------------- |
| `service`    | Actix Web API, product use cases, Postgres repositories, Meilisearch search adapter, OpenAPI docs | [service](service/README.md)       |
| `migrations` | SeaORM migration CLI for database schema changes                                                  | [migrations](migrations/README.md) |
| `indexer`    | Kafka consumer that applies Debezium product events to Meilisearch                                | [indexer](indexer/README.md)       |
| `seed`       | Faker-based product seed binary                                                                   | [seed](seed/README.md)             |
| `web`        | React frontend built with Rsbuild                                                                 | [web](web/README.md)               |

## Docker Startup Order

`migrations` is a required one-shot program, not an optional application service. In Docker Compose it runs after `postgres` is healthy and every other container waits for it to finish successfully.

```txt
postgres healthy -> migrations up -> all other services
```

This protects services that touch the `products` table and keeps CDC infrastructure from starting before the source schema exists.
