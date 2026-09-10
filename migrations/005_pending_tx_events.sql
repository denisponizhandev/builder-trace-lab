CREATE TABLE pending_tx_events (
    id              BIGSERIAL PRIMARY KEY,
    tx_hash         TEXT NOT NULL REFERENCES pending_txs(tx_hash),
    status          TEXT NOT NULL,
    occurred_at     TIMESTAMP NOT NULL,
    block_number    BIGINT
);

CREATE INDEX pending_tx_event_hash_occurred_at_idx ON pending_tx_events(tx_hash, occurred_at);