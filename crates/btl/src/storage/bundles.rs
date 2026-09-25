use sqlx::PgPool;

use crate::domain::result::{BundleProcessingResult};

pub struct BundleRepository {
    pool: PgPool
}

impl BundleRepository {
    pub fn new(pool: PgPool) -> Self {
        BundleRepository {
            pool
        }
    }

    pub async fn insert_bundle_result(&self, r: &BundleProcessingResult) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO bundles (bundle_hash, target_block, tx_count, received_at, simulation_started_at, simulation_completed_at, queue_wait_us, simulation_duration_us)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (bundle_hash) DO NOTHING;
            "#
        )
        .bind(r.bundle_hash.clone())
        .bind(r.target_block)
        .bind(r.tx_count)
        .bind(r.received_at)
        .bind(r.simulation_started_at)
        .bind(r.simulation_completed_at)
        .bind(r.queue_wait_us)
        .bind(r.simulation_duration_us)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}