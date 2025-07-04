use bx_core::Holding;
use candid::Principal;

pub async fn fetch(_principal: Principal) -> Vec<Holding> {
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    vec![Holding {
        source: "ledger".to_string(),
        token: "ICP".to_string(),
        amount: "213.45".to_string(),
        status: "liquid".to_string(),
    }]
}
