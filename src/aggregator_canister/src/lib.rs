use aggregator::pool_registry;
pub use aggregator::*;

#[ic_cdk_macros::init]
fn init() {
    ic_cdk::spawn(async { pool_registry::refresh().await });
    #[cfg(target_arch = "wasm32")]
    {
        pool_registry::schedule_refresh();
        aggregator::lp_cache::schedule_eviction();
    }
}
