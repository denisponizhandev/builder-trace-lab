#!/usr/bin/env bash
# Controlled HTTP load for eth_sendBundle (requires vegeta: https://github.com/tsenart/vegeta)
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

RATE="${RATE:-50}"
DURATION="${DURATION:-60s}"
MAX_WORKERS="${MAX_WORKERS:-50}"
BTL_URL="${BTL_URL:-http://127.0.0.1:8080/eth_sendBundle}"
# Unique JSON bodies round-robin during attack (one small file per variant)
TARGET_COUNT="${TARGET_COUNT:-500}"

if ! command -v vegeta >/dev/null 2>&1; then
  if command -v go >/dev/null 2>&1; then
    GOBIN="$(go env GOPATH 2>/dev/null)/bin"
    if [[ -x "${GOBIN}/vegeta" ]]; then
      export PATH="${GOBIN}:${PATH}"
    fi
  fi
fi
if ! command -v vegeta >/dev/null 2>&1; then
  echo "vegeta not found. Install: go install github.com/tsenart/vegeta@latest" >&2
  echo "Then add to PATH: export PATH=\"\$(go env GOPATH)/bin:\$PATH\"" >&2
  exit 1
fi

BODIES_DIR="$(mktemp -d)"
TARGETS="$(mktemp)"
RESULTS="$(mktemp)"
trap 'rm -rf "$BODIES_DIR" "$TARGETS" "$RESULTS"' EXIT

echo "Writing $TARGET_COUNT unique body files + vegeta targets (POST + @file)..."
for i in $(seq 1 "$TARGET_COUNT"); do
  body_file="${BODIES_DIR}/body_${i}.json"
  printf '{"jsonrpc":"2.0","method":"eth_sendBundle","params":[{"txs":["0x%016x"],"blockNumber":"0x123"}],"id":%d}' \
    "$i" "$i" >"$body_file"
  {
    printf 'POST %s\n' "$BTL_URL"
    printf 'Content-Type: application/json\n'
    printf '@%s\n\n' "$body_file"
  } >>"$TARGETS"
done

echo "vegeta attack -rate=$RATE -duration=$DURATION -max-workers=$MAX_WORKERS"
vegeta attack -rate="$RATE" -duration="$DURATION" -max-workers="$MAX_WORKERS" -targets="$TARGETS" \
  >"$RESULTS"
vegeta report <"$RESULTS"

echo ""
echo "Latency histogram (client):"
vegeta report -type='hist[0,10ms,25ms,50ms,100ms,250ms,500ms,1s,2s,5s]' <"$RESULTS"
