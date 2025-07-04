pub mod dex_fetchers;
pub mod ledger_fetcher;
pub mod neuron_fetcher;

use crate::Holding;
use candid::Principal;

pub async fn get_holdings(principal: Principal) -> Vec<Holding> {
    let mut holdings = Vec::new();
    holdings.extend(ledger_fetcher::fetch(principal).await);
    holdings.extend(neuron_fetcher::fetch(principal).await);
    holdings.extend(dex_fetchers::fetch(principal).await);
    holdings
}
