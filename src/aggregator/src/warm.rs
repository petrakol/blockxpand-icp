use candid::Principal;
use once_cell::sync::Lazy;
use std::collections::{HashSet, VecDeque};
use std::sync::Mutex;
use tracing::{debug, info};

struct Entry {
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    cid: Principal,
    next: u64,
}

static QUEUE: Lazy<Mutex<VecDeque<Entry>>> = Lazy::new(|| Mutex::new(VecDeque::new()));

const ITEMS_PER_TICK: usize = 3;
const MAX_QUEUE_SIZE: usize = 128;

pub fn init() {
    let now = crate::utils::now();
    let mut q = QUEUE.lock().unwrap();
    q.clear();
    let mut seen = HashSet::new();
    for cid in crate::ledger_fetcher::LEDGERS.iter().cloned() {
        if q.len() >= MAX_QUEUE_SIZE {
            break;
        }
        if seen.insert(cid) {
            q.push_back(Entry { cid, next: now });
        }
    }
    for cid in crate::utils::dex_ids() {
        if q.len() >= MAX_QUEUE_SIZE {
            break;
        }
        if seen.insert(cid) {
            q.push_back(Entry { cid, next: now });
        }
    }
    info!(queued = q.len(), "warm queue initialised");
}

pub async fn tick() {
    for _ in 0..ITEMS_PER_TICK {
        let entry_opt = {
            let mut q = QUEUE.lock().unwrap();
            q.pop_front()
        };
        let mut entry = match entry_opt {
            Some(e) => e,
            None => break,
        };

        if crate::utils::now() >= entry.next {
            #[cfg(not(target_arch = "wasm32"))]
            crate::ledger_fetcher::warm_metadata(entry.cid).await;
            crate::utils::warm_icrc_metadata(entry.cid).await;
            debug!("warmed metadata for {}", entry.cid);
            entry.next = crate::utils::now() + crate::utils::DAY_NS;
        }

        {
            let mut q = QUEUE.lock().unwrap();
            q.push_back(entry);
        }
    }
}

#[cfg(test)]
pub fn len() -> usize {
    QUEUE.lock().unwrap().len()
}

#[cfg(test)]
pub fn dump() -> Vec<Principal> {
    QUEUE.lock().unwrap().iter().map(|e| e.cid).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn gen_principal(i: u8) -> Principal {
        let bytes = [i; 32];
        Principal::self_authenticating(&bytes)
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn init_bounds_queue() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "[ledgers]").unwrap();
        for i in 0..150u8 {
            writeln!(f, "L{i} = \"{}\"", gen_principal(i).to_text()).unwrap();
        }
        writeln!(f, "[dex]").unwrap();
        for i in 0..150u8 {
            writeln!(f, "D{i} = \"{}\"", gen_principal(i).to_text()).unwrap();
        }
        std::env::set_var("LEDGERS_FILE", f.path());
        crate::utils::load_dex_config().await;
        once_cell::sync::Lazy::force(&crate::ledger_fetcher::LEDGERS);
        init();
        assert_eq!(len(), MAX_QUEUE_SIZE);
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn init_deduplicates() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "[ledgers]\nA = \"aaaaa-aa\"\nB = \"aaaaa-aa\"").unwrap();
        writeln!(f, "[dex]\nX = \"aaaaa-aa\"\nY = \"aaaaa-aa\"").unwrap();
        std::env::set_var("LEDGERS_FILE", f.path());
        crate::utils::load_dex_config().await;
        once_cell::sync::Lazy::force(&crate::ledger_fetcher::LEDGERS);
        init();
        assert_eq!(len(), 2);
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn deterministic_after_reinit() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "[ledgers]\nMOCK = \"aaaaa-aa\"").unwrap();
        writeln!(f, "[dex]\nX = \"bbbbbb-baaaa-aaaaa-aaadq-cai\"").unwrap();
        std::env::set_var("LEDGERS_FILE", f.path());
        crate::utils::load_dex_config().await;
        once_cell::sync::Lazy::force(&crate::ledger_fetcher::LEDGERS);
        init();
        let first = dump();
        init();
        let second = dump();
        assert_eq!(first, second);
    }
}
