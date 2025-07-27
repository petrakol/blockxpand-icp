pub mod cache;
pub mod cert;
pub mod cycles;
pub mod dex;
pub mod dex_fetchers;
pub mod error;
pub mod ledger_fetcher;
pub mod logging;
pub mod lp_cache;
pub mod metrics;
pub mod neuron_fetcher;
pub mod pool_registry;
pub mod user_settings;
pub mod utils;
pub mod warm;

use crate::utils::{now, MINUTE_NS};
use bx_core::Holding;
use candid::Principal;
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
#[cfg(feature = "claim")]
use std::collections::{HashMap, HashSet};
#[cfg(feature = "claim")]
use std::sync::Mutex;

static MAX_HOLDINGS: Lazy<usize> = Lazy::new(|| {
    option_env!("MAX_HOLDINGS")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(500)
});
lazy_static! {
    pub static ref CALL_PRICE: u128 = std::env::var("CALL_PRICE_CYCLES")
        .ok()
        .and_then(|v| v.parse::<u128>().ok())
        .unwrap_or(0);
    pub static ref CLAIM_PRICE: u128 = std::env::var("CLAIM_PRICE_CYCLES")
        .ok()
        .and_then(|v| v.parse::<u128>().ok())
        .unwrap_or(0);
}
#[cfg(feature = "claim")]
static CLAIM_WALLETS: Lazy<HashSet<Principal>> = Lazy::new(|| {
    option_env!("CLAIM_WALLETS")
        .unwrap_or("")
        .split(',')
        .filter_map(|s| Principal::from_text(s.trim()).ok())
        .collect::<HashSet<_>>()
});
#[cfg(feature = "claim")]
static CLAIM_DENYLIST: Lazy<HashSet<Principal>> = Lazy::new(|| {
    option_env!("CLAIM_DENYLIST")
        .unwrap_or("")
        .split(',')
        .filter_map(|s| Principal::from_text(s.trim()).ok())
        .collect::<HashSet<_>>()
});
#[cfg(feature = "claim")]
static CLAIM_LOCKS: Lazy<Mutex<HashMap<Principal, u64>>> = Lazy::new(|| Mutex::new(HashMap::new()));
#[cfg(feature = "claim")]
static CLAIM_LOCK_TIMEOUT_NS: Lazy<u64> = Lazy::new(|| {
    option_env!("CLAIM_LOCK_TIMEOUT_SECS")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(300)
        * 1_000_000_000u64
});

#[cfg(feature = "claim")]
static CLAIM_LIMIT_WINDOW_NS: Lazy<u64> = Lazy::new(|| {
    option_env!("CLAIM_LIMIT_WINDOW_SECS")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(crate::utils::DAY_SECS)
        * 1_000_000_000u64
});

#[cfg(feature = "claim")]
static CLAIM_DAILY_LIMIT: Lazy<u32> = Lazy::new(|| {
    option_env!("CLAIM_DAILY_LIMIT")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(5)
});

#[cfg(feature = "claim")]
static CLAIM_MAX_TOTAL: Lazy<u64> = Lazy::new(|| {
    option_env!("CLAIM_MAX_TOTAL")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(u64::MAX)
});

#[cfg(feature = "claim")]
static MAX_CLAIM_PER_CALL: Lazy<usize> = Lazy::new(|| {
    option_env!("MAX_CLAIM_PER_CALL")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(usize::MAX)
});

#[cfg(feature = "claim")]
static CLAIM_COUNTS: Lazy<Mutex<HashMap<Principal, (u32, u64)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[cfg(feature = "claim")]
static CLAIM_COOLDOWN_NS: Lazy<u64> = Lazy::new(|| {
    option_env!("CLAIM_COOLDOWN_SECS")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(60)
        * 1_000_000_000u64
});

#[cfg(feature = "claim")]
static CLAIM_COOLDOWN: Lazy<Mutex<HashMap<Principal, u64>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[cfg(feature = "claim")]
static CLAIM_ADAPTER_TIMEOUT_SECS: Lazy<std::sync::atomic::AtomicU64> = Lazy::new(|| {
    use std::sync::atomic::AtomicU64;
    AtomicU64::new(
        option_env!("CLAIM_ADAPTER_TIMEOUT_SECS")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(10),
    )
});

async fn calculate_holdings(
    principal: Principal,
) -> Result<(Vec<Holding>, Vec<HoldingSummary>), rust_decimal::Error> {
    let settings = user_settings::get(&principal).unwrap_or_default();
    use std::collections::HashSet;
    let ledger_set: HashSet<Principal> = settings
        .preferred_ledgers
        .iter()
        .filter_map(|s| Principal::from_text(s).ok())
        .collect();
    let dex_set: HashSet<String> = settings.preferred_dexes.iter().cloned().collect();
    let ledger_filter = if ledger_set.is_empty() {
        None
    } else {
        Some(&ledger_set)
    };
    let dex_filter = if dex_set.is_empty() { None } else { Some(&dex_set) };
    let (ledger, neuron, dex) = futures::join!(
        ledger_fetcher::fetch_filtered(principal, ledger_filter),
        neuron_fetcher::fetch(principal),
        dex_fetchers::fetch_filtered(principal, dex_filter)
    );

    let capacity =
        ledger.as_ref().map_or(0, |v| v.len()) + neuron.len() + dex.as_ref().map_or(0, |v| v.len());
    let mut holdings = Vec::with_capacity(capacity);
    holdings.extend(ledger.unwrap_or_default());
    holdings.extend(neuron);
    holdings.extend(dex.unwrap_or_default());
    if holdings.len() > *MAX_HOLDINGS {
        holdings.truncate(*MAX_HOLDINGS);
    }
    let summary = summarise(&holdings)?;
    Ok((holdings, summary))
}

#[cfg(target_arch = "wasm32")]
fn instructions() -> u64 {
    ic_cdk::api::instruction_counter()
}

#[cfg(not(target_arch = "wasm32"))]
fn instructions() -> u64 {
    0
}

#[cfg(target_arch = "wasm32")]
pub fn pay_cycles(price: u128) {
    if price > 0 {
        let accepted = ic_cdk::api::call::msg_cycles_accept128(price);
        metrics::add_cycles_collected(accepted);
        if accepted < price {
            ic_cdk::api::trap("insufficient cycles");
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn pay_cycles(_price: u128) {}

#[cfg(target_arch = "wasm32")]
fn accept_cycles(price: u128) -> u128 {
    let accepted = ic_cdk::api::call::msg_cycles_accept128(price);
    metrics::add_cycles_collected(accepted);
    accepted
}

#[cfg(not(target_arch = "wasm32"))]
fn accept_cycles(price: u128) -> u128 {
    price
}

#[ic_cdk_macros::update]
pub async fn get_holdings(principal: Principal) -> Result<Vec<Holding>, String> {
    metrics::inc_query();
    let accepted = accept_cycles(*CALL_PRICE);
    if accepted < *CALL_PRICE {
        return Err(format!(
            "Insufficient cycles: sent {}, required {}",
            accepted, *CALL_PRICE
        ));
    }
    cycles::ensure_margin();
    let start_cycles = cycles::available();
    let start = instructions();
    let now = now();
    {
        let cache = cache::get();
        if let Some(v) = cache.get(&principal) {
            let (cached, _, ts) = v.value().clone();
            if now - ts < MINUTE_NS {
                let used = instructions().saturating_sub(start);
                tracing::info!(
                    "get_holdings took {used} instructions ({:.2} B)",
                    used as f64 / 1_000_000_000f64
                );
                return Ok(cached);
            }
        }
    }

    let (holdings, summary) = calculate_holdings(principal)
        .await
        .map_err(|e| e.to_string())?;

    {
        cache::get().insert(principal, (holdings.clone(), summary, now));
    }
    let used = instructions().saturating_sub(start);
    tracing::info!(
        "get_holdings took {used} instructions ({:.2} B)",
        used as f64 / 1_000_000_000f64
    );
    let used_cycles = start_cycles.saturating_sub(cycles::available());
    metrics::record_query_cycles(used_cycles as u64);
    Ok(holdings)
}

#[ic_cdk_macros::update]
pub async fn get_holdings_filtered(
    principal: Principal,
    ledgers: Vec<String>,
    dexes: Vec<String>,
) -> Result<Vec<Holding>, String> {
    metrics::inc_query();
    let accepted = accept_cycles(*CALL_PRICE);
    if accepted < *CALL_PRICE {
        return Err(format!(
            "Insufficient cycles: sent {}, required {}",
            accepted, *CALL_PRICE
        ));
    }
    cycles::ensure_margin();
    let start_cycles = cycles::available();
    let start = instructions();
    use std::collections::HashSet;
    let ledger_set: HashSet<Principal> = ledgers
        .iter()
        .filter_map(|s| Principal::from_text(s).ok())
        .collect();
    let dex_set: HashSet<String> = dexes.into_iter().collect();
    let ledger_filter = if ledger_set.is_empty() { None } else { Some(&ledger_set) };
    let dex_filter = if dex_set.is_empty() { None } else { Some(&dex_set) };
    let (ledger, neuron, dex) = futures::join!(
        ledger_fetcher::fetch_filtered(principal, ledger_filter),
        neuron_fetcher::fetch(principal),
        dex_fetchers::fetch_filtered(principal, dex_filter)
    );
    let capacity =
        ledger.as_ref().map_or(0, |v| v.len()) + neuron.len() + dex.as_ref().map_or(0, |v| v.len());
    let mut holdings = Vec::with_capacity(capacity);
    holdings.extend(ledger.unwrap_or_default());
    holdings.extend(neuron);
    holdings.extend(dex.unwrap_or_default());
    if holdings.len() > *MAX_HOLDINGS {
        holdings.truncate(*MAX_HOLDINGS);
    }
    let used = instructions().saturating_sub(start);
    tracing::info!(
        "get_holdings_filtered took {used} instructions ({:.2} B)",
        used as f64 / 1_000_000_000f64
    );
    let used_cycles = start_cycles.saturating_sub(cycles::available());
    metrics::record_query_cycles(used_cycles as u64);
    Ok(holdings)
}

#[cfg(feature = "claim")]
#[ic_cdk_macros::update]
pub async fn claim_all_rewards(principal: Principal) -> Vec<u64> {
    metrics::inc_query();
    let accepted = accept_cycles(*CLAIM_PRICE);
    if accepted < *CLAIM_PRICE {
        ic_cdk::api::trap(&format!(
            "Insufficient cycles: sent {}, required {}",
            accepted, *CLAIM_PRICE
        ));
    }
    cycles::ensure_margin();
    let start_cycles = cycles::available();
    metrics::inc_claim_attempt();
    let caller = ic_cdk::caller();
    if caller != principal && !CLAIM_WALLETS.contains(&caller) {
        ic_cdk::api::trap("unauthorized");
    }
    if principal == Principal::anonymous() {
        ic_cdk::api::trap("invalid principal");
    }
    if CLAIM_DENYLIST.contains(&principal) {
        ic_cdk::api::trap("denied");
    }
    {
        let mut map = CLAIM_COOLDOWN.lock().unwrap();
        let now = now();
        map.retain(|_, exp| *exp > now);
        if let Some(exp) = map.get(&principal) {
            if *exp > now {
                ic_cdk::api::trap("cooldown");
            }
        }
        map.insert(principal, now + *CLAIM_COOLDOWN_NS);
    }
    {
        let mut counts = CLAIM_COUNTS.lock().unwrap();
        let now = now();
        let entry = counts
            .entry(principal)
            .or_insert((0, now + *CLAIM_LIMIT_WINDOW_NS));
        if now > entry.1 {
            *entry = (0, now + *CLAIM_LIMIT_WINDOW_NS);
        }
        if entry.0 >= *CLAIM_DAILY_LIMIT {
            ic_cdk::api::trap("claim limit reached");
        }
        entry.0 += 1;
    }
    {
        let mut locks = CLAIM_LOCKS.lock().unwrap();
        let now = now();
        locks.retain(|_, exp| *exp > now);
        if locks.contains_key(&principal) {
            ic_cdk::api::trap("claim already in progress");
        }
        locks.insert(principal, now + *CLAIM_LOCK_TIMEOUT_NS);
    }
    struct Guard(Principal);
    impl Drop for Guard {
        fn drop(&mut self) {
            CLAIM_LOCKS.lock().unwrap().remove(&self.0);
        }
    }
    let _guard = Guard(principal);
    use dex::registry;
    let mut adapters: Vec<registry::AdapterEntry> = registry::get();
    if *MAX_CLAIM_PER_CALL < adapters.len() {
        adapters.truncate(*MAX_CLAIM_PER_CALL);
    }
    let mut spent = Vec::with_capacity(adapters.len());
    let mut total: u64 = 0;
    for entry in adapters {
        if total >= *CLAIM_MAX_TOTAL {
            break;
        }
        if let Some(c) = claim_with_timeout(entry.adapter.claim_rewards(principal)).await {
            total = total.saturating_add(c);
            if total > *CLAIM_MAX_TOTAL {
                ic_cdk::api::trap("claim total exceeded");
            }
            spent.push(c);
        }
    }
    metrics::inc_claim_success();
    let used_cycles = start_cycles.saturating_sub(cycles::available());
    metrics::record_query_cycles(used_cycles as u64);
    spent
}

#[cfg(feature = "claim")]
pub(crate) async fn claim_with_timeout<F>(fut: F) -> Option<u64>
where
    F: std::future::Future<Output = Result<u64, String>>,
{
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::sync::atomic::Ordering;
        use tokio::time::{timeout, Duration};
        let secs = CLAIM_ADAPTER_TIMEOUT_SECS.load(Ordering::Relaxed);
        match timeout(Duration::from_secs(secs), fut).await {
            Ok(Ok(v)) => Some(v),
            Ok(Err(e)) => {
                tracing::error!("claim failed: {e}");
                None
            }
            Err(_) => {
                tracing::error!("claim timed out");
                None
            }
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        match fut.await {
            Ok(v) => Some(v),
            Err(e) => {
                tracing::error!("claim failed: {e}");
                None
            }
        }
    }
}

#[ic_cdk_macros::query]
pub fn pools_graphql(query: String) -> String {
    metrics::inc_query();
    pay_cycles(*CALL_PRICE);
    cycles::ensure_margin();
    let start_cycles = cycles::available();
    let res = pool_registry::graphql(query);
    let used_cycles = start_cycles.saturating_sub(cycles::available());
    metrics::record_query_cycles(used_cycles as u64);
    res
}

#[derive(candid::CandidType, serde::Serialize)]
pub struct CertifiedHoldings {
    pub holdings: Vec<Holding>,
    #[serde(with = "serde_bytes")]
    pub certificate: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub witness: Vec<u8>,
}

#[ic_cdk_macros::update]
pub async fn refresh_holdings(principal: Principal) -> Result<(), String> {
    metrics::inc_query();
    pay_cycles(*CALL_PRICE);
    cycles::ensure_margin();
    let start_cycles = cycles::available();
    let now = now();
    let (holdings, summary) = calculate_holdings(principal)
        .await
        .map_err(|e| e.to_string())?;
    cache::get().insert(principal, (holdings.clone(), summary, now));
    cert::update(principal, &holdings);
    let used_cycles = start_cycles.saturating_sub(cycles::available());
    metrics::record_query_cycles(used_cycles as u64);
    Ok(())
}

#[ic_cdk_macros::query]
pub fn get_holdings_cert(principal: Principal) -> CertifiedHoldings {
    metrics::inc_query();
    pay_cycles(*CALL_PRICE);
    cycles::ensure_margin();
    let start_cycles = cycles::available();
    let holdings = cache::get()
        .get(&principal)
        .map(|v| v.value().0.clone())
        .unwrap_or_default();
    let certificate = ic_cdk::api::data_certificate().unwrap_or_default();
    let witness = cert::witness(principal);
    let out = CertifiedHoldings {
        holdings,
        certificate,
        witness,
    };
    let used_cycles = start_cycles.saturating_sub(cycles::available());
    metrics::record_query_cycles(used_cycles as u64);
    out
}

#[derive(Clone, candid::CandidType, serde::Serialize, serde::Deserialize)]
pub struct HoldingSummary {
    pub token: String,
    pub total: f64,
}

#[ic_cdk_macros::update]
pub async fn get_holdings_summary(principal: Principal) -> Result<Vec<HoldingSummary>, String> {
    metrics::inc_query();
    let accepted = accept_cycles(*CALL_PRICE);
    if accepted < *CALL_PRICE {
        return Err(format!(
            "Insufficient cycles: sent {}, required {}",
            accepted, *CALL_PRICE
        ));
    }
    cycles::ensure_margin();
    let start_cycles = cycles::available();
    let now = now();
    {
        if let Some(v) = cache::get().get(&principal) {
            let (_, summary, ts) = v.value().clone();
            if now - ts < MINUTE_NS {
                return Ok(summary);
            }
        }
    }
    let (holdings, summary) = calculate_holdings(principal)
        .await
        .map_err(|e| e.to_string())?;
    cache::get().insert(principal, (holdings, summary.clone(), now));
    let used_cycles = start_cycles.saturating_sub(cycles::available());
    metrics::record_query_cycles(used_cycles as u64);
    Ok(summary)
}

fn summarise(holdings: &[Holding]) -> Result<Vec<HoldingSummary>, rust_decimal::Error> {
    use rust_decimal::prelude::{FromStr, ToPrimitive};
    use std::collections::BTreeMap;
    let mut map: BTreeMap<String, rust_decimal::Decimal> = BTreeMap::new();
    for h in holdings {
        let v = rust_decimal::Decimal::from_str(&h.amount)?;
        *map.entry(h.token.clone())
            .or_insert(rust_decimal::Decimal::ZERO) += v;
    }
    Ok(map
        .into_iter()
        .map(|(token, total)| HoldingSummary {
            token,
            total: total.to_f64().unwrap_or(0.0),
        })
        .collect())
}

#[derive(candid::CandidType, serde::Serialize)]
pub struct Version {
    pub git_sha: &'static str,
    pub build_time: &'static str,
}

#[ic_cdk_macros::query]
pub fn get_version() -> Version {
    metrics::inc_query();
    pay_cycles(*CALL_PRICE);
    cycles::ensure_margin();
    let start_cycles = cycles::available();
    let out = Version {
        git_sha: option_env!("GIT_SHA").unwrap_or("unknown"),
        build_time: option_env!("BUILD_TIME").unwrap_or("unknown"),
    };
    let used_cycles = start_cycles.saturating_sub(cycles::available());
    metrics::record_query_cycles(used_cycles as u64);
    out
}

#[ic_cdk_macros::query]
pub fn get_cycles_log() -> Vec<String> {
    metrics::inc_query();
    pay_cycles(*CALL_PRICE);
    cycles::ensure_margin();
    let start_cycles = cycles::available();
    let log = cycles::log();
    let used_cycles = start_cycles.saturating_sub(cycles::available());
    metrics::record_query_cycles(used_cycles as u64);
    log
}

#[ic_cdk_macros::query]
pub fn get_user_settings(principal: Principal) -> user_settings::UserSettings {
    metrics::inc_query();
    pay_cycles(*CALL_PRICE);
    cycles::ensure_margin();
    let start_cycles = cycles::available();
    let out = user_settings::get(&principal).unwrap_or_default();
    let used_cycles = start_cycles.saturating_sub(cycles::available());
    metrics::record_query_cycles(used_cycles as u64);
    out
}

#[ic_cdk_macros::update]
pub fn update_user_settings(principal: Principal, settings: user_settings::UserSettings) {
    metrics::inc_query();
    pay_cycles(*CALL_PRICE);
    cycles::ensure_margin();
    let start_cycles = cycles::available();
    let caller = ic_cdk::caller();
    if caller != principal {
        ic_cdk::api::trap("unauthorized");
    }
    user_settings::update(principal, settings);
    cache::get().remove(&principal);
    let used_cycles = start_cycles.saturating_sub(cycles::available());
    metrics::record_query_cycles(used_cycles as u64);
}

#[cfg(feature = "claim")]
#[derive(candid::CandidType, serde::Serialize)]
pub struct ClaimStatus {
    pub attempts: u32,
    pub window_expires: u64,
    pub locked: bool,
}

#[cfg(feature = "claim")]
#[ic_cdk_macros::query]
pub fn get_claim_status(principal: Principal) -> ClaimStatus {
    metrics::inc_query();
    pay_cycles(*CALL_PRICE);
    cycles::ensure_margin();
    let start_cycles = cycles::available();
    let now = now();
    let (attempts, window_expires) = CLAIM_COUNTS
        .lock()
        .unwrap()
        .get(&principal)
        .cloned()
        .unwrap_or((0, now + *CLAIM_LIMIT_WINDOW_NS));
    let locked = CLAIM_LOCKS
        .lock()
        .unwrap()
        .get(&principal)
        .is_some_and(|exp| *exp > now);
    let out = ClaimStatus {
        attempts,
        window_expires,
        locked,
    };
    let used_cycles = start_cycles.saturating_sub(cycles::available());
    metrics::record_query_cycles(used_cycles as u64);
    out
}

#[ic_cdk_macros::query]
pub fn health_check() -> &'static str {
    metrics::inc_query();
    pay_cycles(*CALL_PRICE);
    cycles::ensure_margin();
    let start_cycles = cycles::available();
    let out = "ok";
    let used_cycles = start_cycles.saturating_sub(cycles::available());
    metrics::record_query_cycles(used_cycles as u64);
    out
}

#[derive(candid::CandidType, serde::Serialize, serde::Deserialize)]
pub struct TokenTotal {
    pub token: String,
    pub total: f64,
}

fn summarize(holdings: &[Holding]) -> Result<Vec<TokenTotal>, rust_decimal::Error> {
    use rust_decimal::prelude::{FromStr, ToPrimitive};
    use std::collections::HashMap;
    let mut map: HashMap<String, rust_decimal::Decimal> = HashMap::new();
    for h in holdings {
        let v = rust_decimal::Decimal::from_str(&h.amount)?;
        *map.entry(h.token.clone())
            .or_insert(rust_decimal::Decimal::ZERO) += v;
    }
    let mut out: Vec<TokenTotal> = map
        .into_iter()
        .map(|(token, total)| TokenTotal {
            token,
            total: total.to_f64().unwrap_or(0.0),
        })
        .collect();
    out.sort_by(|a, b| a.token.cmp(&b.token));
    Ok(out)
}

#[ic_cdk_macros::query]
pub async fn get_summary(principal: Principal) -> Result<Vec<TokenTotal>, String> {
    metrics::inc_query();
    pay_cycles(*CALL_PRICE);
    cycles::ensure_margin();
    let start_cycles = cycles::available();
    let holdings = get_holdings(principal).await?;
    let res = summarize(&holdings).map_err(|e| e.to_string());
    let used_cycles = start_cycles.saturating_sub(cycles::available());
    metrics::record_query_cycles(used_cycles as u64);
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "current_thread")]
    async fn claim_with_timeout_times_out() {
        use std::sync::atomic::Ordering;
        CLAIM_ADAPTER_TIMEOUT_SECS.store(1, Ordering::Relaxed);
        let fut = async {
            tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
            Ok(1u64)
        };
        let res = claim_with_timeout(fut).await;
        assert!(res.is_none());
    }

    #[test]
    fn pay_cycles_noop_host() {
        let before = metrics::get().cycles.collected;
        pay_cycles(10);
        let after = metrics::get().cycles.collected;
        assert_eq!(before, after);
    }
}
