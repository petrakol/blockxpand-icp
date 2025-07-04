use candid::Principal;
use bx_core::Holding;

pub async fn fetch(_principal: Principal) -> Vec<Holding> {
    vec![Holding {
        source: "neuron".to_string(),
        token: "ICP".to_string(),
        amount: "1200".to_string(),
        status: "locked_8y".to_string(),
    }]
}
