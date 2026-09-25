use std::time::Instant;

use chrono::Utc;
use thiserror::Error;
use tokio::sync::mpsc;

use crate::domain::result::BundleProcessingResult;
use crate::metrics::app_metrics::AppMetrics;

use crate::storage::bundles::BundleRepository;

#[derive(Debug, Error)] 
pub enum StorageWriterError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

pub struct StorageWriter {
    bundle_repo: BundleRepository,
    metrics: AppMetrics
}

impl StorageWriter {
    pub fn new(bundle_repo: BundleRepository, metrics: AppMetrics) -> Self {
        StorageWriter {
            bundle_repo,
            metrics
        }
    }

    pub async fn run_storage_writer(&self, mut rx: mpsc::Receiver<BundleProcessingResult>) -> Result<(), StorageWriterError> {
        
        while let Some(row) = rx.recv().await {
            let db_start = Instant::now();
            match self.bundle_repo.insert_bundle_result(&row).await {
                Ok(()) => {
                    self.metrics.record_processed();
                    self.metrics
                        .observe_db_write_duration_seconds(db_start.elapsed().as_secs_f64());

                    let e2e_secs = (Utc::now() - row.received_at)
                        .num_microseconds()
                        .unwrap_or(0)
                        .max(0) as f64
                        / 1_000_000.0;
                        
                    self.metrics.observe_bundle_end_to_end_seconds(e2e_secs);
                }
                Err(e) => {
                    self.metrics.record_failed();
                    eprintln!("insert failed bundle_hash={}: {}", row.bundle_hash, e);
                }
            }
        }

        Ok(())
    }
}