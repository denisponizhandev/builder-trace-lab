use chrono::Utc;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration, Instant};

use crate::admission::bundle::JobEgress;
use crate::domain::result::BundleProcessingResult;
use crate::metrics::app_metrics::AppMetrics;

async fn run_mock_step(work_ms: u64) -> i64 {
    let t0 = Instant::now();
    sleep(Duration::from_millis(work_ms)).await;
    t0.elapsed().as_micros() as i64
}

pub async fn run_simulation_worker(
    mut job_rx: JobEgress,
    result_tx: mpsc::Sender<BundleProcessingResult>,
    delay_ms: u64,
    metrics: AppMetrics,
) {
    while let Some(job) = job_rx.recv().await {
        metrics.record_simulation_started();
        metrics.inc_simulation_in_flight();

        let simulation_started_at = Utc::now();
        let queue_wait_us = (simulation_started_at - job.received_at)
            .num_microseconds()
            .unwrap_or(0)
            .max(0) as i64;

        metrics.observe_queue_wait_seconds(queue_wait_us as f64 / 1_000_000.0);
        let simulation_duration_us = run_mock_step(delay_ms).await;
       
        metrics.observe_simulation_duration_seconds(simulation_duration_us as f64 / 1_000_000.0);
        metrics.dec_simulation_in_flight();

        let simulation_completed_at = Utc::now();

        let result = BundleProcessingResult {
            bundle_hash: job.bundle_hash,
            target_block: job.target_block,
            tx_count: job.tx_count,
            received_at: job.received_at,
            simulation_started_at,
            simulation_completed_at,
            queue_wait_us,
            simulation_duration_us,
        };

        if result_tx.send(result).await.is_err() {
            break;
        }
    }
}
