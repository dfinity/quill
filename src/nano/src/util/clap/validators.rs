use crate::lib::nns_types::icpts::ICPTs;
use std::str::FromStr;

pub fn e8s_validator(e8s: &str) -> Result<(), String> {
    if e8s.parse::<u64>().is_ok() {
        return Ok(());
    }
    Err("Must specify a non negative whole number.".to_string())
}

pub fn icpts_amount_validator(icpts: &str) -> Result<(), String> {
    ICPTs::from_str(icpts).map(|_| ())
}

pub fn memo_validator(memo: &str) -> Result<(), String> {
    if memo.parse::<u64>().is_ok() {
        return Ok(());
    }
    Err("Must specify a non negative whole number.".to_string())
}

pub fn is_hsm_key_id(key_id: &str) -> Result<(), String> {
    if key_id.len() % 2 != 0 {
        Err("Key id must consist of an even number of hex digits".to_string())
    } else if key_id.contains(|c: char| !c.is_ascii_hexdigit()) {
        Err("Key id must contain only hex digits".to_string())
    } else {
        Ok(())
    }
}
