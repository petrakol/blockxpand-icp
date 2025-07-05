use super::{DexAdapter, RewardInfo};
use async_trait::async_trait;
use bx_core::Holding;
use candid::{CandidType, Decode, Encode, Nat, Principal};
use serde::Deserialize;

#[derive(CandidType, Deserialize, Clone)]
struct Token {
    address: String,
    decimals: u8,
}

#[derive(CandidType, Deserialize, Clone)]
struct PositionInfo {
    token_a: Token,
    token_b: Token,
    #[serde(rename = "token_a_amount")]
    token_a_amount: Nat,
    #[serde(rename = "token_b_amount")]
    token_b_amount: Nat,
    reward_token: Token,
    reward_amount: Nat,
    auto_compound: bool,
}

pub struct SonicAdapter;

#[cfg(not(target_arch = "wasm32"))]
async fn get_agent() -> ic_agent::Agent {
    let url = std::env::var("LEDGER_URL").unwrap_or_else(|_| "http://localhost:4943".into());
    let agent = ic_agent::Agent::builder().with_url(url).build().unwrap();
    let _ = agent.fetch_root_key().await;
    agent
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_positions_impl(principal: Principal) -> Vec<Holding> {
    let router_id = match std::env::var("SONIC_ROUTER") {
        Ok(v) => match Principal::from_text(v) {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        },
        Err(_) => return Vec::new(),
    };
    let agent = get_agent().await;
    let arg = Encode!(&principal).unwrap();
    let bytes = match agent
        .query(&router_id, "get_user_positions")
        .with_arg(arg)
        .call()
        .await
    {
        Ok(b) => b,
        Err(_) => return Vec::new(),
    };
    let positions: Vec<PositionInfo> = Decode!(&bytes, Vec<PositionInfo>).unwrap_or_default();
    let mut out = Vec::new();
    for pos in positions {
        let a0 = format_amount(pos.token_a_amount, pos.token_a.decimals);
        out.push(Holding {
            source: "Sonic".into(),
            token: pos.token_a.address.clone(),
            amount: a0,
            status: "lp_escrow".into(),
        });
        let a1 = format_amount(pos.token_b_amount, pos.token_b.decimals);
        out.push(Holding {
            source: "Sonic".into(),
            token: pos.token_b.address.clone(),
            amount: a1,
            status: "lp_escrow".into(),
        });
        if !pos.auto_compound {
            let ra = format_amount(pos.reward_amount, pos.reward_token.decimals);
            out.push(Holding {
                source: "Sonic".into(),
                token: pos.reward_token.address.clone(),
                amount: ra,
                status: "lp_escrow".into(),
            });
        }
    }
    out
}

#[cfg(target_arch = "wasm32")]
async fn fetch_positions_impl(_principal: Principal) -> Vec<Holding> {
    Vec::new()
}

#[cfg(not(target_arch = "wasm32"))]
fn format_amount(n: Nat, decimals: u8) -> String {
    use num_bigint::BigUint;
    use num_integer::Integer;
    let div = BigUint::from(10u32).pow(decimals as u32);
    let (q, r) = n.0.div_rem(&div);
    let mut frac = r.to_str_radix(10);
    while frac.len() < decimals as usize {
        frac.insert(0, '0');
    }
    if decimals == 0 {
        q.to_str_radix(10)
    } else {
        format!("{}.{frac}", q.to_str_radix(10))
    }
}

#[cfg(target_arch = "wasm32")]
fn format_amount(n: Nat, _decimals: u8) -> String {
    n.0.to_string()
}

#[async_trait]
impl DexAdapter for SonicAdapter {
    async fn fetch_positions(&self, principal: Principal) -> Vec<Holding> {
        fetch_positions_impl(principal).await
    }

    async fn claimable_rewards(&self, _principal: Principal) -> Vec<RewardInfo> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;

    #[tokio::test]
    async fn empty_without_env() {
        std::env::remove_var("SONIC_ROUTER");
        let adapter = SonicAdapter;
        let res = adapter.fetch_positions(Principal::anonymous()).await;
        assert!(res.is_empty());
    }
}
