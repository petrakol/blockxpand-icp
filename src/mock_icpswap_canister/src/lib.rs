use candid::{CandidType, Principal};
use ic_cdk_macros::{query, update};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use serde::Deserialize;
use candid::Nat;

#[derive(CandidType, Deserialize, Clone)]
struct UserPositionInfoWithTokenAmount {
    id: u64,
    token0_amount: u64,
    token1_amount: u64,
}

#[derive(CandidType, Deserialize, Clone)]
struct Token {
    address: String,
    standard: String,
}

#[derive(CandidType, Deserialize, Clone)]
struct PoolData {
    key: String,
    token0: Token,
    token1: Token,
    fee: u64,
    tickSpacing: i32,
    canister_id: Principal,
}

static HEIGHT: Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(0));

#[candid::candid_method(query)]
#[query]
fn get_user_positions_by_principal(_p: Principal) -> Vec<UserPositionInfoWithTokenAmount> {
    vec![UserPositionInfoWithTokenAmount {
        id: 1,
        token0_amount: 500_000_000,
        token1_amount: 100_000_000,
    }]
}

#[derive(CandidType, Deserialize)]
struct PoolMetadata {
    token0_decimals: u8,
    token1_decimals: u8,
}

#[candid::candid_method(query)]
#[query]
fn metadata() -> PoolMetadata {
    PoolMetadata {
        token0_decimals: 8,
        token1_decimals: 8,
    }
}

#[candid::candid_method(query)]
#[query]
fn get_pools() -> Vec<PoolData> {
    vec![PoolData {
        key: "MOCK/ICP".to_string(),
        token0: Token {
            address: "mock0".to_string(),
            standard: "ICRC1".to_string(),
        },
        token1: Token {
            address: "mock1".to_string(),
            standard: "ICRC1".to_string(),
        },
        fee: 0,
        tickSpacing: 1,
        canister_id: ic_cdk::api::id(),
    }]
}

#[candid::candid_method(query)]
#[query]
fn block_height() -> u64 {
    *HEIGHT.lock().unwrap()
}

#[candid::candid_method(update)]
#[update]
fn advance_block() {
    let mut h = HEIGHT.lock().unwrap();
    *h += 1;
}

#[candid::candid_method(update)]
#[update]
async fn claim(p: Principal, ledger: Principal) -> u64 {
    let _ : () = ic_cdk::call(ledger, "credit", (p, Nat::from(50_000_000u64))).await.unwrap();
    10_000
}

ic_cdk::export_candid!();
