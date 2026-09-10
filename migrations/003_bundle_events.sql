CREATE TABLE bundle_events (
    id              BIGSERIAL PRIMARY KEY,
    bundle_hash     TEXT NOT NULL REFERENCES bundles(bundle_hash),
    status          TEXT NOT NULL,
    occurred_at     TIMESTAMP NOT NULL,
    block_number    BIGINT,
    detail          JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX bundle_events_hash_occurred_at_idx ON bundle_events (bundle_hash, occurred_at);