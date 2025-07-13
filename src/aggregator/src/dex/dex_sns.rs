use super::{DexAdapter, RewardInfo};
use async_trait::async_trait;
use crate::error::FetchError;
use bx_core::Holding;
use candid::{Nat, Principal};
#[cfg(not(target_arch = "wasm32"))]
use crate::utils::{format_amount, get_agent};
#[cfg(not(target_arch = "wasm32"))]
use candid::{Decode, Encode};
use num_traits::cast::ToPrimitive;

pub struct SnsAdapter;

#[cfg(not(target_arch = "wasm32"))]
pub fn clear_cache() {}

#[cfg(target_arch = "wasm32")]
pub fn clear_cache() {}

#[async_trait]
impl DexAdapter for SnsAdapter {
    async fn fetch_positions(&self, principal: Principal) -> Result<Vec<Holding>, FetchError> {
        fetch_positions_impl(principal).await
    }

    async fn claimable_rewards(&self, principal: Principal) -> Result<Vec<RewardInfo>, FetchError> {
        claimable_impl(principal).await
    }

    #[cfg(feature = "claim")]
    async fn claim_rewards(&self, principal: Principal) -> Result<u64, String> {
        claim_impl(principal).await
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_positions_impl(principal: Principal) -> Result<Vec<Holding>, FetchError> {
    // Reuse the claimable endpoint for now
    let rewards = claimable_impl(principal).await?;
    Ok(rewards
        .into_iter()
        .map(|r| Holding {
            source: "SNS".into(),
            token: r.token,
            amount: r.amount,
            status: "pending".into(),
        })
        .collect())
}

#[cfg(target_arch = "wasm32")]
async fn fetch_positions_impl(principal: Principal) -> Result<Vec<Holding>, FetchError> {
    let rewards = claimable_impl(principal).await?;
    Ok(rewards
        .into_iter()
        .map(|r| Holding {
            source: "SNS".into(),
            token: r.token,
            amount: r.amount,
            status: "pending".into(),
        })
        .collect())
}

#[cfg(not(target_arch = "wasm32"))]
async fn claimable_impl(principal: Principal) -> Result<Vec<RewardInfo>, FetchError> {
    let dist = match crate::utils::env_principal("SNS_DISTRIBUTOR") {
        Some(p) => p,
        None => return Err(FetchError::InvalidConfig("distributor".into())),
    };
    let agent = get_agent().await;
    let arg = Encode!(&principal).unwrap();
    let bytes = match agent
        .query(&dist, "get_pending")
        .with_arg(arg)
        .call()
        .await
    {
        Ok(b) => b,
        Err(e) => return Err(FetchError::from(e)),
    };
    let amount: Nat = Decode!(&bytes, Nat).unwrap_or_else(|_| 0u32.into());
    let amt = amount.0.to_u64().unwrap_or(0);
    if amt == 0 {
        return Ok(Vec::new());
    }
    Ok(vec![RewardInfo { token: "SNS".into(), amount: format_amount(amount, 8) }])
}

#[cfg(target_arch = "wasm32")]
async fn claimable_impl(principal: Principal) -> Result<Vec<RewardInfo>, FetchError> {
    let dist = crate::utils::env_principal("SNS_DISTRIBUTOR")
        .ok_or_else(|| FetchError::InvalidConfig("distributor".into()))?;
    let (amount,): (Nat,) = ic_cdk::api::call::call(dist, "get_pending", (principal,))
        .await
        .map_err(|e| FetchError::Network(format!("{e:?}")))?;
    let amt = amount.0.to_u64().unwrap_or(0);
    if amt == 0 {
        return Ok(Vec::new());
    }
    Ok(vec![RewardInfo { token: "SNS".into(), amount: crate::utils::format_amount(amount, 8) }])
}

#[cfg(all(feature = "claim", not(target_arch = "wasm32")))]
async fn claim_impl(principal: Principal) -> Result<u64, String> {
    let dist = crate::utils::env_principal("SNS_DISTRIBUTOR")
        .ok_or_else(|| "distributor missing".to_string())?;
    let agent = get_agent().await;
    let arg = Encode!(&principal).unwrap();
    let bytes = agent
        .update(&dist, "claim")
        .with_arg(arg)
        .call_and_wait()
        .await
        .map_err(|e| e.to_string())?;
    let claimed: Nat = Decode!(&bytes, Nat).map_err(|e| e.to_string())?;
    Ok(claimed.0.to_u64().unwrap_or(0))
}

#[cfg(all(feature = "claim", target_arch = "wasm32"))]
async fn claim_impl(principal: Principal) -> Result<u64, String> {
    let dist = crate::utils::env_principal("SNS_DISTRIBUTOR").ok_or("distributor missing")?;
    let (amt,): (Nat,) = ic_cdk::api::call::call(dist, "claim", (principal,))
        .await
        .map_err(|e| format!("{e:?}"))?;
    Ok(amt.0.to_u64().unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;

    #[tokio::test]
    async fn empty_without_env() {
        std::env::remove_var("SNS_DISTRIBUTOR");
        let adapter = SnsAdapter;
        let res = adapter.fetch_positions(Principal::anonymous()).await;
        assert!(matches!(res, Err(FetchError::InvalidConfig(_))));
    }
}
