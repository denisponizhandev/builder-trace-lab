use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct AcceptedBundle {
    bundle_hash: String,
    target_block: i64,
    tx_count: i16,
    received_at: DateTime<Utc>
}

impl AcceptedBundle {
    pub fn new(bundle_hash: String, target_block: i64, tx_count: i16, received_at: DateTime<Utc>) -> Self {
        Self {
            bundle_hash,
            target_block,
            tx_count,
            received_at
        }
    }

    pub fn bundle_hash(&self) -> &str {
        &self.bundle_hash
    }

    pub fn target_block(&self) -> i64 {
        self.target_block
    }

    pub fn tx_count(&self) -> i16 {
        self.tx_count
    }

    pub fn received_at(&self) -> DateTime<Utc> {
        self.received_at
    }
}

#[derive(Debug)]
pub enum PipelineMessage {
    BundleAccepted(AcceptedBundle),
    Shutdown
}
