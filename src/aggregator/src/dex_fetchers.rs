use bx_core::Holding;
use candid::Principal;
use tokio::time::{sleep, Duration};

pub async fn fetch(_principal: Principal) -> Vec<Holding> {
    sleep(Duration::from_millis(5)).await;
    vec![
        Holding {
            source: "ICPSwap".into(),
            token: "ICP".into(),
            amount: "57.32".into(),
            status: "lp_escrow".into(),
        },
        Holding {
            source: "Sonic".into(),
            token: "ckBTC".into(),
            amount: "0.041".into(),
            status: "lp_escrow".into(),
        },
    ]
}
