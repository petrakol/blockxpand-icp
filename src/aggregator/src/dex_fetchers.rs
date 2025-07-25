use crate::dex::dex_icpswap::IcpswapAdapter;
use crate::dex::dex_infinity::InfinityAdapter;
use crate::dex::dex_sonic::SonicAdapter;
use crate::dex::sns_adapter::SnsAdapter;
use crate::dex::DexAdapter;
use crate::error::FetchError;
use bx_core::Holding;
use candid::Principal;
use futures::future::join_all;
#[cfg(not(target_arch = "wasm32"))]
use once_cell::sync::Lazy;
use std::collections::HashSet;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

pub const ICPSWAP: &str = "ICPSWAP_FACTORY";
pub const SONIC: &str = "SONIC_ROUTER";
pub const INFINITY: &str = "INFINITY_VAULT";
pub const SNS: &str = "SNS_DISTRIBUTOR";

static ADAPTERS: &[(&str, &dyn DexAdapter)] = &[
    (ICPSWAP, &IcpswapAdapter),
    (SONIC, &SonicAdapter),
    (INFINITY, &InfinityAdapter),
    (SNS, &SnsAdapter),
];

#[cfg(not(target_arch = "wasm32"))]
static FETCH_ADAPTER_TIMEOUT_SECS: Lazy<u64> = Lazy::new(|| {
    option_env!("FETCH_ADAPTER_TIMEOUT_SECS")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(5)
});

#[cfg(target_arch = "wasm32")]
async fn pause() {}

#[cfg(not(target_arch = "wasm32"))]
async fn pause() {
    tokio::task::yield_now().await;
}

#[cfg(not(target_arch = "wasm32"))]
async fn with_timeout<F>(fut: F) -> Result<Vec<Holding>, FetchError>
where
    F: std::future::Future<Output = Result<Vec<Holding>, FetchError>>,
{
    use tokio::time::timeout;
    match timeout(Duration::from_secs(*FETCH_ADAPTER_TIMEOUT_SECS), fut).await {
        Ok(v) => v,
        Err(_) => Err(FetchError::Network("timeout".into())),
    }
}

#[cfg(target_arch = "wasm32")]
async fn with_timeout<F>(fut: F) -> Result<Vec<Holding>, FetchError>
where
    F: std::future::Future<Output = Result<Vec<Holding>, FetchError>>,
{
    fut.await
}

pub async fn fetch_filtered(
    principal: Principal,
    list: Option<&HashSet<String>>,
) -> Result<Vec<Holding>, FetchError> {
    // allow other tasks to start before launching adapter queries
    pause().await;
    let mut adapters: Vec<&dyn DexAdapter> = Vec::new();
    let want = |name: &str| match list {
        Some(l) => l.contains(name),
        None => true,
    };
    for (name, adapter) in ADAPTERS.iter() {
        if want(name) {
            adapters.push(*adapter);
        }
    }
    let tasks = adapters
        .into_iter()
        .map(|a| async move { with_timeout(a.fetch_positions(principal)).await });
    let results = join_all(tasks).await;
    let capacity: usize = results
        .iter()
        .filter_map(|r| r.as_ref().ok())
        .map(|v| v.len())
        .sum();
    let mut out = Vec::with_capacity(capacity);
    for r in results {
        match r {
            Ok(v) => out.extend(v),
            Err(e) => return Err(e),
        }
    }
    Ok(out)
}

pub async fn fetch(principal: Principal) -> Result<Vec<Holding>, FetchError> {
    fetch_filtered(principal, None).await
}
