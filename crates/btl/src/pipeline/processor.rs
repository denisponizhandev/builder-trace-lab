use thiserror::Error;
use chrono::{DateTime, Utc};

use tokio::sync::mpsc;
use tokio::time::{sleep, Duration, Instant};

use crate::pipeline::message::{PipelineMessage};
use crate::storage::bundles::{BundleRepository, Timings};

#[derive(Debug, Error)] 
pub enum ProcessorError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

pub struct Processor {
    bundle_repo: BundleRepository
}

impl Processor {
    pub fn new(bundle_repo: BundleRepository) -> Self {
        Processor {
            bundle_repo
        }
    }

    // TODO: create simulation module
    async fn run_mock_step(work_ms: u64) -> i64 {
        let t0 = Instant::now();
        sleep(Duration::from_millis(work_ms)).await;
        t0.elapsed().as_micros() as i64
    }

    fn queue_wait_us(received_at: DateTime<Utc>, started_at: DateTime<Utc>) -> i64 {
        (started_at - received_at).num_microseconds().unwrap_or(0).max(0) as i64
    }

    pub async fn run_processor(
        &self,
        mut rx: mpsc::Receiver<PipelineMessage>, 
        simulation_on: bool,
        simulation_delay_ms: u64
    ) -> Result<(), ProcessorError> {
        println!("processor started");

        while let Some(msg) = rx.recv().await {
            match msg {
                PipelineMessage::BundleAccepted(b) => {
                    if simulation_on {
                        let simulation_started_at = Utc::now();
                        let queue_wait_us = Self::queue_wait_us(b.received_at(), simulation_started_at);
                        let simulation_duration_us = Self::run_mock_step(simulation_delay_ms).await;
                        let simulation_completed_at = Utc::now();

                        match self.bundle_repo.insert_accepted_bundle(&b, Some(&Timings {
                            simulation_started_at,
                            simulation_completed_at,
                            queue_wait_us,
                            simulation_duration_us
                        })).await {
                            Ok(()) => {},
                            Err(e) => {
                                eprintln!("persist failed hash={}: {}", b.bundle_hash(), e);
                                continue; 
                            }
                        };

                    } else {
                        match self.bundle_repo.insert_accepted_bundle(&b, None).await {
                            Ok(()) => {},
                            Err(e) => {
                                eprintln!("persist failed hash={}: {}", b.bundle_hash(), e);
                                continue; 
                            }
                        };
                    }
                },
                PipelineMessage::Shutdown => {
                    println!("received {:?} from channel", msg);
                    break;
                },
            }
        }

        println!("processor exited");

        Ok(())
    }
}