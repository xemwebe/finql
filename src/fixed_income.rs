use crate::currency::Currency;
use crate::market::Market;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::f64;
use std::fmt;
use std::fmt::{Display, Formatter};

/// Container for an amount of money in some currency
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq)]
pub struct Amount {
    pub amount: f64,
    pub currency: Currency,
}

impl Display for Amount {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:16.4} {}", self.amount, self.currency)
    }
}

/// Container for a single cash flow
#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct CashFlow {
    pub amount: Amount,
    pub date: NaiveDate,
}

impl CashFlow {
    /// Construct new cash flow
    pub fn new(amount: f64, currency: Currency, date: NaiveDate) -> CashFlow {
        CashFlow {
            amount: Amount { amount, currency },
            date,
        }
    }
    /// Check, whether cash flows could be aggregated
    pub fn aggregatable(&self, cf: &CashFlow) -> bool {
        if self.amount.currency != cf.amount.currency {
            false
        } else if self.date != cf.date {
            false
        } else {
            true
        }
    }

    /// Compare to cash flows for equality within a given absolute tolerance
    pub fn fuzzy_cash_flows_cmp_eq(&self, cf: &CashFlow, tol: f64) -> bool {
        if !self.aggregatable(cf) {
            false
        } else if self.amount.amount.is_nan()
            || cf.amount.amount.is_nan()
            || (self.amount.amount - cf.amount.amount).abs() > tol
        {
            false
        } else {
            true
        }
    }
}

impl Display for CashFlow {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.date, self.amount)
    }
}

/// Get all future cash flows with respect to a given date
pub fn get_cash_flows_after(cash_flows: &Vec<CashFlow>, date: NaiveDate) -> Vec<CashFlow> {
    let mut new_cash_flows = Vec::new();
    for cf in cash_flows {
        if cf.date > date {
            new_cash_flows.push(cf.clone());
        }
    }
    new_cash_flows
}

pub trait FixedIncome {
    type Error;

    /// Transform product into series of cash flows
    fn rollout_cash_flows(
        &self,
        position: f64,
        market: &Market,
    ) -> Result<Vec<CashFlow>, Self::Error>;

    /// Calculate accrued interest for current coupon period
    fn accrued_interest(&self, today: NaiveDate) -> Result<f64, Self::Error>;
}
