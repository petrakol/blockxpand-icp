use candid::Nat;
#[cfg(not(target_arch = "wasm32"))]
use once_cell::sync::{Lazy, OnceCell};
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::RwLock;

/// Common time constants in nanoseconds
pub const MINUTE_NS: u64 = 60_000_000_000;
pub const DAY_NS: u64 = 86_400_000_000_000;
pub const WEEK_NS: u64 = DAY_NS * 7;
pub const DAY_SECS: u64 = 86_400;
/// Seconds in one week
pub const WEEK_SECS: u64 = DAY_SECS * 7;

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
static AGENT: OnceCell<ic_agent::Agent> = OnceCell::new();

#[cfg(not(target_arch = "wasm32"))]
static PRINCIPAL_CACHE: Lazy<RwLock<HashMap<String, Option<candid::Principal>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

#[cfg(not(target_arch = "wasm32"))]
pub async fn get_agent() -> ic_agent::Agent {
    if let Some(a) = AGENT.get() {
        return a.clone();
    }
    let url = std::env::var("LEDGER_URL").unwrap_or_else(|_| "http://localhost:4943".into());
    let agent = ic_agent::Agent::builder()
        .with_url(url)
        .build()
        .expect("failed to build agent");
    if let Err(e) = agent.fetch_root_key().await {
        eprintln!("failed to fetch root key: {e}");
    }
    let _ = AGENT.set(agent.clone());
    agent
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(target_arch = "wasm32"))]
static DEX_DEFAULTS: once_cell::sync::Lazy<std::collections::HashMap<String, candid::Principal>> =
    once_cell::sync::Lazy::new(|| {
        let path =
            std::env::var("LEDGERS_FILE").unwrap_or_else(|_| "config/ledgers.toml".to_string());
        let text = std::fs::read_to_string(path).unwrap_or_default();
        let value: toml::Value =
            toml::from_str(&text).unwrap_or(toml::Value::Table(Default::default()));
        value
            .get("dex")
            .and_then(|d| d.as_table())
            .map(|table| {
                table
                    .iter()
                    .filter_map(|(k, v)| {
                        v.as_str()
                            .and_then(|id| candid::Principal::from_text(id).ok())
                            .map(|p| (k.clone(), p))
                    })
                    .collect()
            })
            .unwrap_or_default()
    });

#[cfg(not(target_arch = "wasm32"))]
pub fn env_principal(name: &str) -> Option<candid::Principal> {
    use tracing::warn;

    if let Some(p) = PRINCIPAL_CACHE.read().unwrap().get(name) {
        return *p;
    }
    let val = match std::env::var(name) {
        Ok(v) => match candid::Principal::from_text(&v) {
            Ok(p) => Some(p),
            Err(e) => {
                eprintln!("{name} is not a valid principal: {e}");
                None
            }
        },
        Err(_) => {
            if let Some(p) = DEX_DEFAULTS.get(name) {
                warn!("{name} missing; using fallback from ledgers.toml");
                Some(*p)
            } else {
                warn!("environment variable {name} not set");
                None
            }
        }
    };
    PRINCIPAL_CACHE
        .write()
        .unwrap()
        .insert(name.to_string(), val);
    val
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn dex_block_height(agent: &ic_agent::Agent, cid: candid::Principal) -> Option<u64> {
    use candid::{Decode, Encode};
    let arg = Encode!().unwrap();
    let bytes = agent
        .query(&cid, "block_height")
        .with_arg(arg)
        .call()
        .await
        .ok()?;
    Decode!(&bytes, u64).ok()
}
