use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct BundleSimulationJob {
    pub bundle_hash: String,
    pub target_block: i64,
    pub tx_count: i16,
    pub received_at: DateTime<Utc>,
}
