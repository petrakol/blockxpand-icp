use candid::Principal;
pub async fn fetch(principal: Principal) -> Vec<super::Holding> {
    vec![
        super::Holding {
            source: "ICPSwap".to_string(),
            token: "ICP".to_string(),
            amount: "57.32".to_string(),
            status: "lp_escrow".to_string(),
        },
        super::Holding {
            source: "Sonic".to_string(),
            token: "ckBTC".to_string(),
            amount: "0.041".to_string(),
            status: "lp_escrow".to_string(),
        },
    ]
}