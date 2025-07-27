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
pub static CALL_PRICE_CYCLES: Lazy<u128> = Lazy::new(|| {
    option_env!("CALL_PRICE_CYCLES")
        .and_then(|v| v.parse::<u128>().ok())
        .unwrap_or(0)
});
pub static CLAIM_PRICE_CYCLES: Lazy<u128> = Lazy::new(|| {
    option_env!("CLAIM_PRICE_CYCLES")
        .and_then(|v| v.parse::<u128>().ok())
        .unwrap_or(0)
});
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
static CLAIM_ADAPTER_TIMEOUT_SECS: Lazy<u64> = Lazy::new(|| {
    option_env!("CLAIM_ADAPTER_TIMEOUT_SECS")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(10)
});

async fn calculate_holdings(principal: Principal) -> (Vec<Holding>, Vec<HoldingSummary>) {
    let settings = user_settings::get(&principal);
    let ledger_filter = settings.as_ref().and_then(|s| s.ledgers.as_ref());
    let dex_filter = settings.as_ref().and_then(|s| s.dexes.as_ref());
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
    let summary = summarise(&holdings);
    (holdings, summary)
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
    use ic_cdk::api::call::{msg_cycles_accept, msg_cycles_available};
    if price > 0 {
        let price_u64: u64 = price.try_into().unwrap_or(u64::MAX);
        if msg_cycles_available() < price_u64 {
            ic_cdk::api::trap("insufficient cycles");
        }
        let accepted = msg_cycles_accept(price_u64);
        metrics::add_cycles_collected(accepted as u128);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn pay_cycles(_price: u128) {}

#[ic_cdk_macros::query]
pub async fn get_holdings(principal: Principal) -> Vec<Holding> {
    metrics::inc_query();
    pay_cycles(*CALL_PRICE_CYCLES);
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
                return cached;
            }
        }
    }

    let (holdings, summary) = calculate_holdings(principal).await;

    {
        cache::get().insert(principal, (holdings.clone(), summary, now));
    }
    let used = instructions().saturating_sub(start);
    tracing::info!(
        "get_holdings took {used} instructions ({:.2} B)",
        used as f64 / 1_000_000_000f64
    );
    holdings
}

#[cfg(feature = "claim")]
#[ic_cdk_macros::update]
pub async fn claim_all_rewards(principal: Principal) -> Vec<u64> {
    metrics::inc_query();
    pay_cycles(*CLAIM_PRICE_CYCLES);
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
    spent
}

#[cfg(feature = "claim")]
async fn claim_with_timeout<F>(fut: F) -> Option<u64>
where
    F: std::future::Future<Output = Result<u64, String>>,
{
    #[cfg(not(target_arch = "wasm32"))]
    {
        use tokio::time::{timeout, Duration};
        match timeout(Duration::from_secs(*CLAIM_ADAPTER_TIMEOUT_SECS), fut).await {
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
    pay_cycles(*CALL_PRICE_CYCLES);
    pool_registry::graphql(query)
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
pub async fn refresh_holdings(principal: Principal) {
    metrics::inc_query();
    pay_cycles(*CALL_PRICE_CYCLES);
    let now = now();
    let (holdings, summary) = calculate_holdings(principal).await;
    cache::get().insert(principal, (holdings.clone(), summary, now));
    cert::update(principal, &holdings);
}

#[ic_cdk_macros::query]
pub fn get_holdings_cert(principal: Principal) -> CertifiedHoldings {
    metrics::inc_query();
    pay_cycles(*CALL_PRICE_CYCLES);
    let holdings = cache::get()
        .get(&principal)
        .map(|v| v.value().0.clone())
        .unwrap_or_default();
    let certificate = ic_cdk::api::data_certificate().unwrap_or_default();
    let witness = cert::witness(principal);
    CertifiedHoldings {
        holdings,
        certificate,
        witness,
    }
}

#[derive(Clone, candid::CandidType, serde::Serialize, serde::Deserialize)]
pub struct HoldingSummary {
    pub token: String,
    pub total: f64,
}

#[ic_cdk_macros::query]
pub async fn get_holdings_summary(principal: Principal) -> Vec<HoldingSummary> {
    metrics::inc_query();
    pay_cycles(*CALL_PRICE_CYCLES);
    let now = now();
    {
        if let Some(v) = cache::get().get(&principal) {
            let (_, summary, ts) = v.value().clone();
            if now - ts < MINUTE_NS {
                return summary;
            }
        }
    }
    let (holdings, summary) = calculate_holdings(principal).await;
    cache::get().insert(principal, (holdings, summary.clone(), now));
    summary
}

fn summarise(holdings: &[Holding]) -> Vec<HoldingSummary> {
    use std::collections::BTreeMap;
    let mut map: BTreeMap<String, f64> = BTreeMap::new();
    for h in holdings {
        if let Ok(v) = h.amount.parse::<f64>() {
            *map.entry(h.token.clone()).or_insert(0.0) += v;
        }
    }
    map.into_iter()
        .map(|(token, total)| HoldingSummary { token, total })
        .collect()
}

#[derive(candid::CandidType, serde::Serialize)]
pub struct Version {
    pub git_sha: &'static str,
    pub build_time: &'static str,
}

#[ic_cdk_macros::query]
pub fn get_version() -> Version {
    metrics::inc_query();
    pay_cycles(*CALL_PRICE_CYCLES);
    Version {
        git_sha: option_env!("GIT_SHA").unwrap_or("unknown"),
        build_time: option_env!("BUILD_TIME").unwrap_or("unknown"),
    }
}

#[ic_cdk_macros::query]
pub fn get_cycles_log() -> Vec<String> {
    metrics::inc_query();
    pay_cycles(*CALL_PRICE_CYCLES);
    cycles::log()
}

#[ic_cdk_macros::query]
pub fn get_user_settings(principal: Principal) -> user_settings::UserSettings {
    metrics::inc_query();
    pay_cycles(*CALL_PRICE_CYCLES);
    user_settings::get(&principal).unwrap_or_default()
}

#[ic_cdk_macros::update]
pub fn update_user_settings(principal: Principal, settings: user_settings::UserSettings) {
    metrics::inc_query();
    pay_cycles(*CALL_PRICE_CYCLES);
    let caller = ic_cdk::caller();
    if caller != principal {
        ic_cdk::api::trap("unauthorized");
    }
    user_settings::update(principal, settings);
    cache::get().remove(&principal);
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
    pay_cycles(*CALL_PRICE_CYCLES);
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
    ClaimStatus {
        attempts,
        window_expires,
        locked,
    }
}

#[ic_cdk_macros::query]
pub fn health_check() -> &'static str {
    metrics::inc_query();
    pay_cycles(*CALL_PRICE_CYCLES);
    "ok"
}

#[derive(candid::CandidType, serde::Serialize, serde::Deserialize)]
pub struct TokenTotal {
    pub token: String,
    pub total: f64,
}

fn summarize(holdings: &[Holding]) -> Vec<TokenTotal> {
    use std::collections::HashMap;
    let mut map: HashMap<String, f64> = HashMap::new();
    for h in holdings {
        if let Ok(v) = h.amount.parse::<f64>() {
            *map.entry(h.token.clone()).or_default() += v;
        }
    }
    let mut out: Vec<TokenTotal> = map
        .into_iter()
        .map(|(token, total)| TokenTotal { token, total })
        .collect();
    out.sort_by(|a, b| a.token.cmp(&b.token));
    out
}

#[ic_cdk_macros::query]
pub async fn get_summary(principal: Principal) -> Vec<TokenTotal> {
    metrics::inc_query();
    pay_cycles(*CALL_PRICE_CYCLES);
    let holdings = get_holdings(principal).await;
    summarize(&holdings)
}
