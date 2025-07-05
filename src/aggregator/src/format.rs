use candid::Nat;
#[cfg(not(target_arch = "wasm32"))]
use num_bigint::BigUint;
#[cfg(not(target_arch = "wasm32"))]
use num_integer::Integer;

#[cfg(not(target_arch = "wasm32"))]
pub fn format_amount(n: Nat, decimals: u8) -> String {
    let div = BigUint::from(10u32).pow(decimals as u32);
    let (q, r) = n.0.div_rem(&div);
    let mut frac = r.to_str_radix(10);
    while frac.len() < decimals as usize {
        frac.insert(0, '0');
    }
    if decimals == 0 {
        q.to_str_radix(10)
    } else {
        format!("{}.{frac}", q.to_str_radix(10))
    }
}

#[cfg(target_arch = "wasm32")]
pub fn format_amount(n: Nat, _decimals: u8) -> String {
    n.0.to_string()
}
