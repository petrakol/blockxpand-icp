use bx_core::Holding;
use candid::Principal;

#[cfg(target_arch = "wasm32")]
async fn sleep_ms(_: u64) {}

#[cfg(not(target_arch = "wasm32"))]
async fn sleep_ms(ms: u64) {
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
}

pub async fn fetch(_principal: Principal) -> Vec<Holding> {
    sleep_ms(10).await;
    vec![
        Holding {
            source: "ICPSwap".to_string(),
            token: "ICP".to_string(),
            amount: "57.32".to_string(),
            status: "lp_escrow".to_string(),
        },
        Holding {
            source: "Sonic".to_string(),
            token: "ckBTC".to_string(),
            amount: "0.041".to_string(),
            status: "lp_escrow".to_string(),
        },
    ]
}
