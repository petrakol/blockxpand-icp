use candid::Principal;
use dashmap::DashMap;
use once_cell::sync::Lazy;

#[derive(
    Default, Clone, candid::CandidType, serde::Serialize, serde::Deserialize, PartialEq, Debug,
)]
pub struct UserSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledgers: Option<Vec<Principal>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dexes: Option<Vec<String>>,
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
            ledgers: Some(vec![p]),
            dexes: None,
        };
        update(p, s1.clone());
        assert_eq!(get(&p), Some(s1.clone()));
        let s2 = UserSettings {
            ledgers: None,
            dexes: Some(vec!["ICPSWAP_FACTORY".into()]),
        };
        update(p, s2.clone());
        assert_eq!(get(&p), Some(s2.clone()));
        remove(p);
        assert!(get(&p).is_none());
    }
}
