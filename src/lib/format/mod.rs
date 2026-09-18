use std::fmt;

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
#[cfg(test)]
mod tests;

/// Makes strings terminal-safe by replacing control codes with
/// characters from the 'Control Pictures' block.
#[derive(Clone, Copy)]
pub struct TerminalSafe;

impl askama::filters::Escaper for TerminalSafe {
    fn write_escaped_str<W: fmt::Write>(&self, mut dest: W, string: &str) -> fmt::Result {
        let mut rest = string;
        while let Some(at) = rest.find(|c| depicted(c).is_some()) {
            dest.write_str(&rest[..at])?;
            // `find` reports a char boundary, so the character it found is the
            // first one of the remainder.
            let mut tail = rest[at..].chars();
            let found = tail.next().expect("find reported a character");
            rest = tail.as_str();
            // The `LF` of a `CRLF` ends the line by itself, so the `CR` goes
            // rather than being depicted in front of it. A value is handed to an
            // escaper in whatever chunks its `Display` writes, so a pair split
            // across two of those would still be depicted; only quill's own
            // template text is written piecewise, and none of it contains a `CR`.
            if found == '\r' && rest.starts_with('\n') {
                continue;
            }
            dest.write_char(depicted(found).expect("find matched on a depicted character"))?;
        }
        dest.write_str(rest)
    }

    fn write_escaped_char<W: fmt::Write>(&self, mut dest: W, c: char) -> fmt::Result {
        // There is nothing to look ahead at, so a `CR` here is always depicted.
        dest.write_char(depicted(c).unwrap_or(c))
    }
}

/// Newlines are structural in a rendered response, so they are left alone. Every
/// other control code has no legitimate place in one, and is shown rather than
/// obeyed.
fn depicted(c: char) -> Option<char> {
    (c != '\n').then(|| control_picture(c)).flatten()
}

/// The first character of the Control Pictures block, U+2400 SYMBOL FOR NULL.
/// The block is laid out in step with the C0 controls it depicts.
const CONTROL_PICTURES: u32 = 0x2400;

/// U+2424, which the Control Pictures block offers alongside the per-character
/// pictures for naming a line ending as such.
const SYMBOL_FOR_NEWLINE: char = '␤';

/// The Control Pictures character depicting `c`, or `None` if `c` is not a
/// control code and depicts itself.
fn control_picture(c: char) -> Option<char> {
    match c {
        '\0'..='\u{1f}' => char::from_u32(CONTROL_PICTURES + c as u32),
        '\u{7f}' => Some('␡'), // U+2421 SYMBOL FOR DELETE
        // The C1 range has no pictures. These reach a terminal as two UTF-8
        // bytes rather than as the control codes they name, so little is likely
        // to act on them, but there is no reason to pass them through either.
        '\u{80}'..='\u{9f}' => Some(char::REPLACEMENT_CHARACTER),
        _ => None,
    }
}

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
    if seconds == 0 {
        // Every component would be filtered out below, leaving an empty string.
        return "0 seconds".to_string();
    }
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
        format!("{t_cycles:.1}T")
    } else {
        format!("{t_cycles:.0}T")
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

/// All askama filters go in this module.
pub mod filters {
    use std::fmt::Display;

    use askama::Values;
    use bigdecimal::BigDecimal;
    use candid::{Nat, Principal};
    use ic_base_types::{CanisterId, PrincipalId};
    use ic_nns_constants::canister_id_to_nns_canister_name;
    use indicatif::HumanBytes;

    use crate::lib::{
        ckbtc_canister_id, e8s_to_tokens,
        format::{format_n_cycles, format_t_cycles},
        get_default_role, get_idl_string, ledger_canister_id,
    };

    use super::{
        format_duration_seconds, format_timestamp_nanoseconds, format_timestamp_seconds,
        SYMBOL_FOR_NEWLINE,
    };

    pub fn tokens_e8s(
        e8s: impl IntoNat,
        _values: &dyn Values,
        units: &str,
    ) -> askama::Result<String> {
        if units == "." {
            Ok(format!("{}", e8s_to_tokens(e8s.into_nat())))
        } else {
            Ok(format!("{} {units}", e8s_to_tokens(e8s.into_nat())))
        }
    }

    pub fn tokens_e8s_guess(
        e8s: impl IntoNat,
        _values: &dyn Values,
        canister: impl ToPrincipal,
    ) -> askama::Result<String> {
        let canister = canister.to_principal();
        if canister == ledger_canister_id() {
            tokens_e8s(e8s, _values, "ICP")
        } else if canister == ckbtc_canister_id(false) {
            tokens_e8s(e8s, _values, "ckBTC")
        } else if canister == ckbtc_canister_id(true) {
            tokens_e8s(e8s, _values, "ckTESTBTC")
        } else {
            tokens_e8s(e8s, _values, "tokens")
        }
    }

    pub fn dur_seconds(seconds: impl ToU64, _values: &dyn Values) -> askama::Result<String> {
        Ok(format_duration_seconds(seconds.to_u64()))
    }

    pub fn dur_nanos(nanoseconds: impl ToU64, _values: &dyn Values) -> askama::Result<String> {
        Ok(format_duration_seconds(
            nanoseconds.to_u64() / 1_000_000_000,
        ))
    }

    pub fn ts_seconds(seconds: impl ToU64, _values: &dyn Values) -> askama::Result<String> {
        Ok(format_timestamp_seconds(seconds.to_u64()))
    }

    pub fn ts_nanos(seconds: impl ToU64, _values: &dyn Values) -> askama::Result<String> {
        Ok(format_timestamp_nanoseconds(seconds.to_u64()))
    }

    pub fn hex(bytes: impl AsRef<[u8]>, _values: &dyn Values) -> askama::Result<String> {
        Ok(hex::encode(bytes))
    }

    /// Indents every line but the first by `width` spaces, leaving blank lines
    /// alone.
    ///
    /// Adapted from askama's `indent` (Apache-2.0/MIT), which stops indenting
    /// altogether once the value reaches 10,000 characters. A proposal summary
    /// may be 30,000 bytes, so a proposer could otherwise pad past that limit to
    /// put a line back at column 0. The name cannot be `indent`: askama resolves
    /// its own filter first, so a local one is never reached.
    pub fn indentf(
        value: impl Display,
        _values: &dyn Values,
        width: usize,
    ) -> askama::Result<String> {
        let value = value.to_string();
        let prefix = " ".repeat(width);
        let mut indented = String::with_capacity(value.len());
        for (idx, line) in value.split_inclusive('\n').enumerate() {
            if idx > 0 && !matches!(line, "\n" | "\r\n") {
                indented.push_str(&prefix);
            }
            indented.push_str(line);
        }
        Ok(indented)
    }

    /// Replaces ascii formatting marks with their Control Pictures.
    pub fn flat(value: impl Display, _values: &dyn Values) -> askama::Result<String> {
        let value = value.to_string();
        let mut flattened = String::with_capacity(value.len());
        let mut chars = value.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '\r' if chars.peek() == Some(&'\n') => {
                    chars.next();
                    flattened.push(SYMBOL_FOR_NEWLINE);
                }
                '\n' => flattened.push(SYMBOL_FOR_NEWLINE),
                '\t' | '\u{b}' | '\u{c}' => flattened
                    .push(super::control_picture(c).expect("an ASCII formatting mark is depicted")),
                _ => flattened.push(c),
            }
        }
        Ok(flattened)
    }

    pub fn cycles_t(cycles: impl IntoNat, _values: &dyn Values) -> askama::Result<String> {
        Ok(format_t_cycles(cycles.into_nat()))
    }

    pub fn cycles_n(cycles: impl IntoNat, _values: &dyn Values) -> askama::Result<String> {
        Ok(format_n_cycles(cycles.into_nat()))
    }

    pub fn candid_payload(
        bytes: impl AsRef<[u8]>,
        _values: &dyn Values,
        canister: impl ToPrincipal,
        function: &str,
    ) -> askama::Result<String> {
        let canister = canister.to_principal();
        let bytes = bytes.as_ref();
        let msg = if bytes.starts_with(b"DIDL") {
            if let Ok(idl) = get_idl_string(
                bytes,
                canister,
                get_default_role(canister).unwrap_or_default(),
                function,
                "args",
            ) {
                idl
            } else {
                hex::encode(bytes)
            }
        } else {
            hex::encode(bytes)
        };
        Ok(msg)
    }

    pub fn num_ufixed(
        n: impl IntoNat,
        _values: &dyn Values,
        digits: i64,
    ) -> askama::Result<BigDecimal> {
        let nat = n.into_nat();
        let bigint = nat.0;
        Ok(BigDecimal::new(bigint.into(), digits))
    }

    pub fn nns_canister_name(
        canister: impl ToPrincipal,
        _values: &dyn Values,
    ) -> askama::Result<String> {
        Ok(canister_id_to_nns_canister_name(
            CanisterId::unchecked_from_principal(canister.to_principal().into()),
        ))
    }

    pub fn bytes(n: impl ToU64, _values: &dyn Values) -> askama::Result<String> {
        let n = n.to_u64();
        Ok(HumanBytes(n).to_string())
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

    impl ToU64 for u32 {
        fn to_u64(&self) -> u64 {
            *self as u64
        }
    }

    impl ToU64 for Nat {
        /// Saturates rather than panicking: these values come off the wire, and a
        /// nonsensically large one should not take down a display command.
        fn to_u64(&self) -> u64 {
            u64::try_from(&self.0).unwrap_or(u64::MAX)
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

    pub trait ToPrincipal {
        fn to_principal(&self) -> Principal;
    }
    impl ToPrincipal for Principal {
        fn to_principal(&self) -> Principal {
            *self
        }
    }
    impl ToPrincipal for CanisterId {
        fn to_principal(&self) -> Principal {
            self.get().0
        }
    }
    impl ToPrincipal for PrincipalId {
        fn to_principal(&self) -> Principal {
            self.0
        }
    }
    impl<T> ToPrincipal for &T
    where
        T: ToPrincipal,
    {
        fn to_principal(&self) -> Principal {
            T::to_principal(self)
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
