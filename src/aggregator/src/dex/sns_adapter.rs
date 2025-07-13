use super::{DexAdapter, RewardInfo};
use crate::error::FetchError;
#[cfg(not(target_arch = "wasm32"))]
use crate::utils::{format_amount, get_agent};
use async_trait::async_trait;
use bx_core::Holding;
use candid::{CandidType, Nat, Principal};
#[cfg(not(target_arch = "wasm32"))]
use candid::{Decode, Encode};
use serde::Deserialize;

pub struct SnsAdapter;

#[cfg(not(target_arch = "wasm32"))]
pub fn clear_cache() {}

#[cfg(target_arch = "wasm32")]
pub fn clear_cache() {}

#[derive(CandidType, Deserialize)]
struct Claimable {
    symbol: String,
    amount: Nat,
    decimals: u8,
}

#[async_trait]
impl DexAdapter for SnsAdapter {
    async fn fetch_positions(&self, principal: Principal) -> Result<Vec<Holding>, FetchError> {
        fetch_positions_impl(principal).await
    }

    async fn claimable_rewards(&self, principal: Principal) -> Result<Vec<RewardInfo>, FetchError> {
        let holdings = fetch_positions_impl(principal).await?;
        Ok(holdings
            .into_iter()
            .map(|h| RewardInfo {
                token: h.token,
                amount: h.amount,
            })
            .collect())
    }

    #[cfg(feature = "claim")]
    async fn claim_rewards(&self, principal: Principal) -> Result<u64, String> {
        claim_impl(principal).await
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_positions_impl(principal: Principal) -> Result<Vec<Holding>, FetchError> {
    let distro_id = match crate::utils::env_principal("SNS_DISTRIBUTOR") {
        Some(p) => p,
        None => return Err(FetchError::InvalidConfig("distributor".into())),
    };
    let agent = get_agent().await;
    let arg = Encode!(&principal).unwrap();
    let bytes = match agent
        .query(&distro_id, "get_claimable_tokens")
        .with_arg(arg)
        .call()
        .await
    {
        Ok(b) => b,
        Err(e) => return Err(FetchError::from(e)),
    };
    let claims: Vec<Claimable> = Decode!(&bytes, Vec<Claimable>).unwrap_or_default();
    let mut out = Vec::new();
    for c in claims {
        out.push(Holding {
            source: "SNS".into(),
            token: c.symbol,
            amount: format_amount(c.amount, c.decimals),
            status: "claimable".into(),
        });
    }
    Ok(out)
}

#[cfg(target_arch = "wasm32")]
async fn fetch_positions_impl(_principal: Principal) -> Result<Vec<Holding>, FetchError> {
    Ok(Vec::new())
}

#[cfg(all(feature = "claim", not(target_arch = "wasm32")))]
async fn claim_impl(principal: Principal) -> Result<u64, String> {
    let distro_id = match crate::utils::env_principal("SNS_DISTRIBUTOR") {
        Some(p) => p,
        None => return Err("distributor".into()),
    };
    let agent = get_agent().await;
    let arg = Encode!(&principal).unwrap();
    let bytes = agent
        .update(&distro_id, "claim")
        .with_arg(arg)
        .call_and_wait()
        .await
        .map_err(|e| e.to_string())?;
    Ok(Decode!(&bytes, u64).unwrap_or_default())
}

#[cfg(all(feature = "claim", target_arch = "wasm32"))]
async fn claim_impl(principal: Principal) -> Result<u64, String> {
    use ic_cdk::api::call::call;
    let distro_id = crate::utils::env_principal("SNS_DISTRIBUTOR").ok_or("distributor")?;
    let (spent,): (u64,) = call(distro_id, "claim", (principal,))
        .await
        .map_err(|(_, e)| e)?;
    Ok(spent)
}
