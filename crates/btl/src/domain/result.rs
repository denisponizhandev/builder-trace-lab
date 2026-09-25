use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct BundleProcessingResult {
    pub bundle_hash: String,
    pub target_block: i64,
    pub tx_count: i16,
    pub received_at: DateTime<Utc>,
    pub simulation_started_at: DateTime<Utc>,
    pub simulation_completed_at: DateTime<Utc>,
    pub queue_wait_us: i64,
    pub simulation_duration_us: i64
}