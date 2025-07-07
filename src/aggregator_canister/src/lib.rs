pub use aggregator::*;

#[ic_cdk_macros::init]
fn init() {
    #[cfg(not(target_arch = "wasm32"))]
    ic_cdk::spawn(async { aggregator::utils::load_dex_config().await });
    #[cfg(not(target_arch = "wasm32"))]
    aggregator::utils::watch_dex_config();
    ic_cdk::spawn(async { aggregator::pool_registry::refresh().await });
    aggregator::pool_registry::schedule_refresh();
    aggregator::lp_cache::schedule_eviction();
    aggregator::warm::init();
}

#[ic_cdk_macros::pre_upgrade]
fn pre_upgrade() {
    let log = aggregator::cycles::take_log();
    ic_cdk::storage::stable_save((log,)).unwrap();
}

#[ic_cdk_macros::post_upgrade]
fn post_upgrade() {
    if let Ok((log,)) = ic_cdk::storage::stable_restore::<(Vec<String>,)>() {
        aggregator::cycles::set_log(log);
    }
}

#[ic_cdk_macros::heartbeat]
async fn heartbeat() {
    aggregator::cycles::tick().await;
    aggregator::warm::tick().await;
}

#[cfg(feature = "export_candid")]
ic_cdk::export::candid::export_service!();
