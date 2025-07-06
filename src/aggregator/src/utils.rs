use candid::Nat;

/// Nanoseconds in one day and one week
pub const DAY_NS: u64 = 86_400_000_000_000;
pub const WEEK_NS: u64 = DAY_NS * 7;
pub const DAY_SECS: u64 = 86_400;

#[cfg(not(target_arch = "wasm32"))]
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

#[cfg(target_arch = "wasm32")]
pub fn now() -> u64 {
    ic_cdk::api::time()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn format_amount(n: Nat, decimals: u8) -> String {
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
pub fn format_amount(n: Nat, _decimals: u8) -> String {
    n.0.to_string()
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn get_agent() -> ic_agent::Agent {
    let url = std::env::var("LEDGER_URL").unwrap_or_else(|_| "http://localhost:4943".into());
    let agent = ic_agent::Agent::builder().with_url(url).build().unwrap();
    let _ = agent.fetch_root_key().await;
    agent
}

#[cfg(not(target_arch = "wasm32"))]
pub fn env_principal(name: &str) -> Option<candid::Principal> {
    std::env::var(name)
        .ok()
        .and_then(|v| candid::Principal::from_text(v).ok())
}
