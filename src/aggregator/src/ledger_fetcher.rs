use candid::Principal;
use bx_core::Holding;

pub async fn fetch(_principal: Principal) -> Vec<Holding> {
    vec![Holding {
        source: "ledger".to_string(),
        token: "ICP".to_string(),
        amount: "213.45".to_string(),
        status: "liquid".to_string(),
    }]
}
