use bx_core::Holding;
use candid::{Encode, Decode, Principal};
use ic_agent::Agent;

fn ledger_canister() -> Principal {
    std::env::var("LEDGER_CANISTER_ID")
        .ok()
        .and_then(|s| Principal::from_text(&s).ok())
        .unwrap_or_else(|| Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap())
}

async fn agent() -> Agent {
    let url = std::env::var("IC_URL").unwrap_or_else(|_| "https://ic0.app".into());
    let agent = Agent::builder()
        .with_url(url)
        .build()
        .expect("create agent");
    let _ = agent.fetch_root_key().await; // ignore in prod
    agent
}

pub async fn fetch(principal: Principal) -> Vec<Holding> {
    let agent = agent().await;
    let canister_id = ledger_canister();

    let args = Encode!(&principal).expect("encode");
    let balance_raw = agent
        .query(&canister_id, "icrc1_balance_of")
        .with_arg(args)
        .call()
        .await
        .unwrap_or_default();
    let balance: u128 = Decode!(&balance_raw, u128).unwrap_or_default();

    let symbol_raw = agent
        .query(&canister_id, "icrc1_symbol")
        .with_arg(Vec::<u8>::new())
        .call()
        .await
        .unwrap_or_default();
    let token: String = Decode!(&symbol_raw, String).unwrap_or_else(|_| "ICP".into());

    vec![Holding {
        source: "ledger".to_string(),
        token,
        amount: balance.to_string(),
        status: "liquid".to_string(),
    }]
}
