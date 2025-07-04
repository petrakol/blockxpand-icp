use bx_core::Holding;
use candid::Principal;
use tokio::time::{sleep, Duration};

pub async fn fetch(_principal: Principal) -> Vec<Holding> {
    sleep(Duration::from_millis(5)).await;
    vec![Holding {
        source: "neuron".into(),
        token: "ICP".into(),
        amount: "1200".into(),
        status: "locked_8y".into(),
    }]
}
