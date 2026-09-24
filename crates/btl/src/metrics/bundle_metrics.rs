use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct BundleMetrics {
    inner: Arc<BundlePipelineMetrics>
}

struct BundlePipelineMetrics {
    bundles_accepted: AtomicU64,
    bundles_rejected: AtomicU64,
    bundles_processed: AtomicU64,
    bundles_failed: AtomicU64
}

impl Default for BundlePipelineMetrics {
    fn default() -> Self {
        Self {
            bundles_accepted: AtomicU64::new(0),
            bundles_rejected: AtomicU64::new(0),
            bundles_processed: AtomicU64::new(0),
            bundles_failed: AtomicU64::new(0)
        }
    }
}

impl BundleMetrics {
    pub fn record_accepted(&self) {
        self.inner.bundles_accepted.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_rejected(&self) {
        self.inner.bundles_rejected.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_processed(&self) {
        self.inner.bundles_processed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_failed(&self) {
        self.inner.bundles_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot_accepted(&self) -> u64 {
        self.inner.bundles_accepted.load(Ordering::Relaxed)
    }

    pub fn snapshot_rejected(&self) -> u64 {
        self.inner.bundles_rejected.load(Ordering::Relaxed)
    }

    pub fn snapshot_processed(&self) -> u64 {
        self.inner.bundles_processed.load(Ordering::Relaxed)
    }

    pub fn snapshot_failed(&self) -> u64 {
        self.inner.bundles_failed.load(Ordering::Relaxed)
    }
}