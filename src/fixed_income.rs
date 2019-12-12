use std::f64;
use std::fmt;
use std::fmt::{Display,Formatter};
use serde::{Deserialize, Serialize};
use crate::currency::Currency;
use crate::market::Market;
use chrono::NaiveDate;

/// Container for a single cash flow
#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct CashFlow {
    pub amount: f64,
    pub date: NaiveDate,
    pub currency: Currency,
}

impl CashFlow {
    /// Check, whether cash flows could be aggregated
    pub fn aggregatable(&self, cf: &CashFlow) -> bool {
        if self.currency != cf.currency {
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
        } else if self.amount.is_nan() || cf.amount.is_nan() || (self.amount-cf.amount).abs() > tol {
            false
        } else {
            true
        }
    }
}

impl Display for CashFlow {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {:16.4} {}", self.date, self.amount, self.currency)
    }
}

/// Get all future cash flows with respect to a given date
pub fn get_cash_flows_after(cash_flows: &Vec<CashFlow>, date: NaiveDate) -> Vec<CashFlow> {
    let mut new_cash_flows = Vec::new();
    for cf in cash_flows {
        if cf.date>date {
            new_cash_flows.push(cf.clone());
        }
    }
    new_cash_flows
}

pub trait FixedIncome {
    type Error;

    /// Transform product into series of cash flows
    fn rollout_cash_flows(&self, position: f64, market: &Market) -> Result<Vec<CashFlow>, Self::Error>;
}

