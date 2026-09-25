use axum::{Router, routing::{get, post}};

use super::handlers::{eth_send_bundle, prometheus_metrics};
use super::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/eth_sendBundle", post(eth_send_bundle))
        .route("/metrics", get(prometheus_metrics))
        .with_state(state)
}