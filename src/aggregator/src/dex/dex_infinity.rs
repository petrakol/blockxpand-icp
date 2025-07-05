use super::{DexAdapter, RewardInfo};
use async_trait::async_trait;
use bx_core::Holding;
use candid::Principal;

pub struct InfinityAdapter;

#[async_trait]
impl DexAdapter for InfinityAdapter {
    async fn fetch_positions(&self, _principal: Principal) -> Vec<Holding> {
        vec![Holding {
            source: "InfinitySwap".to_string(),
            token: "ABC".to_string(),
            amount: "10".to_string(),
            status: "lp_escrow".to_string(),
        }]
    }

    async fn claimable_rewards(&self, _principal: Principal) -> Vec<RewardInfo> {
        vec![RewardInfo {
            token: "ABC".to_string(),
            amount: "0.5".to_string(),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;

    #[tokio::test]
    async fn stub_data() {
        let adapter = InfinityAdapter;
        assert_eq!(
            adapter.fetch_positions(Principal::anonymous()).await.len(),
            1
        );
        assert_eq!(
            adapter
                .claimable_rewards(Principal::anonymous())
                .await
                .len(),
            1
        );
    }
}
