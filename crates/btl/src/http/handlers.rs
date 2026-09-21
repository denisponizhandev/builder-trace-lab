use axum::{extract::State, Json, http::StatusCode};
use chrono::{Utc};

use crate::pipeline::message::{PipelineMessage, AcceptedBundle};
use crate::domain::bundle::{parse_hex_u64, lab_bundle_hash};

use super::state::HttpState;
use super::jsonrpc_types::{JsonRpcRequest, SendBundleParams, JsonRpcSuccessResponse, SendBundleResult, JsonRpcErrorResponse, JsonRpcErrorBody, SumbitOrderOutcome};
use super::validate::{validate_eth_send_bundle, RpcReject};

pub async fn eth_send_bundle(
    State(state): State<HttpState>, 
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

    let msg = PipelineMessage::BundleAccepted(AcceptedBundle::new(
        bundle_hash.clone(),
        target_block,
        tx_count,
        Utc::now()
    ));

    // !!! the show begins here

    if let Err(_) = state.tx.send(msg).await {
        return SumbitOrderOutcome::RpcError(JsonRpcErrorResponse {
            jsonrpc: "2.0".into(),
            error: JsonRpcErrorBody {
                code: -32603,
                message: "internal error".into()
            },
            id: id.clone(),
        });
    }

    SumbitOrderOutcome::Ok(JsonRpcSuccessResponse {
        jsonrpc: "2.0".into(),
        result: SendBundleResult { bundle_hash },
        id: id.clone(),
    })
}