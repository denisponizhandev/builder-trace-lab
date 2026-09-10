#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MIGRATIONS_DIR="${ROOT_DIR}/migrations"

cd "${ROOT_DIR}"

if [[ ! -d "${MIGRATIONS_DIR}" ]]; then
  echo "migrations directory not found: ${MIGRATIONS_DIR}" >&2
  exit 1
fi

# load .env for POSTGRES_USER / POSTGRES_DB
if [[ -f "${ROOT_DIR}/.env" ]]; then
  set -a
  # shellcheck disable=SC1091
  source "${ROOT_DIR}/.env"
  set +a
fi

: "${POSTGRES_USER:?POSTGRES_USER is not set}"
: "${POSTGRES_DB:?POSTGRES_DB is not set}"

echo "Waiting for postgres..."
until docker compose exec -T postgres pg_isready -U "${POSTGRES_USER}" -d "${POSTGRES_DB}" >/dev/null 2>&1; do
  sleep 1
done

shopt -s nullglob
files=("${MIGRATIONS_DIR}"/*.sql)
if (( ${#files[@]} == 0 )); then
  echo "No .sql files in ${MIGRATIONS_DIR}" >&2
  exit 1
fi

for f in "${files[@]}"; do
  echo "Applying $(basename "${f}")..."
  docker compose exec -T postgres \
    psql -v ON_ERROR_STOP=1 -U "${POSTGRES_USER}" -d "${POSTGRES_DB}" -f - < "${f}"
done

echo "All migrations applied."