use thiserror::Error;

use tokio::sync::mpsc;

use crate::domain::result::BundleProcessingResult;
use crate::metrics::bundle_metrics::BundleMetrics;

use crate::storage::bundles::BundleRepository;

#[derive(Debug, Error)] 
pub enum StorageWriterError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

pub struct StorageWriter {
    bundle_repo: BundleRepository,
    metrics: BundleMetrics
}

impl StorageWriter {
    pub fn new(bundle_repo: BundleRepository, metrics: BundleMetrics) -> Self {
        StorageWriter {
            bundle_repo,
            metrics
        }
    }

    pub async fn run_storage_writer(&self, mut rx: mpsc::Receiver<BundleProcessingResult>) -> Result<(), StorageWriterError> {
        
        while let Some(row) = rx.recv().await {
            match self.bundle_repo.insert_bundle_result(&row).await {
                Ok(()) => {
                    self.metrics.record_processed();
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