use candid::Principal;
use dashmap::DashMap;
use once_cell::sync::Lazy;

use serde::{Deserialize, Serialize};

#[derive(Default, Clone, candid::CandidType, Serialize, Deserialize, PartialEq, Debug)]
pub struct UserSettings {
    pub preferred_ledgers: Vec<String>,
    pub preferred_dexes: Vec<String>,
    pub dark_mode: bool,
}

static SETTINGS: Lazy<DashMap<Principal, UserSettings>> = Lazy::new(DashMap::new);

#[derive(candid::CandidType, serde::Serialize, serde::Deserialize)]
pub struct StableEntry {
    pub principal: Principal,
    pub settings: UserSettings,
}

pub fn get(principal: &Principal) -> Option<UserSettings> {
    SETTINGS.get(principal).map(|e| e.value().clone())
}

pub fn update(principal: Principal, settings: UserSettings) {
    SETTINGS.insert(principal, settings);
}

pub fn remove(principal: Principal) {
    SETTINGS.remove(&principal);
}

pub fn stable_save() -> Vec<StableEntry> {
    SETTINGS
        .iter()
        .map(|e| StableEntry {
            principal: *e.key(),
            settings: e.value().clone(),
        })
        .collect()
}

pub fn stable_restore(entries: Vec<StableEntry>) {
    SETTINGS.clear();
    for e in entries {
        SETTINGS.insert(e.principal, e.settings);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;

    #[test]
    fn crud() {
        let p = Principal::from_text("aaaaa-aa").unwrap();
        assert!(get(&p).is_none());
        let s1 = UserSettings {
            preferred_ledgers: vec![p.to_text()],
            preferred_dexes: Vec::new(),
            dark_mode: false,
        };
        update(p, s1.clone());
        assert_eq!(get(&p), Some(s1.clone()));
        let s2 = UserSettings {
            preferred_ledgers: Vec::new(),
            preferred_dexes: vec!["ICPSWAP_FACTORY".to_string()],
            dark_mode: true,
        };
        update(p, s2.clone());
        assert_eq!(get(&p), Some(s2.clone()));
        remove(p);
        assert!(get(&p).is_none());
    }
}
