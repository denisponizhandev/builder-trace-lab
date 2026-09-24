use axum::{Router, routing::post};

use super::handlers::eth_send_bundle;
use super::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/eth_sendBundle", post(eth_send_bundle))
        .with_state(state)
}