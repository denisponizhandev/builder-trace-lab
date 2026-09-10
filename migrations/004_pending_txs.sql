CREATE TABLE pending_txs (
    tx_hash                     TEXT PRIMARY KEY,
    from_address                TEXT NOT NULL,
    to_address                  TEXT,
    max_fee_per_gas             BIGINT NOT NULL,
    max_priority_fee_per_gas    BIGINT NOT NULL,
    first_seen_at               TIMESTAMP NOT NULL,
    status                      TEXT NOT NULL,
    included_at                 TIMESTAMP,
    included_block              BIGINT REFERENCES blocks(block_number),
    wait_seconds                INTEGER
);

CREATE INDEX pending_tx_first_seen_at_idx ON pending_txs(first_seen_at);
CREATE INDEX pending_tx_status_idx ON pending_txs(status);