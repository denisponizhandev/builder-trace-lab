use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest<T> {
    pub jsonrpc: String,
    pub method: String,
    pub params: Vec<T>,
    pub id: serde_json::Value
}

#[derive(Debug, Deserialize)]
pub struct SendBundleParams {
    pub txs: Vec<String>,
    #[serde(rename = "blockNumber")]
    pub block_number: String
}

#[derive(Debug, Serialize)]
pub struct JsonRpcSuccessResponse<T> {
    pub jsonrpc: String,
    pub result: T,
    pub id: serde_json::Value
}

#[derive(Debug, Serialize)]
pub struct SendBundleResult {
    #[serde(rename = "bundleHash")]
    pub bundle_hash: String
}

#[derive(Serialize)]
pub struct JsonRpcErrorBody {
    pub code: i32,
    pub message: String
}

#[derive(Serialize)]
pub struct JsonRpcErrorResponse {
    pub jsonrpc: String,
    pub error: JsonRpcErrorBody,
    pub id: serde_json::Value
}

pub enum SumbitOrderOutcome<T> {
    Ok(JsonRpcSuccessResponse<T>),
    RpcError(JsonRpcErrorResponse)
}

impl<T: Serialize> IntoResponse for SumbitOrderOutcome<T> {
    fn into_response(self) -> Response {
        match self {
            SumbitOrderOutcome::Ok(ok) => (StatusCode::OK, Json(ok)).into_response(),
            SumbitOrderOutcome::RpcError(err) => (StatusCode::OK, Json(err)).into_response()
        }
    }
}