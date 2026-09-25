use std::error::Error;
use std::time::Duration;

use sqlx::postgres::{PgPoolOptions, PgPool};
use tokio::sync::{mpsc, oneshot};
use tokio::signal;
use tokio::task::JoinHandle;

use axum;

pub mod config;
pub mod pipeline;
pub mod http;
pub mod storage;
pub mod domain;
pub mod admission;
pub mod metrics;

use config::GlobalConfig;
use config::AdmissionPolicy;
use http::router::build_router;
use http::state::AppState;
use storage::bundles::BundleRepository;
use domain::job::BundleSimulationJob;
use domain::result::BundleProcessingResult;
use admission::bundle::{JobIngress, JobEgress, BundleAdmission};

use pipeline::simulation::run_simulation_worker;
use pipeline::writer::StorageWriter;
use metrics::app_metrics::AppMetrics;

/// Max time to wait for HTTP stop and for each pipeline stage to drain queued work.
const SHUTDOWN_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);

pub struct App {
    config: GlobalConfig,
    pool: PgPool
}

impl App {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let global_config = GlobalConfig::from_env()?;

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&global_config.db_url)
            .await?;

        Ok(Self {
            config: global_config,
            pool
        })
    }

    pub async fn start(&self) -> Result<(), Box<dyn Error>>{
        println!("pipeline started");
        
        let in_app_policy = self.config.admission_policy;
        let channel_cap = self.config.simulation_queue_capacity;
        let res_channel_cap = self.config.result_queue_capacity;
        let sleep_delay_ms = self.config.sleep_delay_ms;

        let (ingress, rx, job_tx_for_gauges) = match in_app_policy {
            AdmissionPolicy::Unbounded => {
                let (tx, rx) = mpsc::unbounded_channel::<BundleSimulationJob>();
                (JobIngress::Unbounded(tx), JobEgress::Unbounded(rx), None)
            }
            AdmissionPolicy::WaitWhenFull => {
                let (tx, rx) = mpsc::channel::<BundleSimulationJob>(channel_cap);
                let gauge_tx = tx.clone();
                (
                    JobIngress::BoundedWait(tx),
                    JobEgress::BoundedWait(rx),
                    Some(gauge_tx),
                )
            }
            AdmissionPolicy::RejectWhenFull => {
                let (tx, rx) = mpsc::channel::<BundleSimulationJob>(channel_cap);
                let gauge_tx = tx.clone();
                (
                    JobIngress::BoundedReject(tx),
                    JobEgress::BoundedReject(rx),
                    Some(gauge_tx),
                )
            }
        };

        let mode = admission_policy_label(in_app_policy);
        let metrics = AppMetrics::new(mode);
        if let Some(cap) = simulation_queue_capacity(in_app_policy, channel_cap) {
            metrics.set_simulation_queue_capacity(cap);
        }

        let admission = BundleAdmission::new(ingress, metrics.clone());
        let state = AppState::new(admission, metrics.clone());
        
        let router = build_router(state);
        let listener = tokio::net::TcpListener::bind(&self.config.http_bind).await?;

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        
        let http_handle = tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await
        });
       
        let (result_tx, result_rx) = mpsc::channel::<BundleProcessingResult>(res_channel_cap);
        let result_tx_for_gauges = result_tx.clone();

        let metrics_for_gauges = metrics.clone();
        let gauge_handle = tokio::spawn(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(1));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tick.tick().await;
                if let Some(job_tx) = &job_tx_for_gauges {
                    metrics_for_gauges.set_simulation_queue_depth(mpsc_sender_depth(job_tx));
                }
                metrics_for_gauges
                    .set_result_queue_depth(mpsc_sender_depth(&result_tx_for_gauges));
            }
        });

        let metrics_for_sim = metrics.clone();
        let simulation_handle = tokio::spawn(async move {
            run_simulation_worker(rx, result_tx, sleep_delay_ms, metrics_for_sim).await
        });

        let bundle_repo = BundleRepository::new(self.pool.clone());
        let writer = StorageWriter::new(bundle_repo, metrics.clone());

        let writer_handle = tokio::spawn(async move {
            writer.run_storage_writer(result_rx).await
        });

        signal::ctrl_c().await?;
        println!("shutdown requested");

        // Drop gauge sender clones so job/result channels can close after HTTP drains admission.
        gauge_handle.abort();
        let _ = gauge_handle.await;

        if shutdown_tx.send(()).is_err() {
            eprintln!("http shutdown signal: receiver already gone");
        }

        // HTTP graceful stop drops AppState → admission senders close → simulation queue drains.
        join_with_timeout(http_handle, "http", SHUTDOWN_DRAIN_TIMEOUT).await;
        join_with_timeout(simulation_handle, "simulation", SHUTDOWN_DRAIN_TIMEOUT).await;

        match join_with_timeout(writer_handle, "storage writer", SHUTDOWN_DRAIN_TIMEOUT).await {
            Some(Ok(())) => println!("storage writer drained"),
            Some(Err(e)) => eprintln!("storage writer error: {}", e),
            None => {}
        }

        println!("shutdown complete");

        Ok(())
    }
}

fn admission_policy_label(policy: AdmissionPolicy) -> &'static str {
    match policy {
        AdmissionPolicy::Unbounded => "unbounded",
        AdmissionPolicy::WaitWhenFull => "wait",
        AdmissionPolicy::RejectWhenFull => "reject",
    }
}

/// Messages waiting in a bounded tokio mpsc buffer (Sender has no `len()`).
fn mpsc_sender_depth<T>(tx: &mpsc::Sender<T>) -> i64 {
    (tx.max_capacity() - tx.capacity()) as i64
}

fn simulation_queue_capacity(policy: AdmissionPolicy, channel_cap: usize) -> Option<i64> {
    match policy {
        AdmissionPolicy::Unbounded => None,
        AdmissionPolicy::WaitWhenFull | AdmissionPolicy::RejectWhenFull => {
            Some(channel_cap as i64)
        }
    }
}

async fn join_with_timeout<T>(handle: JoinHandle<T>, stage: &str, timeout: Duration) -> Option<T> {
    tokio::pin!(handle);

    tokio::select! {
        res = &mut handle => {
            match res {
                Ok(value) => {
                    println!("{stage} stopped");
                    Some(value)
                }
                Err(join_err) => {
                    if join_err.is_cancelled() {
                        println!("{stage} cancelled");
                    } else if join_err.is_panic() {
                        eprintln!("{stage} panicked: {join_err:?}");
                    } else {
                        eprintln!("{stage} join error: {join_err}");
                    }
                    None
                }
            }
        }
        _ = tokio::time::sleep(timeout) => {
            handle.abort();
            eprintln!(
                "{stage} drain timed out after {}s",
                timeout.as_secs()
            );
            None
        }
    }
}