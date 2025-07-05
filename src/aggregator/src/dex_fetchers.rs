use crate::dex::dex_icpswap::IcpswapAdapter;
use crate::dex::dex_infinity::InfinityAdapter;
use crate::dex::dex_sonic::SonicAdapter;
use crate::dex::DexAdapter;
use bx_core::Holding;
use candid::Principal;
use futures::future::join_all;

pub async fn fetch(principal: Principal) -> Vec<Holding> {
    let adapters: Vec<Box<dyn DexAdapter>> = vec![
        Box::new(IcpswapAdapter),
        Box::new(SonicAdapter),
        Box::new(InfinityAdapter),
    ];
    let futs = adapters.into_iter().map(|a| {
        let p = principal.clone();
        async move { a.fetch_positions(p).await }
    });
    join_all(futs).await.into_iter().flatten().collect()
}
