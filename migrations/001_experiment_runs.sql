CREATE TABLE experiment_runs (
    run_id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scenario                        TEXT NOT NULL CHECK (scenario IN('A', 'B', 'C', 'D')),
    source                          TEXT NOT NULL CHECK (source IN('bundle', 'mempool', 'combined')),
    admission_policy                TEXT NOT NULL CHECK (admission_policy IN('unbounded', 'wait', 'reject', 'drop')),
    simulation_delay_ms             INTEGER NOT NULL CHECK (simulation_delay_ms >= 0),
    queue_capacity                  INTEGER CHECK (queue_capacity IS NULL OR queue_capacity >= 1),
    worker_count                    INTEGER NOT NULL CHECK (worker_count >= 1),
    rate_limit_per_sec              INTEGER,
    target_producer_rate_per_sec    INTEGER,
    target_duration_sec             INTEGER NOT NULL CHECK (target_duration_sec >= 1),
    started_at                      TIMESTAMPTZ NOT NULL DEFAULT now(),
    finished_at                     TIMESTAMPTZ,
    full_config                     JSONB NOT NULL DEFAULT '{}'::jsonb,
    notes                           TEXT
);