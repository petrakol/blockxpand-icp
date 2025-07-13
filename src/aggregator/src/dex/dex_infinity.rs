use super::{DexAdapter, RewardInfo};
#[cfg(not(target_arch = "wasm32"))]
use crate::{
    lp_cache,
    utils::{format_amount, get_agent, now},
};
use async_trait::async_trait;
use crate::error::FetchError;
use bx_core::Holding;
#[cfg(not(target_arch = "wasm32"))]
use candid::Nat;
use candid::{CandidType, Principal};
#[cfg(not(target_arch = "wasm32"))]
use candid::{Decode, Encode};
#[cfg(not(target_arch = "wasm32"))]
use dashmap::DashMap;
#[cfg(not(target_arch = "wasm32"))]
use num_traits::cast::ToPrimitive;
#[cfg(not(target_arch = "wasm32"))]
use once_cell::sync::Lazy;
use serde::Deserialize;

pub struct InfinityAdapter;

#[cfg(not(target_arch = "wasm32"))]
pub fn clear_cache() {
    META_CACHE.clear();
}

#[derive(CandidType, Deserialize)]
struct VaultPosition {
    ledger: Principal,
    subaccount: Vec<u8>,
}

#[cfg(not(target_arch = "wasm32"))]
static META_CACHE: Lazy<DashMap<Principal, (String, u8, u64)>> = Lazy::new(DashMap::new);
#[cfg(not(target_arch = "wasm32"))]
const META_TTL_NS: u64 = crate::utils::DAY_NS; // 24h

#[async_trait]
impl DexAdapter for InfinityAdapter {
    async fn fetch_positions(&self, principal: Principal) -> Result<Vec<Holding>, FetchError> {
        fetch_positions_impl(principal).await
    }

    async fn claimable_rewards(&self, _principal: Principal) -> Result<Vec<RewardInfo>, FetchError> {
        Ok(Vec::new())
    }

    #[cfg(feature = "claim")]
    async fn claim_rewards(&self, _principal: Principal) -> Result<u64, String> {
        Ok(0)
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_positions_impl(principal: Principal) -> Result<Vec<Holding>, FetchError> {
    let vault_id = match crate::utils::env_principal("INFINITY_VAULT") {
        Some(p) => p,
        None => return Err(FetchError::InvalidConfig("vault".into())),
    };
    let agent = get_agent().await;
    let arg = Encode!(&principal).unwrap();
    let bytes = match agent
        .query(&vault_id, "get_user_positions")
        .with_arg(arg)
        .call()
        .await
    {
        Ok(b) => b,
        Err(e) => return Err(FetchError::from(e)),
    };
    let positions: Vec<VaultPosition> = Decode!(&bytes, Vec<VaultPosition>).unwrap_or_default();
    let height = crate::utils::dex_block_height(&agent, vault_id)
        .await
        .unwrap_or(0);
    let holdings = lp_cache::get_or_fetch(principal, "infinity", height, || async {
        let mut temp = Vec::new();
        for pos in positions {
            let (symbol, decimals) = match fetch_meta(&agent, pos.ledger).await {
                Some(v) => v,
                None => continue,
            };
            let bal = match balance_of(&agent, pos.ledger, vault_id, pos.subaccount.clone()).await {
                Some(n) => n,
                None => continue,
            };
            temp.push(Holding {
                source: "InfinitySwap".into(),
                token: symbol,
                amount: format_amount(bal, decimals),
                status: "lp_escrow".into(),
            });
        }
        temp
    })
    .await;
    Ok(holdings)
}

#[cfg(target_arch = "wasm32")]
async fn fetch_positions_impl(_principal: Principal) -> Result<Vec<Holding>, FetchError> {
    Ok(Vec::new())
}

#[cfg(not(target_arch = "wasm32"))]
async fn balance_of(
    agent: &ic_agent::Agent,
    ledger: Principal,
    owner: Principal,
    sub: Vec<u8>,
) -> Option<Nat> {
    #[derive(CandidType)]
    struct Account {
        owner: Principal,
        subaccount: Option<Vec<u8>>,
    }
    let arg = Encode!(&Account {
        owner,
        subaccount: Some(sub),
    })
    .unwrap();
    let bytes = agent
        .query(&ledger, "icrc1_balance_of")
        .with_arg(arg)
        .call()
        .await
        .ok()?;
    Decode!(&bytes, Nat).ok()
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_meta(agent: &ic_agent::Agent, ledger: Principal) -> Option<(String, u8)> {
    if let Some(e) = META_CACHE.get(&ledger) {
        if e.value().2 > now() {
            return Some((e.value().0.clone(), e.value().1));
        }
    }
    let arg = Encode!().unwrap();
    let bytes = agent
        .query(&ledger, "icrc1_metadata")
        .with_arg(arg)
        .call()
        .await
        .ok()?;
    let items: Vec<(String, candid::types::value::IDLValue)> =
        Decode!(&bytes, Vec<(String, candid::types::value::IDLValue)>).ok()?;
    let mut symbol = String::new();
    let mut decimals = 0u8;
    use candid::types::value::IDLValue as Val;
    for (k, v) in items {
        match (k.as_str(), v) {
            ("icrc1:symbol", Val::Text(s)) => symbol = s,
            ("icrc1:decimals", Val::Nat(n)) => decimals = n.0.to_u64().unwrap_or(0) as u8,
            ("icrc1:decimals", Val::Nat8(n)) => decimals = n,
            ("icrc1:decimals", Val::Nat16(n)) => decimals = n as u8,
            ("icrc1:decimals", Val::Nat32(n)) => decimals = n as u8,
            ("icrc1:decimals", Val::Nat64(n)) => decimals = n as u8,
            _ => {}
        }
    }
    META_CACHE.insert(ledger, (symbol.clone(), decimals, now() + META_TTL_NS));
    Some((symbol, decimals))
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;
    use quickcheck_macros::quickcheck;

    #[tokio::test]
    async fn empty_without_env() {
        std::env::remove_var("INFINITY_VAULT");
        let adapter = InfinityAdapter;
        let res = adapter.fetch_positions(Principal::anonymous()).await;
        assert!(matches!(res, Err(FetchError::InvalidConfig(_))));
    }

    #[quickcheck]
    fn fuzz_decode_position(data: Vec<u8>) -> bool {
        let _ = Decode!(&data, Vec<VaultPosition>);
        true
    }
}
