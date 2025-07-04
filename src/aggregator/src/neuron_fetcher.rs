use bx_core::Holding;
use candid::Principal;

pub async fn fetch(_principal: Principal) -> Vec<Holding> {
    tokio::time::sleep(std::time::Duration::from_millis(7)).await;
    vec![Holding {
        source: "neuron".to_string(),
        token: "ICP".to_string(),
        amount: "1200".to_string(),
        status: "locked_8y".to_string(),
    }]
}
