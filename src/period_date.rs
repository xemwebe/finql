use crate::datatypes::Transaction;
use chrono::{Datelike, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fmt::Display;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PeriodDateError {
    #[error("Fixed date is chosen but no date is given")]
    MissingFixedDate,
    #[error("Unknown period date type")]
    UnknownPeriodDateType,
    #[error("Cannot deduce inception date")]
    MissingInceptionDate,
    #[error("Try to create invalid date")]
    InvalidDate,
}

/// Period start or end date
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PeriodDate {
    Inception,
    Today,
    FirstOfMonth,
    FirstOfYear,
    FixedDate(NaiveDate),
}

impl Display for PeriodDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PeriodDate::Inception => write!(f, "Inception"),
            PeriodDate::Today => write!(f, "Today"),
            PeriodDate::FirstOfMonth => write!(f, "FirstOfMonth"),
            PeriodDate::FirstOfYear => write!(f, "FirstOfYear"),
            PeriodDate::FixedDate(_) => write!(f, "FixedDate"),
        }
    }
}

impl Default for PeriodDate {
    fn default() -> PeriodDate {
        PeriodDate::Today
    }
}

impl PeriodDate {
    pub fn new(date_type: &str, date: Option<NaiveDate>) -> Result<PeriodDate, PeriodDateError> {
        match date_type {
            "Inception" => Ok(PeriodDate::Inception),
            "Today" => Ok(PeriodDate::Today),
            "FirstOfMonth" => Ok(PeriodDate::FirstOfMonth),
            "FirstOfYear" => Ok(PeriodDate::FirstOfYear),
            "FixedDate" => {
                if let Some(date) = date {
                    Ok(PeriodDate::FixedDate(date))
                } else {
                    Err(PeriodDateError::MissingFixedDate)
                }
            }
            _ => Err(PeriodDateError::UnknownPeriodDateType),
        }
    }

    pub fn date(&self, inception: Option<NaiveDate>) -> Result<NaiveDate, PeriodDateError> {
        match self {
            PeriodDate::Today => Ok(Local::today().naive_local()),
            PeriodDate::FirstOfMonth => {
                let today = Local::today().naive_local();
                NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
                    .ok_or(PeriodDateError::InvalidDate)
            }
            PeriodDate::FirstOfYear => {
                let today = Local::today().naive_local();
                NaiveDate::from_ymd_opt(today.year(), 1, 1).ok_or(PeriodDateError::InvalidDate)
            }
            PeriodDate::FixedDate(date) => Ok(*date),
            PeriodDate::Inception => inception.ok_or(PeriodDateError::MissingInceptionDate),
        }
    }

    pub fn date_from_trades(&self, trades: &[Transaction]) -> Result<NaiveDate, PeriodDateError> {
        let inception = trades.iter().map(|t| t.cash_flow.date).min();
        self.date(inception)
    }
}
