use candid::Principal;
pub async fn fetch(principal: Principal) -> Vec<super::Holding> {
    vec![super::Holding {
        source: "ledger".to_string(),
        token: "ICP".to_string(),
        amount: "213.45".to_string(),
        status: "liquid".to_string(),
    }]
}