pub use aggregator::*;

#[ic_cdk_macros::init]
fn init() {
    ic_cdk::spawn(async { aggregator::pool_registry::refresh().await });
    aggregator::pool_registry::schedule_refresh();
    #[cfg(target_arch = "wasm32")]
    {
        aggregator::lp_cache::schedule_eviction();
    }
}
