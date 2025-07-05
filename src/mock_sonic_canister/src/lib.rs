use candid::{CandidType, Principal};
use ic_cdk_macros::query;
use serde::Deserialize;

#[derive(CandidType, Deserialize, Clone)]
struct Token {
    address: String,
    decimals: u8,
}

#[derive(CandidType, Deserialize, Clone)]
struct PositionInfo {
    token_a: Token,
    token_b: Token,
    token_a_amount: u64,
    token_b_amount: u64,
    reward_token: Token,
    reward_amount: u64,
    auto_compound: bool,
}

#[candid::candid_method(query)]
#[query]
fn get_user_positions(_p: Principal) -> Vec<PositionInfo> {
    vec![
        PositionInfo {
            token_a: Token {
                address: "sonic0".to_string(),
                decimals: 8,
            },
            token_b: Token {
                address: "sonic1".to_string(),
                decimals: 8,
            },
            token_a_amount: 1_000_000_000,
            token_b_amount: 2_000_000_000,
            reward_token: Token {
                address: "SNR".to_string(),
                decimals: 8,
            },
            reward_amount: 50_000_000,
            auto_compound: false,
        },
        PositionInfo {
            token_a: Token {
                address: "sonic2".to_string(),
                decimals: 8,
            },
            token_b: Token {
                address: "sonic3".to_string(),
                decimals: 8,
            },
            token_a_amount: 3_000_000_000,
            token_b_amount: 4_000_000_000,
            reward_token: Token {
                address: "SNR".to_string(),
                decimals: 8,
            },
            reward_amount: 0,
            auto_compound: true,
        },
    ]
}

ic_cdk::export_candid!();
