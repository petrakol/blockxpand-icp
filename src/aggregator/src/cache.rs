use bx_core::Holding;
use candid::Principal;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

pub type Cache = HashMap<Principal, (Vec<Holding>, u64)>;

static CACHE: Lazy<Mutex<Cache>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn get_mut() -> std::sync::MutexGuard<'static, Cache> {
    CACHE.lock().unwrap()
}
