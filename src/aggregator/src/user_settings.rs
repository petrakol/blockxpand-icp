use candid::Principal;
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

#[derive(
    Default, Clone, candid::CandidType, serde::Serialize, serde::Deserialize, PartialEq, Debug,
)]
pub struct UserSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledgers: Option<HashSet<Principal>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dexes: Option<HashSet<String>>,
}

static SETTINGS: Lazy<Mutex<HashMap<Principal, UserSettings>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(candid::CandidType, serde::Serialize, serde::Deserialize)]
pub struct StableEntry {
    pub principal: Principal,
    pub settings: UserSettings,
}

pub fn get(principal: &Principal) -> Option<UserSettings> {
    SETTINGS.lock().unwrap().get(principal).cloned()
}

pub fn update(principal: Principal, settings: UserSettings) {
    SETTINGS.lock().unwrap().insert(principal, settings);
}

pub fn remove(principal: Principal) {
    SETTINGS.lock().unwrap().remove(&principal);
}

pub fn stable_save() -> Vec<StableEntry> {
    SETTINGS
        .lock()
        .unwrap()
        .iter()
        .map(|(p, s)| StableEntry {
            principal: *p,
            settings: s.clone(),
        })
        .collect()
}

pub fn stable_restore(entries: Vec<StableEntry>) {
    let mut map = SETTINGS.lock().unwrap();
    map.clear();
    for e in entries {
        map.insert(e.principal, e.settings);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;
    use std::collections::HashSet;

    #[test]
    fn crud() {
        let p = Principal::from_text("aaaaa-aa").unwrap();
        assert!(get(&p).is_none());
        let mut set = HashSet::new();
        set.insert(p);
        let s1 = UserSettings {
            ledgers: Some(set.clone()),
            dexes: None,
        };
        update(p, s1.clone());
        assert_eq!(get(&p), Some(s1.clone()));
        let mut dex = HashSet::new();
        dex.insert("ICPSWAP_FACTORY".into());
        let s2 = UserSettings {
            ledgers: None,
            dexes: Some(dex.clone()),
        };
        update(p, s2.clone());
        assert_eq!(get(&p), Some(s2.clone()));
        remove(p);
        assert!(get(&p).is_none());
    }
}
