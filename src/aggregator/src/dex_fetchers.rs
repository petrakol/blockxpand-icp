use crate::dex::dex_icpswap::IcpswapAdapter;
use crate::dex::dex_infinity::InfinityAdapter;
use crate::dex::dex_sonic::SonicAdapter;
use crate::dex::DexAdapter;
use bx_core::Holding;
use candid::Principal;
use futures::future::join_all;

#[cfg(target_arch = "wasm32")]
async fn sleep_ms(_: u64) {}

#[cfg(not(target_arch = "wasm32"))]
async fn sleep_ms(ms: u64) {
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
}

pub async fn fetch(principal: Principal) -> Vec<Holding> {
    sleep_ms(10).await;
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
