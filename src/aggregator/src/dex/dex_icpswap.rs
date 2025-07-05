use candid::Principal;
use bx_core::Holding;
use super::{DexAdapter, RewardInfo};
use async_trait::async_trait;

pub struct IcpswapAdapter;

#[async_trait]
impl DexAdapter for IcpswapAdapter {
    async fn fetch_positions(&self, _principal: Principal) -> Vec<Holding> {
        vec![Holding {
            source: "ICPSwap".to_string(),
            token: "ICP".to_string(),
            amount: "57.32".to_string(),
            status: "lp_escrow".to_string(),
        }]
    }

    async fn claimable_rewards(&self, _principal: Principal) -> Vec<RewardInfo> {
        vec![RewardInfo {
            token: "ICP".to_string(),
            amount: "0.12".to_string(),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;

    #[tokio::test]
    async fn stub_data() {
        let adapter = IcpswapAdapter;
        let pos = adapter.fetch_positions(Principal::anonymous()).await;
        assert_eq!(pos.len(), 1);
        let rewards = adapter.claimable_rewards(Principal::anonymous()).await;
        assert_eq!(rewards.len(), 1);
    }
}
