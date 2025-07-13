pub mod cache;
pub mod cert;
pub mod cycles;
pub mod dex;
pub mod dex_fetchers;
pub mod ledger_fetcher;
pub mod lp_cache;
pub mod metrics;
pub mod logging;
pub mod neuron_fetcher;
pub mod pool_registry;
pub mod utils;
pub mod warm;
pub mod error;

use crate::utils::{now, MINUTE_NS};
use bx_core::Holding;
use candid::Principal;
#[cfg(feature = "claim")]
use once_cell::sync::Lazy;
#[cfg(feature = "claim")]
static CLAIM_WALLETS: Lazy<Vec<Principal>> = Lazy::new(|| {
    option_env!("CLAIM_WALLETS")
        .unwrap_or("")
        .split(',')
        .filter_map(|s| Principal::from_text(s.trim()).ok())
        .collect()
});

async fn calculate_holdings(principal: Principal) -> Vec<Holding> {
    let (ledger, neuron, dex) = futures::join!(
        ledger_fetcher::fetch(principal),
        neuron_fetcher::fetch(principal),
        dex_fetchers::fetch(principal)
    );

    let mut holdings = Vec::new();
    holdings.extend(ledger.unwrap_or_default());
    holdings.extend(neuron);
    holdings.extend(dex.unwrap_or_default());
    holdings
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
    metrics::inc_query();
    let start = instructions();
    let now = now();
    {
        let cache = cache::get();
        if let Some(v) = cache.get(&principal) {
            let (cached, ts) = v.value().clone();
            if now - ts < MINUTE_NS {
                let used = instructions().saturating_sub(start);
                tracing::info!(
                    "get_holdings took {used} instructions ({:.2} B)",
                    used as f64 / 1_000_000_000f64
                );
                return cached;
            }
        }
    }

    let (ledger, neuron, dex) = futures::join!(
        ledger_fetcher::fetch(principal),
        neuron_fetcher::fetch(principal),
        dex_fetchers::fetch(principal)
    );

    let mut holdings = Vec::new();
    holdings.extend(ledger.unwrap_or_default());
    holdings.extend(neuron);
    holdings.extend(dex.unwrap_or_default());

    {
        cache::get().insert(principal, (holdings.clone(), now));
    }
    let used = instructions().saturating_sub(start);
    tracing::info!(
        "get_holdings took {used} instructions ({:.2} B)",
        used as f64 / 1_000_000_000f64
    );
    holdings
}

#[cfg(feature = "claim")]
#[ic_cdk_macros::update]
pub async fn claim_all_rewards(principal: Principal) -> Vec<u64> {
    metrics::inc_query();
    let caller = ic_cdk::caller();
    if caller != principal && !CLAIM_WALLETS.contains(&caller) {
        ic_cdk::api::trap("unauthorized");
    }
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
    metrics::inc_query();
    pool_registry::graphql(query)
}

#[derive(candid::CandidType, serde::Serialize)]
pub struct CertifiedHoldings {
    pub holdings: Vec<Holding>,
    #[serde(with = "serde_bytes")]
    pub certificate: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub witness: Vec<u8>,
}

#[ic_cdk_macros::update]
pub async fn refresh_holdings(principal: Principal) {
    metrics::inc_query();
    let now = now();
    let holdings = calculate_holdings(principal).await;
    cache::get().insert(principal, (holdings.clone(), now));
    cert::update(principal, &holdings);
}

#[ic_cdk_macros::query]
pub fn get_holdings_cert(principal: Principal) -> CertifiedHoldings {
    metrics::inc_query();
    let holdings = cache::get()
        .get(&principal)
        .map(|v| v.value().0.clone())
        .unwrap_or_default();
    let certificate = ic_cdk::api::data_certificate().unwrap_or_default();
    let witness = cert::witness(principal);
    CertifiedHoldings {
        holdings,
        certificate,
        witness,
    }
}

#[derive(candid::CandidType, serde::Serialize)]
pub struct Version {
    pub git_sha: &'static str,
    pub build_time: &'static str,
}

#[ic_cdk_macros::query]
pub fn get_version() -> Version {
    metrics::inc_query();
    Version {
        git_sha: option_env!("GIT_SHA").unwrap_or("unknown"),
        build_time: option_env!("BUILD_TIME").unwrap_or("unknown"),
    }
}
