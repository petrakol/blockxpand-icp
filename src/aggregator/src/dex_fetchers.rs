use bx_core::Holding;
use candid::{Decode, Encode, Principal};
use ic_agent::Agent;

async fn agent() -> Agent {
    let url = std::env::var("IC_URL").unwrap_or_else(|_| "https://ic0.app".into());
    let agent = Agent::builder()
        .with_url(url)
        .build()
        .expect("create agent");
    let _ = agent.fetch_root_key().await;
    agent
}

fn dex_canisters() -> Vec<(Principal, &'static str)> {
    vec![
        (
            std::env::var("ICPSWAP_CANISTER_ID")
                .ok()
                .and_then(|s| Principal::from_text(&s).ok())
                .unwrap_or_else(|| Principal::from_text("aaaaa-aa").unwrap()),
            "ICPSwap",
        ),
        (
            std::env::var("SONIC_CANISTER_ID")
                .ok()
                .and_then(|s| Principal::from_text(&s).ok())
                .unwrap_or_else(|| Principal::from_text("aaaaa-aa").unwrap()),
            "Sonic",
        ),
        (
            std::env::var("INFINITY_CANISTER_ID")
                .ok()
                .and_then(|s| Principal::from_text(&s).ok())
                .unwrap_or_else(|| Principal::from_text("aaaaa-aa").unwrap()),
            "InfinitySwap",
        ),
    ]
}

pub async fn fetch(principal: Principal) -> Vec<Holding> {
    let agent = agent().await;
    let mut out = Vec::new();
    for (canister, name) in dex_canisters() {
        let args = Encode!(&principal).expect("encode");
        let res = agent
            .query(&canister, "get_user_position")
            .with_arg(args)
            .call()
            .await
            .unwrap_or_default();
        let amount: u128 = Decode!(&res, u128).unwrap_or_default();
        if amount > 0 {
            out.push(Holding {
                source: name.to_string(),
                token: "ICP".to_string(),
                amount: amount.to_string(),
                status: "lp_escrow".to_string(),
            });
        }
    }
    out
}
