CREATE TABLE blocks (
    block_number        BIGINT PRIMARY KEY,
    block_hash          TEXT UNIQUE NOT NULL,
    block_timestamp     BIGINT NOT NULL,
    base_fee_per_gas    BIGINT,
    received_at         TIMESTAMP NOT NULL DEFAULT now()
);

CREATE INDEX blocks_block_timestamp_idx ON blocks (block_timestamp);
