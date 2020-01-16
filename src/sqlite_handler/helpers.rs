use super::DataError;
use crate::currency::Currency;
use crate::fixed_income::CashFlow;
use chrono::{DateTime, NaiveDate, Utc};
use std::str::FromStr;

/// Transform optional `usize` to optional `i64`
pub fn usize_to_int(val: Option<usize>) -> Option<i64> {
    match val {
        Some(v) => Some(v as i64),
        None => None,
    }
}

/// Transform optional `i64` to optional `usize`
pub fn int_to_usize(val: Option<i64>) -> Option<usize> {
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
