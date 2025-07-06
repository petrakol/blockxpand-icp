use crate::utils::now;
use bx_core::Holding;
use candid::Principal;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::future::Future;

struct Entry {
    data: Vec<Holding>,
    height: u64,
    ts: u64,
}

static CACHE: Lazy<DashMap<(Principal, String), Entry>> = Lazy::new(DashMap::new);

const STALE_NS: u64 = 604_800_000_000_000; // one week

pub async fn get_or_fetch<F, Fut>(
    principal: Principal,
    pool: &str,
    height: u64,
    fetch: F,
) -> Vec<Holding>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Vec<Holding>>,
{
    if let Some(e) = CACHE.get(&(principal, pool.to_string())) {
        if e.height == height && now() - e.ts < STALE_NS {
            return e.data.clone();
        }
    }
    let data = fetch().await;
    let ts = now();
    CACHE.insert(
        (principal, pool.to_string()),
        Entry {
            data: data.clone(),
            height,
            ts,
        },
    );
    data
}

pub fn evict_stale() {
    let n = now();
    CACHE.retain(|_, v| n - v.ts < STALE_NS);
}

#[cfg(target_arch = "wasm32")]
pub fn schedule_eviction() {
    use std::time::Duration;
    ic_cdk_timers::set_timer_interval(Duration::from_secs(604_800), || {
        evict_stale();
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use bx_core::Holding;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test(flavor = "current_thread")]
    async fn cache_respects_height() {
        static CALLS: AtomicUsize = AtomicUsize::new(0);
        let principal = Principal::from_text("aaaaa-aa").unwrap();
        let pool = "p1";
        let h1 = 1u64;
        let v1 = get_or_fetch(principal, pool, h1, || async {
            CALLS.fetch_add(1, Ordering::SeqCst);
            vec![Holding {
                source: "x".into(),
                token: "t".into(),
                amount: "1".into(),
                status: "lp_escrow".into(),
            }]
        })
        .await;
        assert_eq!(CALLS.load(Ordering::SeqCst), 1);
        let v2 = get_or_fetch(principal, pool, h1, || async {
            CALLS.fetch_add(1, Ordering::SeqCst);
            vec![]
        })
        .await;
        assert_eq!(CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(v2, v1);
        let v3 = get_or_fetch(principal, pool, h1 + 1, || async {
            CALLS.fetch_add(1, Ordering::SeqCst);
            vec![Holding {
                source: "x".into(),
                token: "t".into(),
                amount: "2".into(),
                status: "lp_escrow".into(),
            }]
        })
        .await;
        assert_eq!(CALLS.load(Ordering::SeqCst), 2);
        assert_eq!(v3[0].amount, "2");
    }
}
