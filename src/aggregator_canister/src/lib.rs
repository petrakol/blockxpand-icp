pub use aggregator::*;

#[ic_cdk_macros::init]
fn init() {
    ic_cdk::spawn(async { aggregator::utils::load_dex_config().await });
    aggregator::utils::watch_dex_config();
    ic_cdk::spawn(async { aggregator::pool_registry::refresh().await });
    aggregator::pool_registry::schedule_refresh();
    aggregator::lp_cache::schedule_eviction();
}
