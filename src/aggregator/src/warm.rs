use candid::Principal;
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::sync::Mutex;

struct Entry {
    cid: Principal,
    next: u64,
}

static QUEUE: Lazy<Mutex<VecDeque<Entry>>> = Lazy::new(|| Mutex::new(VecDeque::new()));

const ITEMS_PER_TICK: usize = 3;

pub fn init() {
    let now = crate::utils::now();
    let mut q = QUEUE.lock().unwrap();
    q.clear();
    for cid in crate::ledger_fetcher::LEDGERS.iter().cloned() {
        q.push_back(Entry { cid, next: now });
    }
    #[cfg(not(target_arch = "wasm32"))]
    for cid in crate::utils::dex_ids() {
        q.push_back(Entry { cid, next: now });
    }
}

pub async fn tick() {
    let mut q = QUEUE.lock().unwrap();
    let now = crate::utils::now();
    for _ in 0..ITEMS_PER_TICK {
        let mut entry = match q.pop_front() {
            Some(e) => e,
            None => break,
        };
        if now >= entry.next {
            drop(q);
            #[cfg(not(target_arch = "wasm32"))]
            {
                crate::ledger_fetcher::warm_metadata(entry.cid).await;
                crate::utils::warm_icrc_metadata(entry.cid).await;
            }
            entry.next = crate::utils::now() + crate::utils::DAY_NS;
            q = QUEUE.lock().unwrap();
        }
        q.push_back(entry);
    }
}
