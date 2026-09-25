use crate::admission::bundle::BundleAdmission;
use crate::metrics::app_metrics::AppMetrics;

#[derive(Clone)]
pub struct AppState {
    pub admission: BundleAdmission,
    pub metrics: AppMetrics,
}

impl AppState {
    pub fn new(admission: BundleAdmission, metrics: AppMetrics) -> Self {
        Self {
            admission,
            metrics,
        }
    }
}