use crate::dex::dex_icpswap::IcpswapAdapter;
use crate::dex::dex_infinity::InfinityAdapter;
use crate::dex::dex_sonic::SonicAdapter;
use crate::dex::DexAdapter;
use once_cell::sync::Lazy;
use bx_core::Holding;
use candid::Principal;
use futures::future::join_all;

/// Cached list of all DEX adapters so they only allocate once.
static DEX_ADAPTERS: Lazy<Vec<Box<dyn DexAdapter>>> = Lazy::new(|| {
    vec![
        Box::new(IcpswapAdapter),
        Box::new(SonicAdapter),
        Box::new(InfinityAdapter),
    ]
});

pub async fn fetch(principal: Principal) -> Vec<Holding> {
    let futs = DEX_ADAPTERS.iter().map(|a| async move { a.fetch_positions(principal).await });
    join_all(futs).await.into_iter().flatten().collect()
}
