use tokio::time::Instant;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;

use crate::domain::job::BundleSimulationJob;
use crate::metrics::bundle_metrics::BundleMetrics;

#[derive(Clone)]
pub enum JobIngress {
    Unbounded(mpsc::UnboundedSender<BundleSimulationJob>),
    BoundedWait(mpsc::Sender<BundleSimulationJob>),
    BoundedReject(mpsc::Sender<BundleSimulationJob>)
}

pub enum JobEgress {
    Unbounded(mpsc::UnboundedReceiver<BundleSimulationJob>),
    BoundedWait(mpsc::Receiver<BundleSimulationJob>),
    BoundedReject(mpsc::Receiver<BundleSimulationJob>)
}

impl JobEgress {
    pub async fn recv(&mut self) -> Option<BundleSimulationJob> {
        match self {
            JobEgress::Unbounded(rx) => rx.recv().await,
            JobEgress::BoundedWait(rx) => rx.recv().await,
            JobEgress::BoundedReject(rx) => rx.recv().await
        }
    }
}

pub enum AdmitResult {
    Accepted,
    RejectedQueueFull,
    RejectedSubsystemDown
}

#[derive(Clone)]
pub struct BundleAdmission {
    ingress: JobIngress,
    metrics: BundleMetrics
}

impl BundleAdmission {
    pub fn new(ingress: JobIngress, metrics: BundleMetrics) -> Self {
        BundleAdmission {
            ingress,
            metrics
        }
    }

    pub async fn admit(&self, mut job: BundleSimulationJob) -> AdmitResult {
        match &self.ingress {
            JobIngress::Unbounded(tx) => {
                job.enqueued_at = Some(Instant::now());

                if tx.send(job).is_err() {
                    return AdmitResult::RejectedSubsystemDown;
                }
                self.metrics.record_accepted();

                AdmitResult::Accepted
            }
            JobIngress::BoundedWait(tx) => {
                job.enqueued_at = Some(Instant::now());

                if tx.send(job).await.is_err() {
                    return AdmitResult::RejectedSubsystemDown;
                }
                self.metrics.record_accepted();

                AdmitResult::Accepted
            }
            JobIngress::BoundedReject(tx) => {
                job.enqueued_at = Some(Instant::now());

                match tx.try_send(job) {
                    Ok(()) => {
                        self.metrics.record_accepted();

                        AdmitResult::Accepted
                    }
                    Err(TrySendError::Full(_)) => {
                        self.metrics.record_rejected();
                        
                        AdmitResult::RejectedQueueFull
                    }
                    Err(TrySendError::Closed(_)) => {
                        AdmitResult::RejectedSubsystemDown
                    }
                }
            }
        }
    }
}