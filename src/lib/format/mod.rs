use chrono::{DateTime, TimeZone, Utc};
use itertools::Itertools;

pub mod ckbtc;
pub mod gtc;
pub mod icp_ledger;
pub mod icrc1;
pub mod nns_governance;
pub mod registry;

pub fn format_datetime(datetime: DateTime<Utc>) -> String {
    format!("{} UTC", datetime.format("%b %d %Y %X"))
}

pub fn format_timestamp_seconds(seconds: u64) -> String {
    format_datetime(Utc.timestamp_opt(seconds.try_into().unwrap(), 0).unwrap())
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

#[test]
fn magic_durations() {
    assert_eq!(format_duration_seconds(15_778_800), "6 months");
    assert_eq!(format_duration_seconds(252_460_800), "8 years");
}
