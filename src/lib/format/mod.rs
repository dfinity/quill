use bigdecimal::BigDecimal;
use candid::{Nat, Principal};
use chrono::{DateTime, TimeZone, Utc};
use icrc_ledger_types::icrc1::account::Account;
use itertools::Itertools;
use num_bigint::ToBigInt;

use super::ParsedAccount;

pub mod ckbtc;
pub mod gtc;
pub mod ledger;
pub mod nns_governance;
pub mod registry;
pub mod sns_governance;
pub mod sns_root;
pub mod sns_swap;
pub mod sns_wasm;

pub fn format_datetime(datetime: DateTime<Utc>) -> String {
    format!("{} UTC", datetime.format("%b %d %Y %X"))
}

pub fn format_timestamp_seconds(seconds: u64) -> String {
    format_datetime(Utc.timestamp_opt(seconds.try_into().unwrap(), 0).unwrap())
}

pub fn format_timestamp_nanoseconds(nanoseconds: u64) -> String {
    format_datetime(Utc.timestamp_nanos(nanoseconds.try_into().unwrap()))
}

pub fn format_duration_seconds(mut seconds: u64) -> String {
    // Required for magic numbers like '8 years' to show up as such instead of '8 years 2 days'.
    const SECONDS_PER_YEAR: u64 = 31557600; // 365.25 * 24 * 60 * 60
    const SECONDS_PER_MONTH: u64 = SECONDS_PER_YEAR / 12;
    const SECONDS_PER_MINUTE: u64 = 60;
    const SECONDS_PER_HOUR: u64 = SECONDS_PER_MINUTE * 60;
    const SECONDS_PER_DAY: u64 = SECONDS_PER_HOUR * 24;
    let years = seconds / SECONDS_PER_YEAR;
    seconds %= SECONDS_PER_YEAR;
    let months = seconds / SECONDS_PER_MONTH;
    seconds %= SECONDS_PER_MONTH;
    let days = seconds / SECONDS_PER_DAY;
    seconds %= SECONDS_PER_DAY;
    let hours = seconds / SECONDS_PER_HOUR;
    seconds %= SECONDS_PER_HOUR;
    let minutes = seconds / SECONDS_PER_MINUTE;
    seconds %= SECONDS_PER_MINUTE;
    [
        (years, "year"),
        (months, "month"),
        (days, "day"),
        (hours, "hour"),
        (minutes, "minute"),
        (seconds, "second"),
    ]
    .iter()
    .filter(|&&(n, _)| n != 0)
    .format_with(", ", |&(n, t), f| {
        f(&format_args!(
            "{n} {t}{s}",
            s = if n == 1 { "" } else { "s" }
        ))
    })
    .to_string()
}

pub fn icrc1_account(owner: Principal, subaccount: Option<[u8; 32]>) -> ParsedAccount {
    ParsedAccount(Account { owner, subaccount })
}

pub fn format_t_cycles(cycles: Nat) -> String {
    let t_cycles = BigDecimal::new(cycles.0.into(), 12);
    let e10 = t_cycles.digits();
    if e10 < 14 {
        format!("{:.1}T", t_cycles)
    } else {
        format!("{:.0}T", t_cycles)
    }
}

pub fn format_n_cycles(cycles: Nat) -> String {
    let e10 = BigDecimal::from(cycles.0.to_bigint().unwrap()).digits();
    if e10 < 4 {
        return cycles.to_string();
    }
    let unit = (e10 - 1) / 3;
    let letter = b"KMBTQ"[unit as usize - 1] as char;
    let scale = unit * 3;
    let printable = BigDecimal::new(cycles.0.into(), scale as i64);
    if e10 - scale == 1 {
        format!("{printable:.1}{letter}")
    } else {
        format!("{printable:.0}{letter}")
    }
}

pub mod filters {
    use bigdecimal::BigDecimal;
    use candid::{Nat, Principal};

    use crate::lib::{ckbtc_canister_id, e8s_to_tokens, ledger_canister_id};

    use super::{
        format_duration_seconds, format_n_cycles, format_t_cycles, format_timestamp_nanoseconds,
        format_timestamp_seconds,
    };

    pub fn tokens_e8s(e8s: impl IntoNat, units: &str) -> askama::Result<String> {
        if units == "." {
            Ok(format!("{}", e8s_to_tokens(e8s.into_nat())))
        } else {
            Ok(format!("{} {units}", e8s_to_tokens(e8s.into_nat())))
        }
    }

    pub fn tokens_e8s_guess(e8s: impl IntoNat, canister: &Principal) -> askama::Result<String> {
        if *canister == ledger_canister_id() {
            tokens_e8s(e8s, "ICP")
        } else if *canister == ckbtc_canister_id(false) {
            tokens_e8s(e8s, "ckBTC")
        } else if *canister == ckbtc_canister_id(true) {
            tokens_e8s(e8s, "ckTESTBTC")
        } else {
            tokens_e8s(e8s, "tokens")
        }
    }

    pub fn dur_seconds(seconds: impl ToU64) -> askama::Result<String> {
        Ok(format_duration_seconds(seconds.to_u64()))
    }

    pub fn ts_seconds(seconds: impl ToU64) -> askama::Result<String> {
        Ok(format_timestamp_seconds(seconds.to_u64()))
    }

    pub fn ts_nanos(seconds: impl ToU64) -> askama::Result<String> {
        Ok(format_timestamp_nanoseconds(seconds.to_u64()))
    }

    pub fn cycles_t(cycles: impl IntoNat) -> askama::Result<String> {
        Ok(format_t_cycles(cycles.into_nat()))
    }

    pub fn cycles_precise(cycles: impl IntoNat) -> askama::Result<String> {
        Ok(format_n_cycles(cycles.into_nat()))
    }

    pub fn hex(bytes: impl AsRef<[u8]>) -> askama::Result<String> {
        Ok(hex::encode(bytes))
    }

    pub trait IntoNat {
        fn into_nat(self) -> Nat;
    }

    impl IntoNat for u64 {
        fn into_nat(self) -> Nat {
            Nat::from(self)
        }
    }

    impl IntoNat for Nat {
        fn into_nat(self) -> Nat {
            self
        }
    }

    impl<T> IntoNat for &T
    where
        T: IntoNat + Clone,
    {
        fn into_nat(self) -> Nat {
            T::into_nat(self.clone())
        }
    }

    pub trait ToU64 {
        fn to_u64(&self) -> u64;
    }

    impl ToU64 for u64 {
        fn to_u64(&self) -> u64 {
            *self
        }
    }

    impl<T> ToU64 for &T
    where
        T: ToU64,
    {
        fn to_u64(&self) -> u64 {
            T::to_u64(self)
        }
    }
}

#[test]
fn magic_durations() {
    assert_eq!(format_duration_seconds(15_778_800), "6 months");
    assert_eq!(format_duration_seconds(252_460_800), "8 years");
}

#[test]
fn cycle_units() {
    assert_eq!(format_t_cycles(100_000_000_000_u64.into()), "0.1T");
    assert_eq!(format_t_cycles(1_100_000_000_000_u64.into()), "1.1T");
    assert_eq!(format_t_cycles(10_100_000_000_000_u64.into()), "10T");
    assert_eq!(format_t_cycles(1_000_000_000_000_000_u64.into()), "1000T");
    assert_eq!(format_n_cycles(1_000_000_000_000_000_u64.into()), "1.0Q");
    assert_eq!(format_n_cycles(100_000_000_000_u64.into()), "100B");
    assert_eq!(format_n_cycles(10_100_000_000_u64.into()), "10B");
    assert_eq!(format_n_cycles(1_100_000_000_u64.into()), "1.1B");
    assert_eq!(format_n_cycles(1_000_u64.into()), "1.0K");
    assert_eq!(format_n_cycles(1_100_u64.into()), "1.1K");
    assert_eq!(format_n_cycles(100_u64.into()), "100");
}
