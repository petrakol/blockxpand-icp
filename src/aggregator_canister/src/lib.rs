pub use aggregator::*;
pub mod ic_http;
use async_graphql::{EmptyMutation, EmptySubscription, Object, Request as GqlRequest, Schema};
use once_cell::sync::Lazy;

#[ic_cdk_macros::init]
fn init() {
    aggregator::logging::init();
    #[cfg(not(target_arch = "wasm32"))]
    ic_cdk::spawn(async {
        aggregator::utils::load_dex_config().await;
        aggregator::dex::registry::load_adapters().await;
    });
    #[cfg(not(target_arch = "wasm32"))]
    aggregator::utils::watch_dex_config();
    #[cfg(not(target_arch = "wasm32"))]
    aggregator::pool_registry::watch_pools_file();
    ic_cdk::spawn(async { aggregator::pool_registry::refresh().await });
    aggregator::pool_registry::schedule_refresh();
    aggregator::lp_cache::schedule_eviction();
    aggregator::warm::init();
}

#[ic_cdk_macros::pre_upgrade]
fn pre_upgrade() {
    let log = aggregator::cycles::take_log();
    let meta = aggregator::ledger_fetcher::stable_save();
    let lp = aggregator::lp_cache::stable_save();
    let settings = aggregator::user_settings::stable_save();
    let metrics = aggregator::metrics::stable_save();
    ic_cdk::storage::stable_save((log, meta, lp, settings, metrics)).unwrap();
}

#[ic_cdk_macros::post_upgrade]
fn post_upgrade() {
    if let Ok((log, meta, lp, settings, metrics)) = ic_cdk::storage::stable_restore::<(
        Vec<String>,
        Vec<aggregator::ledger_fetcher::StableMeta>,
        Vec<aggregator::lp_cache::StableEntry>,
        Vec<aggregator::user_settings::StableEntry>,
        (u64, u64, u64, u64, u64, u64, u64, u64),
    )>() {
        aggregator::cycles::set_log(log);
        aggregator::ledger_fetcher::stable_restore(meta);
        aggregator::lp_cache::stable_restore(lp);
        aggregator::user_settings::stable_restore(settings);
        aggregator::metrics::stable_restore(metrics);
    }
}

#[ic_cdk_macros::heartbeat]
async fn heartbeat() {
    aggregator::metrics::inc_heartbeat(aggregator::utils::now());
    aggregator::cycles::tick().await;
    aggregator::warm::tick().await;
}

#[ic_cdk_macros::query]
fn get_metrics() -> String {
    pay_cycles(*CALL_PRICE_CYCLES);
    serde_json::to_string(&aggregator::metrics::get()).unwrap()
}

use crate::ic_http::{Request as HttpRequest, Response as HttpResponse};
use aggregator::{pay_cycles, CALL_PRICE_CYCLES};

#[derive(async_graphql::SimpleObject)]
struct GHolding {
    source: String,
    token: String,
    amount: String,
    status: String,
}

impl From<bx_core::Holding> for GHolding {
    fn from(h: bx_core::Holding) -> Self {
        GHolding {
            source: h.source,
            token: h.token,
            amount: h.amount,
            status: h.status,
        }
    }
}

#[derive(async_graphql::SimpleObject)]
struct GTokenTotal {
    token: String,
    total: f64,
}

impl From<aggregator::TokenTotal> for GTokenTotal {
    fn from(t: aggregator::TokenTotal) -> Self {
        GTokenTotal {
            token: t.token,
            total: t.total,
        }
    }
}
use serde_bytes::ByteBuf;

#[derive(Default)]
struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn holdings(&self, principal: String) -> async_graphql::Result<Vec<GHolding>> {
        let p = candid::Principal::from_text(&principal)?;
        Ok(aggregator::get_holdings(p)
            .await
            .into_iter()
            .map(GHolding::from)
            .collect())
    }

    async fn summary(&self, principal: String) -> async_graphql::Result<Vec<GTokenTotal>> {
        let p = candid::Principal::from_text(&principal)?;
        Ok(aggregator::get_summary(p)
            .await
            .into_iter()
            .map(GTokenTotal::from)
            .collect())
    }
}

static SCHEMA: Lazy<Schema<QueryRoot, EmptyMutation, EmptySubscription>> =
    Lazy::new(|| Schema::build(QueryRoot::default(), EmptyMutation, EmptySubscription).finish());
#[ic_cdk_macros::query]
pub async fn http_request(req: HttpRequest) -> HttpResponse {
    use candid::Principal;

    pay_cycles(*CALL_PRICE_CYCLES);

    let path = req.url.split('?').next().unwrap_or("");
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let not_found = || HttpResponse {
        status_code: 404,
        headers: vec![("Content-Type".into(), "application/json".into())],
        body: ByteBuf::from(r#"{"error":"not found"}"#),
    };

    match parts.as_slice() {
        ["holdings", pid] => {
            let principal = match Principal::from_text(pid) {
                Ok(p) => p,
                Err(_) => return not_found(),
            };
            let holdings = aggregator::get_holdings(principal).await;
            let body = serde_json::to_vec(&holdings).unwrap();
            HttpResponse {
                status_code: 200,
                headers: vec![("Content-Type".into(), "application/json".into())],
                body: ByteBuf::from(body),
            }
        }
        ["metrics"] => {
            let metrics = aggregator::metrics::get();
            let body = serde_json::to_vec(&metrics).unwrap();
            HttpResponse {
                status_code: 200,
                headers: vec![("Content-Type".into(), "application/json".into())],
                body: ByteBuf::from(body),
            }
        }
        ["summary", pid] => {
            let principal = match Principal::from_text(pid) {
                Ok(p) => p,
                Err(_) => return not_found(),
            };
            let summary = aggregator::get_summary(principal).await;
            let body = serde_json::to_vec(&summary).unwrap();
            HttpResponse {
                status_code: 200,
                headers: vec![("Content-Type".into(), "application/json".into())],
                body: ByteBuf::from(body),
            }
        }
        ["graphql"] => {
            let gql_req: GqlRequest = match serde_json::from_slice(req.body.as_ref()) {
                Ok(r) => r,
                Err(_) => {
                    let query = std::str::from_utf8(req.body.as_ref()).unwrap_or("");
                    GqlRequest::new(query)
                }
            };
            let resp = SCHEMA.execute(gql_req).await;
            let body = serde_json::to_vec(&resp).unwrap();
            HttpResponse {
                status_code: 200,
                headers: vec![("Content-Type".into(), "application/json".into())],
                body: ByteBuf::from(body),
            }
        }
        _ => not_found(),
    }
}
#[cfg(feature = "export_candid")]
ic_cdk::export_candid!();

#[cfg(test)]
mod tests {
    use super::*;
    use aggregator::cache;
    use bx_core::Holding;
    use serde_json;
    use serial_test::serial;

    #[tokio::test]
    #[serial_test::serial]
    async fn http_paths() {
        let p = candid::Principal::from_text("aaaaa-aa").unwrap();
        cache::get().clear();
        let holdings = vec![
            Holding {
                source: "test".into(),
                token: "AAA".into(),
                amount: "1".into(),
                status: "ok".into(),
            },
            Holding {
                source: "test".into(),
                token: "AAA".into(),
                amount: "2".into(),
                status: "ok".into(),
            },
        ];
        let summary = {
            use std::collections::BTreeMap;
            use rust_decimal::prelude::{FromStr, ToPrimitive, Zero};
            let mut map: BTreeMap<String, rust_decimal::Decimal> = BTreeMap::new();
            for h in &holdings {
                if let Ok(v) = rust_decimal::Decimal::from_str(&h.amount) {
                    *map.entry(h.token.clone()).or_insert(rust_decimal::Decimal::ZERO) += v;
                }
            }
            map.into_iter()
                .map(|(token, total)| aggregator::HoldingSummary {
                    token,
                    total: total.to_f64().unwrap_or(0.0),
                })
                .collect::<Vec<_>>()
        };
        cache::get().insert(p, (holdings.clone(), summary, aggregator::utils::now()));

        let req = HttpRequest {
            method: "GET".into(),
            url: format!("/holdings/{p}"),
            headers: vec![],
            body: ByteBuf::default(),
        };
        let resp = http_request(req).await;
        assert_eq!(resp.status_code, 200u16);

        let req = HttpRequest {
            method: "GET".into(),
            url: format!("/summary/{p}"),
            headers: vec![],
            body: ByteBuf::default(),
        };
        let resp = http_request(req).await;
        assert_eq!(resp.status_code, 200u16);
        let body = std::str::from_utf8(resp.body.as_ref()).unwrap();
        println!("body http: {}", body);
        assert!(body.contains("AAA"));
        assert!(body.contains("3"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn graphql_query() {
        let p = candid::Principal::from_text("aaaaa-aa").unwrap();
        cache::get().clear();
        cache::get().insert(
            p,
            (
                vec![Holding {
                    source: "test".into(),
                    token: "BBB".into(),
                    amount: "5".into(),
                    status: "ok".into(),
                }],
                vec![aggregator::HoldingSummary {
                    token: "BBB".into(),
                    total: 5.0,
                }],
                aggregator::utils::now(),
            ),
        );
        let query = format!("{{ summary(principal: \"{p}\") {{ token total }} }}");
        let req = HttpRequest {
            method: "POST".into(),
            url: "/graphql".into(),
            headers: vec![],
            body: ByteBuf::from(serde_json::to_vec(&serde_json::json!({"query": query})).unwrap()),
        };
        let resp = http_request(req).await;
        assert_eq!(resp.status_code, 200u16);
        let body = std::str::from_utf8(resp.body.as_ref()).unwrap();
        assert!(body.contains("BBB"));
    }
}
