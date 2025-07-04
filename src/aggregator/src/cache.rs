use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use candid::Principal;
use bx_core::Holding;

pub type Cache = HashMap<Principal, (Vec<Holding>, u64)>;

static CACHE: Lazy<Mutex<Cache>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn get_mut() -> std::sync::MutexGuard<'static, Cache> {
    CACHE.lock().unwrap()
}
