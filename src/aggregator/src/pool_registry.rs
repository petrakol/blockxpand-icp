use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use candid::CandidType;
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PoolMeta {
    pub id: String,
    pub token_a: String,
    pub token_b: String,
    pub decimals_a: u8,
    pub decimals_b: u8,
    pub image_a: Option<String>,
    pub image_b: Option<String>,
}

#[derive(Deserialize)]
struct PoolsFile {
    pool: Vec<PoolMeta>,
}

static REGISTRY: Lazy<RwLock<HashMap<String, PoolMeta>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub fn list() -> Vec<PoolMeta> {
    REGISTRY.read().unwrap().values().cloned().collect()
}

pub async fn refresh() {
    let path = std::env::var("POOLS_FILE").unwrap_or_else(|_| "data/pools.toml".into());
    let content = match tokio::fs::read_to_string(&path).await {
        Ok(c) => c,
        Err(_) => return,
    };
    let pf: PoolsFile = match toml::from_str(&content) {
        Ok(p) => p,
        Err(_) => return,
    };
    let mut map = HashMap::new();
    for p in pf.pool.into_iter() {
        map.insert(p.id.clone(), p);
    }
    *REGISTRY.write().unwrap() = map;
}

#[cfg(target_arch = "wasm32")]
pub fn schedule_refresh() {
    use std::time::Duration;
    ic_cdk_timers::set_timer_interval(Duration::from_secs(86_400), || {
        ic_cdk::spawn(async { refresh().await });
    });
}

pub fn graphql(_query: String) -> String {
    let data = list();
    serde_json::json!({"data": {"pools": data}}).to_string()
}
