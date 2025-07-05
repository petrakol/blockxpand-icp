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
    async fn fetch_positions(&self, principal: Principal) -> Vec<Holding>;
    async fn claimable_rewards(&self, principal: Principal) -> Vec<RewardInfo>;
}

pub mod dex_icpswap;
pub mod dex_infinity;
pub mod dex_sonic;
