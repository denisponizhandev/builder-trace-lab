# Builder Trace Lab

A Rust **builder-style pipeline** lab: ingest order flow, route it through queues and mock processing, persist results in PostgreSQL, and observe metrics. The goal is to **compare backpressure policies** (unbounded / wait / reject) with data, not to make a slow consumer faster.

## What’s implemented

- HTTP **`eth_sendBundle`** (mock private bundles)
- Pipeline: admission → simulation worker (sleep) → storage writer → Postgres
- Admission modes: **A** unbounded, **B** bounded + wait, **C** bounded + reject (`ADMISSION_POLICY`)
- Prometheus: **`GET /metrics`** (counters, histograms, queue depth)

Planned next: Grafana, synthetic load, filtered mempool, SQL analysis.

## Stack

Rust (tokio, axum), PostgreSQL (docker-compose), in-process Prometheus client.

## Quick start

```bash
cp .env.example .env
docker compose up -d   # Postgres
cargo run -p btl
```

Smoke check:

```bash
curl -s http://127.0.0.1:8080/metrics | head
curl -X POST http://127.0.0.1:8080/eth_sendBundle -H 'Content-Type: application/json' -d '...'
```

Environment variables: see `.env.example`.
