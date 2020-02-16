use crate::currency::Currency;
use crate::data_handler::DataError;
use crate::fixed_income::CashFlow;
///! Useful helper functions that do not belong to any other module
use chrono::offset::TimeZone;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, Utc};
use std::str::FromStr;

/// Returns true if some optional String argument is not None and  the value equals a given str reference
pub fn some_equal(opt: &Option<String>, s: &str) -> bool {
    match opt {
        None => false,
        Some(opt_s) => &opt_s == &s,
    }
}

/// Transform optional `usize` to optional `i64`
pub fn usize_to_i64(val: Option<usize>) -> Option<i64> {
    match val {
        Some(v) => Some(v as i64),
        None => None,
    }
}

/// Transform optional `i64` to optional `usize`
pub fn i64_to_usize(val: Option<i64>) -> Option<usize> {
    match val {
        Some(v) => Some(v as usize),
        None => None,
    }
}

/// Transform optional `usize` to optional `i64`
pub fn usize_to_i32(val: Option<usize>) -> Option<i32> {
    match val {
        Some(v) => Some(v as i32),
        None => None,
    }
}

/// Transform optional `i64` to optional `usize`
pub fn i32_to_usize(val: Option<i32>) -> Option<usize> {
    match val {
        Some(v) => Some(v as usize),
        None => None,
    }
}
/// Construct cash flow from raw strings
pub fn raw_to_cash_flow(amount: f64, currency: &str, date: &str) -> Result<CashFlow, DataError> {
    let currency = Currency::from_str(currency).map_err(|e| DataError::NotFound(e.to_string()))?;
    let date = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|e| DataError::NotFound(e.to_string()))?;
    Ok(CashFlow::new(amount, currency, date))
}

/// Convert string to DateTime<Utc>
pub fn to_time(time: &str) -> Result<DateTime<Utc>, DataError> {
    let time =
        DateTime::parse_from_rfc3339(time).map_err(|e| DataError::NotFound(e.to_string()))?;
    let time: DateTime<Utc> = DateTime::from(time);
    Ok(time)
}

/// Given a date and time construct a UTC DateTime, assuming that
/// the date belongs to local time zone
pub fn make_time(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> Option<DateTime<Utc>> {
    let time: NaiveDateTime = NaiveDate::from_ymd(year, month, day).and_hms(hour, minute, second);
    let time = Local.from_local_datetime(&time).single();
    match time {
        Some(time) => Some(DateTime::from(time)),
        None => None,
    }
}
