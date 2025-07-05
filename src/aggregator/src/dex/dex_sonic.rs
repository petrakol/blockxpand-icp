use candid::Principal;
use bx_core::Holding;
use super::{DexAdapter, RewardInfo};
use async_trait::async_trait;

pub struct SonicAdapter;

#[async_trait]
impl DexAdapter for SonicAdapter {
    async fn fetch_positions(&self, _principal: Principal) -> Vec<Holding> {
        vec![Holding {
            source: "Sonic".to_string(),
            token: "ckBTC".to_string(),
            amount: "0.041".to_string(),
            status: "lp_escrow".to_string(),
        }]
    }

    async fn claimable_rewards(&self, _principal: Principal) -> Vec<RewardInfo> {
        vec![RewardInfo {
            token: "ckBTC".to_string(),
            amount: "0.001".to_string(),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;

    #[tokio::test]
    async fn stub_data() {
        let adapter = SonicAdapter;
        assert_eq!(adapter.fetch_positions(Principal::anonymous()).await.len(), 1);
        assert_eq!(adapter.claimable_rewards(Principal::anonymous()).await.len(), 1);
    }
}
