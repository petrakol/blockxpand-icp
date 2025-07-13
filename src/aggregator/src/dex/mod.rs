use crate::error::FetchError;
use async_trait::async_trait;
use bx_core::Holding;
use candid::Principal;

#[derive(Debug, Clone, PartialEq)]
pub struct RewardInfo {
    pub token: String,
    pub amount: String,
}

#[async_trait]
pub trait DexAdapter: Send + Sync {
    async fn fetch_positions(&self, principal: Principal) -> Result<Vec<Holding>, FetchError>;
    async fn claimable_rewards(
        &self,
        _principal: Principal,
    ) -> Result<Vec<RewardInfo>, FetchError> {
        Ok(Vec::new())
    }
    #[cfg(feature = "claim")]
    async fn claim_rewards(&self, _principal: Principal) -> Result<u64, String> {
        Ok(0)
    }
}

pub mod dex_icpswap;
pub mod dex_infinity;
pub mod dex_sonic;
pub mod sns_adapter;
