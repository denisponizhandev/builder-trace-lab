use crate::admission::bundle::BundleAdmission;

#[derive(Clone)]
pub struct AppState {
    pub admission: BundleAdmission
}

impl AppState {
    pub fn new(admission: BundleAdmission) -> Self {
        Self {
            admission
        }
    }
}