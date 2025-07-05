use candid::{CandidType, Nat, Principal};
use ic_cdk_macros::query;
use serde::Deserialize;

#[candid::candid_method(query)]
#[query]
fn icrc1_metadata() -> Vec<(String, candid::types::value::IDLValue)> {
    vec![
        (
            "icrc1:symbol".to_string(),
            candid::types::value::IDLValue::Text("MOCK".to_string()),
        ),
        (
            "icrc1:decimals".to_string(),
            candid::types::value::IDLValue::Nat8(8),
        ),
        (
            "icrc1:fee".to_string(),
            candid::types::value::IDLValue::Nat(100u64.into()),
        ),
    ]
}

#[derive(CandidType, Deserialize)]
struct Account {
    owner: Principal,
    subaccount: Option<Vec<u8>>,
}

#[candid::candid_method(query)]
#[query]
fn icrc1_balance_of(account: Account) -> Nat {
    if account.owner == Principal::anonymous() {
        Nat::from(1_000_000_000u64)
    } else {
        Nat::from(500_000_000u64)
    }
}

ic_cdk::export_candid!();
