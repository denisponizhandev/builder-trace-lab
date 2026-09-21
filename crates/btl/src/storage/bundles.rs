use sqlx::PgPool;
use chrono::{DateTime, Utc};

use crate::pipeline::message::{AcceptedBundle};

pub struct Timings {
    pub simulation_started_at: DateTime<Utc>,
    pub simulation_completed_at: DateTime<Utc>,
    pub queue_wait_us: i64,
    pub simulation_duration_us: i64
}

pub struct BundleRepository {
    pool: PgPool
}

impl BundleRepository {
    pub fn new(pool: PgPool) -> Self {
        BundleRepository {
            pool
        }
    }

    pub async fn insert_accepted_bundle(&self, bundle: &AcceptedBundle, timings: Option<&Timings>) -> Result<(), sqlx::Error> {
        match timings {
            Some(t) => {    
                sqlx::query(
                    r#"
                    INSERT INTO bundles (bundle_hash, target_block, tx_count, received_at, simulation_started_at, simulation_completed_at, queue_wait_us, simulation_duration_us)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                    ON CONFLICT (bundle_hash) DO NOTHING;
                    "#
                )
                .bind(bundle.bundle_hash())
                .bind(bundle.target_block())
                .bind(bundle.tx_count())
                .bind(bundle.received_at())
                .bind(t.simulation_started_at)
                .bind(t.simulation_completed_at)
                .bind(t.queue_wait_us)
                .bind(t.simulation_duration_us)
                .execute(&self.pool)
                .await?;
            }
            None => {
                sqlx::query(
                    r#"
                    INSERT INTO bundles (bundle_hash, target_block, tx_count, received_at)
                    VALUES ($1, $2, $3, $4)
                    ON CONFLICT (bundle_hash) DO NOTHING;
                    "#
                )
                .bind(bundle.bundle_hash())
                .bind(bundle.target_block())
                .bind(bundle.tx_count())
                .bind(bundle.received_at())
                .execute(&self.pool)
                .await?;
            }
        }
    
        Ok(())
    }
}