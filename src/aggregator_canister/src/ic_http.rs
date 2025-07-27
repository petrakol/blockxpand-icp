use candid::{CandidType, Deserialize};
use serde_bytes::ByteBuf;

#[derive(Clone, CandidType, Deserialize)]
pub struct Request {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: ByteBuf,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct Response {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: ByteBuf,
}
