# Builder Trace Lab

A Rust **builder-style pipeline** lab: ingest order flow, route it through queues and mock processing, persist results in PostgreSQL, and observe metrics. The goal is to **compare backpressure policies** (unbounded / wait / reject) with data, not to make a slow consumer faster.

## What’s implemented

- HTTP `eth_sendBundle` (mock private bundles)
- Pipeline: admission -> simulation worker (sleep) -> storage writer -> Postgres
- Admission policies: **unbounded** / **wait** / **reject** (`ADMISSION_POLICY`)
- Prometheus: `GET /metrics` (labels `source=bundle`, `mode` = admission policy)
- Docker: **Prometheus** scrape + **Grafana** dashboard `Bundle pipeline` (provisioned from `grafana/dashboards/`)
- Per-bundle rows in `bundles` (timings + hash); graceful shutdown across HTTP / simulation / writer



## What we measure today

**Counters:** received -> accepted or rejected -> simulation started -> processed or failed (storage errors).

**Gauges:** simulation queue depth & capacity (bounded modes), in-flight simulations, result queue depth. Unbounded mode does not expose simulation queue depth.

**Histograms:** queue wait (receive -> simulation start), mock simulation duration, end-to-end (receive -> successful INSERT), HTTP handler time, DB write time.

**Logs:** `bundle_hash` in structured logs - not in metric labels.

Switch policy by restarting with `ADMISSION_POLICY=unbounded|wait|reject`.

## Planned next

- Demo run under **vegeta** load + short write-up (article) from live metrics



## Stack

Rust (tokio, axum), PostgreSQL, Prometheus, Grafana (docker-compose); in-process Prometheus client in `btl`.

## Quick start

```bash
cp .env.example .env
# fill Postgres + btl env; HTTP_BIND port must match prometheus/prometheus.yml (default 8080)
docker compose up -d
./scripts/apply-migrations.sh
cargo run -p btl
```



### Observability


| Service    | URL (defaults)                                                                                           |
| ---------- | -------------------------------------------------------------------------------------------------------- |
| Prometheus | [http://127.0.0.1:9090](http://127.0.0.1:9090)                                                           |
| Grafana    | [http://127.0.0.1:3000](http://127.0.0.1:3000)                                                           |
| Dashboard  | [http://127.0.0.1:3000/d/btl-bundle/bundle-pipeline](http://127.0.0.1:3000/d/btl-bundle/bundle-pipeline) |


Grafana login: `GRAFANA_ADMIN_USER` / `GRAFANA_ADMIN_PASSWORD` from `.env` (defaults `admin` / `admin`).

Prometheus scrapes `host.docker.internal:8080/metrics` — run `btl` on the host, not inside compose.

1. **Targets:** Prometheus → Status → Targets → job `btl` should be **UP**.
2. **Datasource:** Grafana → Connections → Data sources → Prometheus → Save & test.
3. **Dashboard:** open **Bundle pipeline**, set variable **mode** to your `ADMISSION_POLICY`, refresh every 10s.

Full walkthrough (metrics → PromQL → panels):

Smoke check:

```bash
curl -s http://127.0.0.1:8080/metrics | head
curl -X POST http://127.0.0.1:8080/eth_sendBundle -H 'Content-Type: application/json' -d '{"jsonrpc":"2.0","method":"eth_sendBundle","params":[{"txs":["0x01"],"blockNumber":"0x1"}],"id":1}'
```

After a few requests, panel **Bundle funnel** should show non-zero rates.

### Load test (vegeta)

Install [vegeta](https://github.com/tsenart/vegeta) (`go install github.com/tsenart/vegeta@latest`). With `btl` running:

```bash
RATE=80 DURATION=60s MAX_WORKERS=50 ./scripts/vegeta-load.sh
```

Watch **Bundle pipeline** in Grafana (`mode` = your `ADMISSION_POLICY`). Tune `RATE` above mock service rate (`SIMULATION_DELAY_MS` sets ~1 worker throughput).

Environment variables: see `.env.example`.