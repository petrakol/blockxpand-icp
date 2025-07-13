use candid::CandidType;
use core::sync::atomic::{AtomicU64, Ordering};
use serde::Serialize;

static QUERY_COUNT: AtomicU64 = AtomicU64::new(0);
static HEARTBEAT_COUNT: AtomicU64 = AtomicU64::new(0);
static LAST_HEARTBEAT: AtomicU64 = AtomicU64::new(0);
static QUERY_INSTRUCTIONS: AtomicU64 = AtomicU64::new(0);
static CLAIM_COUNT: AtomicU64 = AtomicU64::new(0);
static CLAIM_INSTRUCTIONS: AtomicU64 = AtomicU64::new(0);

#[derive(CandidType, Serialize)]
pub struct Metrics {
    pub cycles: u128,
    pub query_count: u64,
    pub query_instructions: u64,
    pub heartbeat_count: u64,
    pub last_heartbeat: u64,
    pub claim_count: u64,
    pub claim_instructions: u64,
}

pub fn inc_query() {
    QUERY_COUNT.fetch_add(1, Ordering::Relaxed);
}

pub fn add_query_instructions(delta: u64) {
    QUERY_INSTRUCTIONS.fetch_add(delta, Ordering::Relaxed);
}

pub fn inc_claim(delta: u64, instructions: u64) {
    CLAIM_COUNT.fetch_add(delta, Ordering::Relaxed);
    CLAIM_INSTRUCTIONS.fetch_add(instructions, Ordering::Relaxed);
}

pub fn inc_heartbeat(now: u64) {
    HEARTBEAT_COUNT.fetch_add(1, Ordering::Relaxed);
    LAST_HEARTBEAT.store(now, Ordering::Relaxed);
}

pub fn get() -> Metrics {
    let cycles = if cfg!(target_arch = "wasm32") {
        ic_cdk::api::canister_balance128()
    } else {
        0
    };
    Metrics {
        cycles,
        query_count: QUERY_COUNT.load(Ordering::Relaxed),
        query_instructions: QUERY_INSTRUCTIONS.load(Ordering::Relaxed),
        heartbeat_count: HEARTBEAT_COUNT.load(Ordering::Relaxed),
        last_heartbeat: LAST_HEARTBEAT.load(Ordering::Relaxed),
        claim_count: CLAIM_COUNT.load(Ordering::Relaxed),
        claim_instructions: CLAIM_INSTRUCTIONS.load(Ordering::Relaxed),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn stable_save() -> (u64, u64, u64, u64, u64, u64) {
    (
        QUERY_COUNT.load(Ordering::Relaxed),
        QUERY_INSTRUCTIONS.load(Ordering::Relaxed),
        HEARTBEAT_COUNT.load(Ordering::Relaxed),
        LAST_HEARTBEAT.load(Ordering::Relaxed),
        CLAIM_COUNT.load(Ordering::Relaxed),
        CLAIM_INSTRUCTIONS.load(Ordering::Relaxed),
    )
}

#[cfg(target_arch = "wasm32")]
pub fn stable_restore(data: (u64, u64, u64, u64, u64, u64)) {
    QUERY_COUNT.store(data.0, Ordering::Relaxed);
    QUERY_INSTRUCTIONS.store(data.1, Ordering::Relaxed);
    HEARTBEAT_COUNT.store(data.2, Ordering::Relaxed);
    LAST_HEARTBEAT.store(data.3, Ordering::Relaxed);
    CLAIM_COUNT.store(data.4, Ordering::Relaxed);
    CLAIM_INSTRUCTIONS.store(data.5, Ordering::Relaxed);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn stable_save() -> (u64, u64, u64, u64, u64, u64) {
    (0, 0, 0, 0, 0, 0)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn stable_restore(_: (u64, u64, u64, u64, u64, u64)) {}
