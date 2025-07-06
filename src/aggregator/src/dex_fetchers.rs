use crate::dex::dex_icpswap::IcpswapAdapter;
use crate::dex::dex_infinity::InfinityAdapter;
use crate::dex::dex_sonic::SonicAdapter;
use crate::dex::DexAdapter;
use bx_core::Holding;
use candid::Principal;
use futures::future::join_all;

#[cfg(target_arch = "wasm32")]
async fn pause() {}

#[cfg(not(target_arch = "wasm32"))]
async fn pause() {
    tokio::task::yield_now().await;
}

pub async fn fetch(principal: Principal) -> Vec<Holding> {
    // allow other tasks to start before launching adapter queries
    pause().await;
    let adapters: Vec<Box<dyn DexAdapter>> = vec![
        Box::new(IcpswapAdapter),
        Box::new(SonicAdapter),
        Box::new(InfinityAdapter),
    ];
    let tasks = adapters
        .into_iter()
        .map(|a| async move { a.fetch_positions(principal).await });
    join_all(tasks).await.into_iter().flatten().collect()
}
