use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn parse_hex_u64(hex: &str) -> Result<i64, std::num::ParseIntError> {
    let s = hex.strip_prefix("0x").unwrap_or(hex);
    u64::from_str_radix(s, 16).map(|n| n as i64)
}

pub fn lab_bundle_hash(txs: &[String]) -> String {
    let mut h = DefaultHasher::new();
    txs.hash(&mut h);
    format!("0x{:016x}", h.finish())
}