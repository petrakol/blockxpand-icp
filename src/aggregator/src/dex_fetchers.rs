use crate::dex::dex_icpswap::IcpswapAdapter;
use crate::dex::dex_infinity::InfinityAdapter;
use crate::dex::dex_sonic::SonicAdapter;
use crate::dex::DexAdapter;
use crate::error::FetchError;
use bx_core::Holding;
use candid::Principal;
use futures::future::join_all;

#[cfg(target_arch = "wasm32")]
async fn pause() {}

#[cfg(not(target_arch = "wasm32"))]
async fn pause() {
    tokio::task::yield_now().await;
}

pub async fn fetch(principal: Principal) -> Result<Vec<Holding>, FetchError> {
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
    let results = join_all(tasks).await;
    let mut out = Vec::new();
    for r in results {
        match r {
            Ok(v) => out.extend(v),
            Err(e) => return Err(e),
        }
    }
    Ok(out)
}
