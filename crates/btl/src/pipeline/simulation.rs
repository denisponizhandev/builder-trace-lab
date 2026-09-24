use tokio::sync::mpsc;
use chrono::Utc;

use tokio::time::{sleep, Duration, Instant};

use crate::admission::bundle::JobEgress;
use crate::domain::result::BundleProcessingResult;

async fn run_mock_step(work_ms: u64) -> i64 {
    let t0 = Instant::now();
    sleep(Duration::from_millis(work_ms)).await;
    t0.elapsed().as_micros() as i64
}

pub async fn run_simulation_worker(
    mut job_rx: JobEgress,
    result_tx: mpsc::Sender<BundleProcessingResult>,
    delay_ms: u64
) {
    while let Some(job) = job_rx.recv().await {
        let simulation_started_at = Utc::now();
        let queue_wait_us = job
            .enqueued_at
            .expect("accepted job must have enqueued_at")
            .elapsed()
            .as_micros() as i64;
            
        let simulation_duration_us = run_mock_step(delay_ms).await;
        let simulation_completed_at = Utc::now();

        let result = BundleProcessingResult {
            bundle_hash: job.bundle_hash,
            target_block: job.target_block,
            tx_count: job.tx_count,
            received_at: job.received_at,
            simulation_started_at,
            simulation_completed_at,
            queue_wait_us,
            simulation_duration_us
        };

        if result_tx.send(result).await.is_err() {
            break;
        }
    }
}