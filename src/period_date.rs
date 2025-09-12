use crate::datatypes::Transaction;
use core::default::Default;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt::Display;
use thiserror::Error;
use time::{Date, Month, OffsetDateTime, UtcOffset};

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
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub enum PeriodDate {
    Inception,
    #[default]
    Today,
    FirstOfMonth,
    FirstOfYear,
    FixedDate(Date),
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

impl PeriodDate {
    pub fn new(date_type: &str, date: Option<Date>) -> Result<PeriodDate, PeriodDateError> {
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

    pub fn date(&self, inception: Option<Date>) -> Result<Date, PeriodDateError> {
        match self {
            PeriodDate::Today => Ok(OffsetDateTime::now_utc().date()),
            PeriodDate::FirstOfMonth => {
                let today = OffsetDateTime::now_utc().date();
                Date::from_calendar_date(today.year(), today.month(), 1)
                    .map_err(|_| PeriodDateError::InvalidDate)
            }
            PeriodDate::FirstOfYear => {
                let today = OffsetDateTime::now_utc().date();
                Date::from_calendar_date(today.year(), Month::January, 1)
                    .map_err(|_| PeriodDateError::InvalidDate)
            }
            PeriodDate::FixedDate(date) => Ok(*date),
            PeriodDate::Inception => inception.ok_or(PeriodDateError::MissingInceptionDate),
        }
    }

    pub fn date_from_trades(&self, trades: &[Transaction]) -> Result<Date, PeriodDateError> {
        let inception = trades.iter().map(|t| t.cash_flow.date).min();
        self.date(inception)
    }
}
