use super::{DexAdapter, RewardInfo};
use async_trait::async_trait;
use bx_core::Holding;
use candid::{CandidType, Decode, Encode, Nat, Principal};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::Deserialize;

#[derive(CandidType, Deserialize)]
struct Token {
    address: String,
    standard: String,
}

#[derive(CandidType, Deserialize)]
struct PoolData {
    key: String,
    token0: Token,
    token1: Token,
    fee: Nat,
    tickSpacing: i32,
    #[serde(rename = "canisterId")]
    canister_id: Principal,
}

#[derive(CandidType, Deserialize)]
struct UserPositionInfoWithTokenAmount {
    #[serde(rename = "id")]
    id: Nat,
    #[serde(rename = "token0Amount")]
    token0_amount: Nat,
    #[serde(rename = "token1Amount")]
    token1_amount: Nat,
}

#[derive(CandidType, Deserialize, Clone)]
struct PoolMetadata {
    token0_decimals: u8,
    token1_decimals: u8,
}

static META_CACHE: Lazy<DashMap<Principal, (PoolMetadata, u64)>> = Lazy::new(DashMap::new);
const META_TTL_NS: u64 = 86_400_000_000_000; // 24h

#[cfg(not(target_arch = "wasm32"))]
async fn get_agent() -> ic_agent::Agent {
    let url = std::env::var("LEDGER_URL").unwrap_or_else(|_| "http://localhost:4943".into());
    let agent = ic_agent::Agent::builder().with_url(url).build().unwrap();
    let _ = agent.fetch_root_key().await;
    agent
}

#[async_trait]
impl DexAdapter for IcpswapAdapter {
    async fn fetch_positions(&self, principal: Principal) -> Vec<Holding> {
        fetch_positions_impl(principal).await
    }

    async fn claimable_rewards(&self, _principal: Principal) -> Vec<RewardInfo> {
        Vec::new()
    }
}

pub struct IcpswapAdapter;

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_positions_impl(principal: Principal) -> Vec<Holding> {
    let factory_id = match std::env::var("ICPSWAP_FACTORY") {
        Ok(v) => match Principal::from_text(v) {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        },
        Err(_) => return Vec::new(),
    };
    let agent = get_agent().await;
    let arg = Encode!().unwrap();
    let bytes = match agent
        .query(&factory_id, "getPools")
        .with_arg(arg)
        .call()
        .await
    {
        Ok(b) => b,
        Err(_) => return Vec::new(),
    };
    let pools: Vec<PoolData> = Decode!(&bytes, Vec<PoolData>).unwrap_or_default();
    let mut out = Vec::new();
    for pool in pools.iter() {
        let positions = match query_positions(&agent, pool.canister_id, principal).await {
            Some(v) => v,
            None => continue,
        };
        let meta = match fetch_meta(&agent, pool.canister_id).await {
            Some(m) => m,
            None => continue,
        };
        for pos in positions {
            let a0 = format_amount(pos.token0_amount, meta.token0_decimals);
            out.push(Holding {
                source: "ICPSwap".into(),
                token: pool.token0.address.clone(),
                amount: a0,
                status: "lp_escrow".into(),
            });
            let a1 = format_amount(pos.token1_amount, meta.token1_decimals);
            out.push(Holding {
                source: "ICPSwap".into(),
                token: pool.token1.address.clone(),
                amount: a1,
                status: "lp_escrow".into(),
            });
        }
    }
    out
}

#[cfg(target_arch = "wasm32")]
async fn fetch_positions_impl(_principal: Principal) -> Vec<Holding> {
    Vec::new()
}

#[cfg(not(target_arch = "wasm32"))]
async fn query_positions(
    agent: &ic_agent::Agent,
    cid: Principal,
    owner: Principal,
) -> Option<Vec<UserPositionInfoWithTokenAmount>> {
    let arg = Encode!(&owner).unwrap();
    let bytes = agent
        .query(&cid, "get_user_positions_by_principal")
        .with_arg(arg)
        .call()
        .await
        .ok()?;
    Decode!(&bytes, Vec<UserPositionInfoWithTokenAmount>).ok()
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_meta(agent: &ic_agent::Agent, cid: Principal) -> Option<PoolMetadata> {
    if let Some(entry) = META_CACHE.get(&cid) {
        if entry.value().1 > now() {
            return Some(entry.value().0.clone());
        }
    }
    let arg = Encode!().unwrap();
    let bytes = agent
        .query(&cid, "metadata")
        .with_arg(arg)
        .call()
        .await
        .ok()?;
    let meta: PoolMetadata = Decode!(&bytes, PoolMetadata).ok()?;
    META_CACHE.insert(cid, (meta.clone(), now() + META_TTL_NS));
    Some(meta)
}

#[cfg(not(target_arch = "wasm32"))]
fn format_amount(n: Nat, decimals: u8) -> String {
    use num_bigint::BigUint;
    use num_integer::Integer;
    use num_traits::cast::ToPrimitive;
    let div = BigUint::from(10u32).pow(decimals as u32);
    let (q, r) = n.0.div_rem(&div);
    let mut frac = r.to_str_radix(10);
    while frac.len() < decimals as usize {
        frac.insert(0, '0');
    }
    if decimals == 0 {
        q.to_str_radix(10)
    } else {
        format!("{}.{frac}", q.to_str_radix(10))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

#[cfg(target_arch = "wasm32")]
fn now() -> u64 {
    ic_cdk::api::time()
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::sync::Mutex;

    static LAST_QUERY: Lazy<Mutex<Vec<u8>>> = Lazy::new(|| Mutex::new(vec![]));

    #[tokio::test(flavor = "current_thread")]
    async fn fetch_positions_empty_without_env() {
        let adapter = IcpswapAdapter;
        let res = adapter.fetch_positions(Principal::anonymous()).await;
        assert!(res.is_empty());
    }
}
