use std::error::Error;
use std::time::Duration;

use sqlx::PgPool;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

use crate::admission::bundle::{BundleAdmission, JobEgress, JobIngress};
use crate::config::{AdmissionPolicy, GlobalConfig};
use crate::domain::job::BundleSimulationJob;
use crate::domain::result::BundleProcessingResult;
use crate::http::router::build_router;
use crate::http::state::AppState;
use crate::metrics::app_metrics::AppMetrics;
use crate::pipeline::simulation::run_simulation_worker;
use crate::pipeline::writer::{StorageWriter, StorageWriterError};
use crate::storage::bundles::BundleRepository;

const SHUTDOWN_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);
const GAUGE_TICK_SECS: u64 = 1;

pub struct JobChannels {
    pub ingress: JobIngress,
    pub egress: JobEgress,
    pub job_tx_for_gauges: Option<mpsc::Sender<BundleSimulationJob>>,
}

pub fn build_job_channels(policy: AdmissionPolicy, channel_cap: usize) -> JobChannels {
    match policy {
        AdmissionPolicy::Unbounded => {
            let (tx, rx) = mpsc::unbounded_channel::<BundleSimulationJob>();
            JobChannels {
                ingress: JobIngress::Unbounded(tx),
                egress: JobEgress::Unbounded(rx),
                job_tx_for_gauges: None,
            }
        }
        AdmissionPolicy::WaitWhenFull => {
            let (tx, rx) = mpsc::channel::<BundleSimulationJob>(channel_cap);
            let gauge_tx = tx.clone();
            JobChannels {
                ingress: JobIngress::BoundedWait(tx),
                egress: JobEgress::BoundedWait(rx),
                job_tx_for_gauges: Some(gauge_tx),
            }
        }
        AdmissionPolicy::RejectWhenFull => {
            let (tx, rx) = mpsc::channel::<BundleSimulationJob>(channel_cap);
            let gauge_tx = tx.clone();
            JobChannels {
                ingress: JobIngress::BoundedReject(tx),
                egress: JobEgress::BoundedReject(rx),
                job_tx_for_gauges: Some(gauge_tx),
            }
        }
    }
}

pub fn build_metrics(policy: AdmissionPolicy, channel_cap: usize) -> AppMetrics {
    let metrics = AppMetrics::new(admission_policy_label(policy));
    if let Some(cap) = simulation_queue_capacity_gauge(policy, channel_cap) {
        metrics.set_simulation_queue_capacity(cap);
    }
    metrics
}

pub struct StageHandles {
    pub http: JoinHandle<Result<(), std::io::Error>>,
    pub gauge: JoinHandle<()>,
    pub simulation: JoinHandle<()>,
    pub writer: JoinHandle<Result<(), StorageWriterError>>,
    pub shutdown_tx: oneshot::Sender<()>,
}

pub async fn spawn_stages(
    config: &GlobalConfig,
    pool: &PgPool,
    channels: JobChannels,
    metrics: AppMetrics,
) -> Result<StageHandles, Box<dyn Error>> {
    let admission = BundleAdmission::new(channels.ingress, metrics.clone());
    let state = AppState::new(admission, metrics.clone());

    let router = build_router(state);
    let listener = tokio::net::TcpListener::bind(&config.http_bind).await?;

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let http = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await
    });

    let (result_tx, result_rx) =
        mpsc::channel::<BundleProcessingResult>(config.result_queue_capacity);
    let result_tx_for_gauges = result_tx.clone();

    let job_tx_for_gauges = channels.job_tx_for_gauges;
    let metrics_for_gauges = metrics.clone();
    let gauge = tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(GAUGE_TICK_SECS));
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
    let job_egress = channels.egress;
    let sleep_delay_ms = config.sleep_delay_ms;
    let simulation = tokio::spawn(async move {
        run_simulation_worker(job_egress, result_tx, sleep_delay_ms, metrics_for_sim).await
    });

    let bundle_repo = BundleRepository::new(pool.clone());
    let writer = StorageWriter::new(bundle_repo, metrics);
    let writer = tokio::spawn(async move { writer.run_storage_writer(result_rx).await });

    Ok(StageHandles {
        http,
        gauge,
        simulation,
        writer,
        shutdown_tx,
    })
}

pub async fn wait_shutdown(handles: StageHandles) {
    println!("shutdown requested");

    handles.gauge.abort();
    let _ = handles.gauge.await;

    if handles.shutdown_tx.send(()).is_err() {
        eprintln!("http shutdown signal: receiver already gone");
    }

    join_with_timeout(handles.http, "http", SHUTDOWN_DRAIN_TIMEOUT).await;
    join_with_timeout(handles.simulation, "simulation", SHUTDOWN_DRAIN_TIMEOUT).await;

    match join_with_timeout(handles.writer, "storage writer", SHUTDOWN_DRAIN_TIMEOUT).await {
        Some(Ok(())) => println!("storage writer drained"),
        Some(Err(e)) => eprintln!("storage writer error: {}", e),
        None => {}
    }

    println!("shutdown complete");
}

fn admission_policy_label(policy: AdmissionPolicy) -> &'static str {
    match policy {
        AdmissionPolicy::Unbounded => "unbounded",
        AdmissionPolicy::WaitWhenFull => "wait",
        AdmissionPolicy::RejectWhenFull => "reject",
    }
}

fn mpsc_sender_depth<T>(tx: &mpsc::Sender<T>) -> i64 {
    (tx.max_capacity() - tx.capacity()) as i64
}

fn simulation_queue_capacity_gauge(policy: AdmissionPolicy, channel_cap: usize) -> Option<i64> {
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
