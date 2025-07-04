pub mod aggregator;
#[derive(serde::Serialize)]
pub struct Holding {
    pub source: String,
    pub token: String,
    pub amount: String,
    pub status: String,
}
