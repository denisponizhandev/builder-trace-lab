CREATE TABLE pending_txs (
    job_id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tx_hash                     TEXT NOT NULL,
    from_address                TEXT NOT NULL,
    to_address                  TEXT,
    max_fee_per_gas             BIGINT NOT NULL,
    max_priority_fee_per_gas    BIGINT NOT NULL,
    received_at                 TIMESTAMPTZ NOT NULL DEFAULT now(),
    simulation_started_at       TIMESTAMPTZ,
    simulation_completed_at     TIMESTAMPTZ,
    stored_at                   TIMESTAMPTZ,
    queue_wait_us               BIGINT CHECK (queue_wait_us IS NULL OR queue_wait_us >= 0),
    simulation_duration_us      BIGINT CHECK (simulation_duration_us IS NULL OR simulation_duration_us >= 0),
    experiment_run_id           UUID REFERENCES experiment_runs (run_id)
);

CREATE UNIQUE INDEX pending_txs_run_tx_hash_uidx
    ON pending_txs (experiment_run_id, tx_hash)
    WHERE experiment_run_id IS NOT NULL;

CREATE UNIQUE INDEX pending_txs_live_tx_hash_uidx
    ON pending_txs (tx_hash)
    WHERE experiment_run_id IS NULL;

CREATE INDEX pending_txs_experiment_run_id_idx
    ON pending_txs (experiment_run_id)
    WHERE experiment_run_id IS NOT NULL;
