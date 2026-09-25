use std::time::Instant;

use axum::{
    extract::State,
    http::header,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use tracing::info;

use crate::domain::bundle::{lab_bundle_hash, parse_hex_u64};
use crate::domain::job::BundleSimulationJob;
use crate::metrics::app_metrics::AppMetrics;

use crate::admission::bundle::AdmitResult;

use super::jsonrpc_types::{
    JsonRpcErrorBody, JsonRpcErrorResponse, JsonRpcRequest, JsonRpcSuccessResponse,
    SendBundleParams, SendBundleResult, SumbitOrderOutcome,
};
use super::state::AppState;
use super::validate::{validate_eth_send_bundle, RpcReject};

/// Records `http_response_duration_seconds` when the handler returns (any outcome).
struct ObserveHttpOnDrop {
    started: Instant,
    metrics: AppMetrics,
}

impl Drop for ObserveHttpOnDrop {
    fn drop(&mut self) {
        self.metrics.observe_http_response_duration_seconds(
            self.started.elapsed().as_secs_f64(),
        );
    }
}

pub async fn eth_send_bundle(
    State(state): State<AppState>,
    Json(mut body): Json<JsonRpcRequest<SendBundleParams>>,
) -> SumbitOrderOutcome<SendBundleResult> {
    let _http_observe = ObserveHttpOnDrop {
        started: Instant::now(),
        metrics: state.metrics.clone(),
    };

    let id = body.id.clone();

    if body.params.len() != 1 {
        return SumbitOrderOutcome::RpcError(JsonRpcErrorResponse {
            jsonrpc: "2.0".into(),
            error: JsonRpcErrorBody {
                code: -32602,
                message: "invalid params".into(),
            },
            id: id.clone(),
        });
    }

    let params = body.params.remove(0);

    if let Err(err) = validate_eth_send_bundle(&body.jsonrpc, &body.method, &params) {
        match err {
            RpcReject::BadJsonRpc => {
                return SumbitOrderOutcome::RpcError(JsonRpcErrorResponse {
                    jsonrpc: "2.0".into(),
                    error: JsonRpcErrorBody {
                        code: -32600,
                        message: "invalid request".into(),
                    },
                    id: id.clone(),
                });
            }
            RpcReject::WrongMethod => {
                return SumbitOrderOutcome::RpcError(JsonRpcErrorResponse {
                    jsonrpc: "2.0".into(),
                    error: JsonRpcErrorBody {
                        code: -32601,
                        message: "method not found".into(),
                    },
                    id: id.clone(),
                });
            }
            RpcReject::EmptyTxs => {
                return SumbitOrderOutcome::RpcError(JsonRpcErrorResponse {
                    jsonrpc: "2.0".into(),
                    error: JsonRpcErrorBody {
                        code: -32602,
                        message: "invalid params".into(),
                    },
                    id: id.clone(),
                });
            }
        }
    }

    // should be moved to domain layer
    let bundle_hash = lab_bundle_hash(&params.txs);
    let target_block = match parse_hex_u64(&params.block_number) {
        Ok(v) => v,
        Err(_) => {
            return SumbitOrderOutcome::RpcError(JsonRpcErrorResponse {
                jsonrpc: "2.0".into(),
                error: JsonRpcErrorBody {
                    code: -32602,
                    message: "invalid block number".into(),
                },
                id: id.clone(),
            });
        }
    };

    let tx_count = match params.txs.len().try_into() {
        Ok(v) => v,
        Err(_) => {
            return SumbitOrderOutcome::RpcError(JsonRpcErrorResponse {
                jsonrpc: "2.0".into(),
                error: JsonRpcErrorBody {
                    code: -32602,
                    message: "invalid params".into(),
                },
                id: id.clone(),
            });
        }
    };

    info!(bundle_hash = %bundle_hash, "bundle received");

    let job = BundleSimulationJob {
        bundle_hash: bundle_hash.clone(),
        target_block,
        tx_count,
        received_at: Utc::now(),
    };

    state.metrics.record_received();

    match state.admission.admit(job).await {
        AdmitResult::Accepted => SumbitOrderOutcome::Ok(JsonRpcSuccessResponse {
            jsonrpc: "2.0".into(),
            result: SendBundleResult { bundle_hash },
            id: id.clone(),
        }),
        AdmitResult::RejectedQueueFull => SumbitOrderOutcome::RpcError(JsonRpcErrorResponse {
            jsonrpc: "2.0".into(),
            error: JsonRpcErrorBody {
                code: -32603,
                message: "overload".into(),
            },
            id: id.clone(),
        }),
        AdmitResult::RejectedSubsystemDown => SumbitOrderOutcome::RpcError(JsonRpcErrorResponse {
            jsonrpc: "2.0".into(),
            error: JsonRpcErrorBody {
                code: -32603,
                message: "internal system error".into(),
            },
            id: id.clone(),
        }),
    }
}

pub async fn prometheus_metrics(State(state): State<AppState>) -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")],
        state.metrics.encode(),
    )
}
