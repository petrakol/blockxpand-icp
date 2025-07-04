use bx_core::Holding;
use candid::{CandidType, Decode, Encode, Principal};
use ic_agent::Agent;
use serde::Deserialize;

#[derive(CandidType)]
struct ListNeurons {
    of_principal: Option<Principal>,
}

#[derive(CandidType, Deserialize)]
struct NeuronInfo {
    dissolve_delay_seconds: u64,
    stake_e8s: u64,
}

fn governance_canister() -> Principal {
    std::env::var("GOVERNANCE_CANISTER_ID")
        .ok()
        .and_then(|s| Principal::from_text(&s).ok())
        .unwrap_or_else(|| Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap())
}

async fn agent() -> Agent {
    let url = std::env::var("IC_URL").unwrap_or_else(|_| "https://ic0.app".into());
    let agent = Agent::builder()
        .with_url(url)
        .build()
        .expect("create agent");
    let _ = agent.fetch_root_key().await;
    agent
}

pub async fn fetch(principal: Principal) -> Vec<Holding> {
    let agent = agent().await;
    let canister_id = governance_canister();

    let req = ListNeurons {
        of_principal: Some(principal),
    };
    let args = Encode!(&req).expect("encode");
    let res = agent
        .query(&canister_id, "list_neurons")
        .with_arg(args)
        .call()
        .await
        .unwrap_or_default();
    let neurons: Vec<NeuronInfo> = Decode!(&res, Vec<NeuronInfo>).unwrap_or_default();

    neurons
        .into_iter()
        .map(|n| Holding {
            source: "neuron".into(),
            token: "ICP".into(),
            amount: (n.stake_e8s as u128 / 100_000_000).to_string(),
            status: if n.dissolve_delay_seconds > 0 {
                "locked".into()
            } else {
                "dissolved".into()
            },
        })
        .collect()
}
