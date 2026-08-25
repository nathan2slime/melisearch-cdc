# syntax=docker/dockerfile:1

FROM rust:1.94-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates git pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY apps apps

RUN cargo build --release -p service

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

ENV HOST=0.0.0.0
ENV PORT=5400

COPY --from=builder /app/target/release/service /usr/local/bin/service

EXPOSE 5400

CMD ["service"]
