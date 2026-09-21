use thiserror::Error;
use chrono::{DateTime, Utc};

use tokio::sync::mpsc;
use tokio::time::{sleep, Duration, Instant};
use sqlx::PgPool;

use crate::pipeline::message::{PipelineMessage, AcceptedBundle};

#[derive(Debug, Error)] 
pub enum ProcessorError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

struct Timings {
    simulation_started_at: DateTime<Utc>,
    simulation_completed_at: DateTime<Utc>,
    queue_wait_us: i64,
    simulation_duration_us: i64
}

async fn persist_inbound_bundle(
    pool: &PgPool, 
    bundle: &AcceptedBundle,
    timings: Option<Timings>
) -> Result<(), ProcessorError> {
    match timings {
        Some(t) => {    
            sqlx::query(
                r#"
                INSERT INTO bundles (bundle_hash, target_block, tx_count, received_at, simulation_started_at, simulation_completed_at, queue_wait_us, simulation_duration_us)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (bundle_hash) DO NOTHING;
                "#
            )
            .bind(bundle.bundle_hash())
            .bind(bundle.target_block())
            .bind(bundle.tx_count())
            .bind(bundle.received_at())
            .bind(t.simulation_started_at)
            .bind(t.simulation_completed_at)
            .bind(t.queue_wait_us)
            .bind(t.simulation_duration_us)
            .execute(pool)
            .await?;
        }
        None => {
            sqlx::query(
                r#"
                INSERT INTO bundles (bundle_hash, target_block, tx_count, received_at)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (bundle_hash) DO NOTHING;
                "#
            )
            .bind(bundle.bundle_hash())
            .bind(bundle.target_block())
            .bind(bundle.tx_count())
            .bind(bundle.received_at())
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}

async fn run_mock_step(work_ms: u64) -> i64 {
    let t0 = Instant::now();
    sleep(Duration::from_millis(work_ms)).await;
    t0.elapsed().as_micros() as i64
}

pub async fn run_processor(
    mut rx: mpsc::Receiver<PipelineMessage>, 
    pool: PgPool, 
    simulation_on: bool,
    simulation_delay_ms: u64
) -> Result<(), ProcessorError> {
    println!("processor started");

    while let Some(msg) = rx.recv().await {
        match msg {
            PipelineMessage::BundleAccepted(b) => {
                if simulation_on {
                    let simulation_started_at = Utc::now();
                    let queue_wait_us = (simulation_started_at - b.received_at()).num_microseconds().unwrap_or(0).max(0) as i64;
                    let simulation_duration_us = run_mock_step(simulation_delay_ms).await;
                    let simulation_completed_at = Utc::now();
                    
                    match persist_inbound_bundle(&pool, &b, Some(Timings {
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
                    match persist_inbound_bundle(&pool, &b, None).await {
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