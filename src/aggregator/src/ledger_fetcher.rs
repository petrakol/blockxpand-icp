use bx_core::Holding;
use candid::Principal;
use tokio::time::{sleep, Duration};

pub async fn fetch(_principal: Principal) -> Vec<Holding> {
    sleep(Duration::from_millis(5)).await;
    vec![Holding {
        source: "ledger".into(),
        token: "ICP".into(),
        amount: "100.0".into(),
        status: "liquid".into(),
    }]
}
