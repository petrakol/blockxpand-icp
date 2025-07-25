use bx_core::Holding;
use candid::Principal;
use dashmap::DashMap;
use once_cell::sync::Lazy;

use crate::HoldingSummary;

pub type Cache = DashMap<Principal, (Vec<Holding>, Vec<HoldingSummary>, u64)>;

static CACHE: Lazy<Cache> = Lazy::new(DashMap::new);

pub fn get() -> &'static Cache {
    &CACHE
}
