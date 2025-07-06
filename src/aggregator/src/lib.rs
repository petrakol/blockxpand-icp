pub mod cache;
pub mod dex;
pub mod dex_fetchers;
pub mod ledger_fetcher;
pub mod lp_cache;
pub mod neuron_fetcher;
pub mod pool_registry;

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

#[cfg(target_arch = "wasm32")]
fn instructions() -> u64 {
    ic_cdk::api::instruction_counter()
}

#[cfg(not(target_arch = "wasm32"))]
fn instructions() -> u64 {
    0
}

#[ic_cdk_macros::query]
pub async fn get_holdings(principal: Principal) -> Vec<Holding> {
    let start = instructions();
    let now = now();
    {
        let cache = cache::get_mut();
        if let Some((cached, ts)) = cache.get(&principal).cloned() {
            if now - ts < 60_000_000_000 {
                let used = instructions().saturating_sub(start);
                ic_cdk::println!(
                    "get_holdings took {used} instructions ({:.2} B)",
                    used as f64 / 1_000_000_000f64
                );
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
    let used = instructions().saturating_sub(start);
    ic_cdk::println!(
        "get_holdings took {used} instructions ({:.2} B)",
        used as f64 / 1_000_000_000f64
    );
    holdings
}

#[cfg(feature = "claim")]
pub async fn claim_all_rewards(principal: Principal) -> Vec<u64> {
    use dex::{
        dex_icpswap::IcpswapAdapter, dex_infinity::InfinityAdapter, dex_sonic::SonicAdapter,
        DexAdapter,
    };
    let adapters: Vec<Box<dyn DexAdapter>> = vec![
        Box::new(IcpswapAdapter),
        Box::new(SonicAdapter),
        Box::new(InfinityAdapter),
    ];
    let mut spent = Vec::new();
    for a in adapters {
        if let Ok(c) = a.claim_rewards(principal).await {
            spent.push(c);
        }
    }
    spent
}

#[ic_cdk_macros::query]
pub fn pools_graphql(query: String) -> String {
    pool_registry::graphql(query)
}
