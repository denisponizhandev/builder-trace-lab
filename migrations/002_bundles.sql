CREATE TABLE bundles (
    bundle_hash                 TEXT PRIMARY KEY,
    target_block                BIGINT NOT NULL,
    tx_count                    INTEGER NOT NULL CHECK (tx_count >= 1),
    received_at                 TIMESTAMPTZ NOT NULL,
    simulation_started_at       TIMESTAMPTZ,
    simulation_completed_at     TIMESTAMPTZ,
    stored_at                   TIMESTAMPTZ NOT NULL DEFAULT now(),
    queue_wait_us               BIGINT CHECK (queue_wait_us IS NULL OR queue_wait_us >= 0),
    simulation_duration_us      BIGINT CHECK (simulation_duration_us IS NULL OR simulation_duration_us >= 0),
    experiment_run_id           UUID REFERENCES experiment_runs (run_id)
);

CREATE INDEX bundles_experiment_run_id_idx
    ON bundles (experiment_run_id)
    WHERE experiment_run_id IS NOT NULL;

CREATE INDEX bundles_received_at_idx
    ON bundles (received_at);
