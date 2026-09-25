# Builder Trace Lab

A Rust **builder-style pipeline** lab: ingest order flow, route it through queues and mock processing, persist results in PostgreSQL, and observe metrics. The goal is to **compare backpressure policies** (unbounded / wait / reject) with data, not to make a slow consumer faster.

## What’s implemented

- HTTP **`eth_sendBundle`** (mock private bundles)
- Pipeline: admission -> simulation worker (sleep) -> storage writer -> Postgres
- Admission modes: **A** unbounded, **B** bounded + wait, **C** bounded + reject (`ADMISSION_POLICY`)
- Prometheus: **`GET /metrics`** (labels `source=bundle`, `mode` = admission policy)
- Per-bundle rows in **`bundles`** (timings + hash); graceful shutdown across HTTP / simulation / writer

## What we measure today

**Counters:** received -> accepted or rejected -> simulation started -> processed or failed (storage errors).

**Gauges:** simulation queue depth & capacity (bounded modes), in-flight simulations, result queue depth. Unbounded mode does not expose simulation queue depth.

**Histograms:** queue wait (receive -> simulation start), mock simulation duration, end-to-end (receive -> successful INSERT), HTTP handler time, DB write time.

**Logs:** `bundle_hash` in structured logs - not in metric labels.

Switch policy by restarting with `ADMISSION_POLICY=unbounded|wait|reject`.

## Planned next

- Grafana + Prometheus scrape off `/metrics`
- Recorded experiment runs and time-series samples in Postgres
- Synthetic load generator and repeatable A / B / C runs under load
- SQL-backed analysis and written conclusions per admission mode
- Second ingress: filtered public mempool, then combined bundle + mempool

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
