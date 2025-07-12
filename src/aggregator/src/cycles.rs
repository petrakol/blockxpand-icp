#[cfg(target_arch = "wasm32")]
use candid::Principal;
#[cfg(target_arch = "wasm32")]
use ic_cdk::api::{call::call, canister_balance128, time};
#[cfg(target_arch = "wasm32")]
use once_cell::sync::Lazy;
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static LAST_CHECK: RefCell<u64> = RefCell::new(0);
    static BACKOFF_UNTIL: RefCell<u64> = RefCell::new(0);
    static FAILURES: RefCell<u8> = RefCell::new(0);
    static LOG: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

#[cfg(target_arch = "wasm32")]
static WALLET: Lazy<Option<Principal>> =
    Lazy::new(|| option_env!("CYCLES_WALLET").and_then(|s| Principal::from_text(s).ok()));

#[cfg(target_arch = "wasm32")]
const MIN_BALANCE: u128 = 500_000_000_000; // 0.5 T

#[cfg(target_arch = "wasm32")]
fn max_backoff_minutes() -> u64 {
    option_env!("CYCLE_BACKOFF_MAX")
        .and_then(|s| s.parse().ok())
        .unwrap_or(60)
}

#[cfg(target_arch = "wasm32")]
pub async fn tick() {
    use crate::utils::MINUTE_NS;
    let now = time();
    let allowed = BACKOFF_UNTIL.with(|b| now >= *b.borrow());
    if !allowed {
        return;
    }
    let run = LAST_CHECK.with(|c| {
        if now - *c.borrow() >= MINUTE_NS {
            *c.borrow_mut() = now;
            true
        } else {
            false
        }
    });
    if !run {
        return;
    }
    if canister_balance128() < MIN_BALANCE {
        if let Some(w) = *WALLET {
            let before = canister_balance128();
            let res: Result<(), _> = call(w, "wallet_receive", ()).await;
            let after = canister_balance128();
            if res.is_ok() && after > before {
                FAILURES.with(|f| *f.borrow_mut() = 0);
                BACKOFF_UNTIL.with(|b| *b.borrow_mut() = now);
                LOG.with(|l| l.borrow_mut().push(format!("{now}: refilled to {after}")));
            } else {
                let fails = FAILURES.with(|f| {
                    let mut v = f.borrow_mut();
                    *v = v.saturating_add(1);
                    *v
                });
                let backoff_m = (1u64 << fails.min(6) as u64).min(max_backoff_minutes().max(1));
                BACKOFF_UNTIL.with(|b| *b.borrow_mut() = now + backoff_m * MINUTE_NS);
                LOG.with(|l| l.borrow_mut().push(format!("{now}: refill failed, backoff {backoff_m}m")));
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn log() -> Vec<String> {
    LOG.with(|l| l.borrow().clone())
}

#[cfg(target_arch = "wasm32")]
pub fn take_log() -> Vec<String> {
    LOG.with(|l| std::mem::take(&mut *l.borrow_mut()))
}

#[cfg(target_arch = "wasm32")]
pub fn set_log(log: Vec<String>) {
    LOG.with(|l| *l.borrow_mut() = log);
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn tick() {}
#[cfg(not(target_arch = "wasm32"))]
pub fn log() -> Vec<String> {
    Vec::new()
}
#[cfg(not(target_arch = "wasm32"))]
pub fn take_log() -> Vec<String> {
    Vec::new()
}
#[cfg(not(target_arch = "wasm32"))]
pub fn set_log(_: Vec<String>) {}
