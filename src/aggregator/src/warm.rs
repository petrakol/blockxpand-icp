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
            {
                crate::ledger_fetcher::warm_metadata(entry.cid).await;
                crate::utils::warm_icrc_metadata(entry.cid).await;
            }
            entry.next = crate::utils::now() + crate::utils::DAY_NS;
        }

        {
            let mut q = QUEUE.lock().unwrap();
            q.push_back(entry);
        }
    }
}
