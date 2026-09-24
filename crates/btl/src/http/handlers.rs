use axum::{extract::State, Json};
use chrono::{Utc};

use crate::domain::bundle::{parse_hex_u64, lab_bundle_hash};
use crate::domain::job::BundleSimulationJob;

use crate::admission::bundle::AdmitResult;

use super::state::AppState;
use super::jsonrpc_types::{JsonRpcRequest, SendBundleParams, JsonRpcSuccessResponse, SendBundleResult, JsonRpcErrorResponse, JsonRpcErrorBody, SumbitOrderOutcome};
use super::validate::{validate_eth_send_bundle, RpcReject};

pub async fn eth_send_bundle(
    State(state): State<AppState>, 
    Json(mut body): Json<JsonRpcRequest<SendBundleParams>>
) -> SumbitOrderOutcome<SendBundleResult> {

    let id = body.id.clone(); 

    if body.params.len() != 1 {
        return SumbitOrderOutcome::RpcError(JsonRpcErrorResponse {
            jsonrpc: "2.0".into(),
            error: JsonRpcErrorBody {
                code: -32602,
                message: "invalid params".into()
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
                        message: "invalid request".into()
                    },
                    id: id.clone(),
                });
            }
            RpcReject::WrongMethod => {
                return SumbitOrderOutcome::RpcError(JsonRpcErrorResponse {
                    jsonrpc: "2.0".into(),
                    error: JsonRpcErrorBody {
                        code: -32601,
                        message: "method not found".into()
                    },
                    id: id.clone(),
                });
            }
            RpcReject::EmptyTxs => {
                return SumbitOrderOutcome::RpcError(JsonRpcErrorResponse {
                    jsonrpc: "2.0".into(),
                    error: JsonRpcErrorBody {
                        code: -32602,
                        message: "invalid params".into()
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
                    message: "invalid block number".into()
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
                    message: "invalid params".into()
                },
                id: id.clone(),
            })
        }
    };

    let job = BundleSimulationJob {
        bundle_hash: bundle_hash.clone(),
        target_block: target_block,
        tx_count: tx_count,
        received_at: Utc::now(),
        enqueued_at: None
    };

    match state.admission.admit(job).await {
        AdmitResult::Accepted => {
            return SumbitOrderOutcome::Ok(JsonRpcSuccessResponse {
                jsonrpc: "2.0".into(),
                result: SendBundleResult { bundle_hash },
                id: id.clone(),
            });
        }
        AdmitResult::RejectedQueueFull => {
            return SumbitOrderOutcome::RpcError(JsonRpcErrorResponse {
                jsonrpc: "2.0".into(),
                error: JsonRpcErrorBody {
                    code: -32603,
                    message: "overload".into()
                },
                id: id.clone(),
            });
        }
        AdmitResult::RejectedSubsystemDown => {
            return SumbitOrderOutcome::RpcError(JsonRpcErrorResponse {
                jsonrpc: "2.0".into(),
                error: JsonRpcErrorBody {
                    code: -32603,
                    message: "internal system error".into()
                },
                id: id.clone(),
            })
        }
    }
}