pub mod cache;
pub mod dex;
pub mod dex_fetchers;
pub mod ledger_fetcher;
pub mod neuron_fetcher;

use bx_core::Holding;
use candid::Principal;

#[cfg(target_arch = "wasm32")]
fn now() -> u64 {
    ic_cdk::api::time()
}

#[cfg(not(target_arch = "wasm32"))]
fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

#[ic_cdk_macros::query]
pub async fn get_holdings(principal: Principal) -> Vec<Holding> {
    let now = now();
    {
        let cache = cache::get_mut();
        if let Some((cached, ts)) = cache.get(&principal).cloned() {
            if now - ts < 60_000_000_000 {
                return cached;
            }
        }
    }

    let mut holdings = Vec::new();
    holdings.extend(ledger_fetcher::fetch(principal).await);
    holdings.extend(neuron_fetcher::fetch(principal).await);
    holdings.extend(dex_fetchers::fetch(principal).await);

    {
        let mut cache = cache::get_mut();
        cache.insert(principal, (holdings.clone(), now));
    }
    holdings
}
