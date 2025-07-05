use bx_core::Holding;
use candid::{Decode, Encode, Nat, Principal};
use dashmap::DashMap;
use futures::future::join_all;
use num_traits::cast::ToPrimitive;
use once_cell::sync::Lazy;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::future::Future;

// Metadata for each ledger is cached with an expiry and a stable hash.
// When a hash mismatch is detected, the entry is replaced so callers
// always see the latest token symbol, decimals, and transfer fee.

#[cfg(not(target_arch = "wasm32"))]
use ic_agent::Agent;

#[cfg(not(target_arch = "wasm32"))]
fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}
#[cfg(target_arch = "wasm32")]
fn now() -> u64 {
    ic_cdk::api::time()
}

#[derive(Deserialize)]
struct LedgersConfig {
    ledgers: std::collections::HashMap<String, String>,
}

static LEDGERS: Lazy<Vec<Principal>> = Lazy::new(|| {
    let cfg: LedgersConfig =
        toml::from_str(include_str!("../../../config/ledgers.toml")).expect("invalid config");
    cfg.ledgers
        .values()
        .map(|id| Principal::from_text(id).expect("invalid principal"))
        .collect()
});

#[derive(Clone)]
struct Meta {
    symbol: String,
    decimals: u8,
    fee: u64,
    hash: [u8; 32],
    expires: u64,
}
static META_CACHE: Lazy<DashMap<Principal, Meta>> = Lazy::new(DashMap::new);

#[cfg(not(target_arch = "wasm32"))]
async fn with_retry<F, Fut, T>(mut f: F) -> Result<T, ic_agent::AgentError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, ic_agent::AgentError>>,
{
    let mut delay = 100u64;
    for attempt in 0..3 {
        match f().await {
            Ok(v) => return Ok(v),
            Err(_e) if attempt < 2 => {
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                delay *= 2;
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}

#[cfg(not(target_arch = "wasm32"))]
async fn get_agent() -> Agent {
    let url = std::env::var("LEDGER_URL").unwrap_or_else(|_| "http://localhost:4943".to_string());
    let agent = Agent::builder().with_url(url).build().unwrap();
    let _ = agent.fetch_root_key().await;
    agent
}

#[cfg(not(target_arch = "wasm32"))]
async fn icrc1_metadata(agent: &Agent, canister_id: Principal) -> Result<Vec<(String, candid::types::value::IDLValue)>, ic_agent::AgentError> {
    let arg = candid::Encode!().unwrap();
    let bytes = agent.query(&canister_id, "icrc1_metadata").with_arg(arg).call().await?;
    let res: Vec<(String, candid::types::value::IDLValue)> = candid::Decode!(&bytes, Vec<(String, candid::types::value::IDLValue)>).unwrap();
    Ok(res)
}

#[cfg(not(target_arch = "wasm32"))]
async fn icrc1_balance_of(agent: &Agent, canister_id: Principal, owner: Principal) -> Result<Nat, ic_agent::AgentError> {
    #[derive(candid::CandidType)]
    struct Account { owner: Principal, subaccount: Option<Vec<u8>> }
    let arg = candid::Encode!(&Account { owner, subaccount: None }).unwrap();
    let bytes = agent
        .query(&canister_id, "icrc1_balance_of")
        .with_arg(arg)
        .call()
        .await?;
    let res: Nat = candid::Decode!(&bytes, Nat).unwrap();
    Ok(res)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch(principal: Principal) -> Vec<Holding> {
    let agent = get_agent().await;
    let futures = LEDGERS.iter().cloned().map(|cid| {
        let agent = agent.clone();
        async move {
            let (symbol, decimals, _) = match fetch_metadata(&agent, cid).await {
                Ok(v) => v,
                Err(_) => {
                    return Holding {
                        source: "ledger".into(),
                        token: "unknown".into(),
                        amount: "0".into(),
                        status: "error".into(),
                    }
                }
            };
            match with_retry(|| icrc1_balance_of(&agent, cid, principal)).await {
                Ok(nat) => Holding {
                    source: "ledger".into(),
                    token: symbol,
                    amount: format_amount(nat, decimals),
                    status: "liquid".into(),
                },
                Err(_) => Holding {
                    source: "ledger".into(),
                    token: symbol,
                    amount: "0".into(),
                    status: "error".into(),
                },
            }
        }
    });
    join_all(futures).await
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch(_principal: Principal) -> Vec<Holding> {
    Vec::new()
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_metadata(
    agent: &Agent,
    cid: Principal,
) -> Result<(String, u8, u64), ic_agent::AgentError> {
    if let Some(meta) = META_CACHE.get(&cid) {
        if meta.expires > now() { return Ok((meta.symbol.clone(), meta.decimals, meta.fee)); }
    }
    let items = with_retry(|| icrc1_metadata(agent, cid)).await?;
    let encoded = Encode!(&items).unwrap();
    let hash: [u8; 32] = Sha256::digest(&encoded).into();
    if let Some(meta) = META_CACHE.get(&cid) {
        if meta.hash == hash {
            META_CACHE.insert(
                cid,
                Meta {
                    hash,
                    expires: now() + 86_400_000_000_000,
                    ..meta.clone()
                },
            );
            return Ok((meta.symbol.clone(), meta.decimals, meta.fee));
        }
    }
    let mut symbol = String::new();
    let mut decimals: u8 = 0;
    let mut fee: u64 = 0;
    for (k, v) in items {
        use candid::types::value::IDLValue::*;
        match (k.as_str(), v) {
            ("icrc1:symbol", Text(s)) => symbol = s,
            ("icrc1:decimals", Nat(n)) => decimals = n.0.to_u64().unwrap_or(0) as u8,
            ("icrc1:decimals", Nat32(n)) => decimals = n as u8,
            ("icrc1:decimals", Nat64(n)) => decimals = n as u8,
            ("icrc1:decimals", Nat16(n)) => decimals = n as u8,
            ("icrc1:decimals", Nat8(n)) => decimals = n,
            ("icrc1:fee", Nat(n)) => fee = n.0.to_u64().unwrap_or(0),
            ("icrc1:fee", Nat32(n)) => fee = n as u64,
            ("icrc1:fee", Nat64(n)) => fee = n,
            ("icrc1:fee", Nat16(n)) => fee = n as u64,
            ("icrc1:fee", Nat8(n)) => fee = n as u64,
            _ => {}
        }
    }
    META_CACHE.insert(
        cid,
        Meta {
            symbol: symbol.clone(),
            decimals,
            fee,
            hash,
            expires: now() + 86_400_000_000_000,
        },
    );
    Ok((symbol, decimals, fee))
}

#[cfg(not(target_arch = "wasm32"))]
fn format_amount(nat: Nat, decimals: u8) -> String {
    use num_bigint::BigUint;
    use num_integer::Integer;
    let div = BigUint::from(10u32).pow(decimals as u32);
    let (q, r) = nat.0.div_rem(&div);
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
