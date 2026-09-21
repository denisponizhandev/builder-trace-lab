use super::jsonrpc_types::{SendBundleParams};

pub enum RpcReject {
    BadJsonRpc,
    WrongMethod,
    EmptyTxs
}

pub fn validate_eth_send_bundle(jsonrpc: &str, method: &str, p: &SendBundleParams) -> Result<(), RpcReject> {
    if jsonrpc != "2.0" { return Err(RpcReject::BadJsonRpc); }
    if method != "eth_sendBundle" { return Err(RpcReject::WrongMethod); }
    if p.txs.is_empty() { return Err(RpcReject::EmptyTxs); }
    
    Ok(())
}