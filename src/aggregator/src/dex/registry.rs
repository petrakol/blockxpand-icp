use super::{
    dex_icpswap::IcpswapAdapter, dex_infinity::InfinityAdapter, dex_sonic::SonicAdapter,
    sns_adapter::SnsAdapter, DexAdapter,
};
use once_cell::sync::Lazy;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct AdapterEntry {
    pub name: String,
    pub adapter: Arc<dyn DexAdapter>,
}

static ADAPTERS: Lazy<RwLock<Vec<AdapterEntry>>> = Lazy::new(|| RwLock::new(Vec::new()));

#[cfg(not(target_arch = "wasm32"))]
pub async fn load_adapters() {
    use candid::Principal;
    use std::fs;
    use std::path::Path;

    let path = crate::utils::ledgers_path();
    if !Path::new(&path).exists() {
        return;
    }
    let text = fs::read_to_string(&path).unwrap_or_default();
    let value: toml::Value =
        toml::from_str(&text).unwrap_or(toml::Value::Table(Default::default()));
    let mut dex_table = value
        .get("dex")
        .and_then(|t| t.as_table())
        .cloned()
        .unwrap_or_default();
    for key in dex_table.clone().keys() {
        if let Ok(v) = std::env::var(key) {
            dex_table.insert(key.clone(), toml::Value::String(v));
        }
    }
    let mut list = Vec::new();
    for (name, val) in dex_table {
        if let Some(id_str) = val.as_str() {
            if let Ok(principal) = Principal::from_text(id_str) {
                if let Some(adapter) = match name.as_str() {
                    "ICPSWAP_FACTORY" => Some(Arc::new(IcpswapAdapter) as Arc<dyn DexAdapter>),
                    "SONIC_ROUTER" => Some(Arc::new(SonicAdapter) as Arc<dyn DexAdapter>),
                    "INFINITY_VAULT" => Some(Arc::new(InfinityAdapter) as Arc<dyn DexAdapter>),
                    n if n.starts_with("SNS_") => {
                        Some(Arc::new(SnsAdapter::new(principal)) as Arc<dyn DexAdapter>)
                    }
                    _ => None,
                } {
                    list.push(AdapterEntry { name, adapter });
                }
            }
        }
    }
    let mut reg = ADAPTERS.write().unwrap();
    *reg = list;
}

#[cfg(target_arch = "wasm32")]
pub async fn load_adapters() {}

pub fn get() -> Vec<AdapterEntry> {
    ADAPTERS.read().unwrap().clone()
}
