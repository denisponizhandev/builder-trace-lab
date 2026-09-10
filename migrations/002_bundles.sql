CREATE TABLE bundles (
    bundle_hash         TEXT PRIMARY KEY,
    target_block        BIGINT NOT NULL,
    replacement_uuid    UUID,
    tx_count            SMALLINT NOT NULL,
    status              TEXT NOT NULL,
    received_at         TIMESTAMP NOT NULL DEFAULT now(),
    included_block      BIGINT REFERENCES blocks(block_number),
    simulation_passed   BOOLEAN
);

CREATE INDEX bundles_target_block_idx ON bundles (target_block);
CREATE INDEX bundles_status_idx ON bundles (status);
CREATE INDEX bundles_received_at_idx ON bundles (received_at);