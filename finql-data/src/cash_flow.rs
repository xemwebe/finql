use std::{fmt,fmt::{Display, Formatter}};
use std::collections::BTreeMap;
use std::ops::Neg;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, NaiveDate};

use crate::currency::{Currency, CurrencyConverter, CurrencyError};

/// Container for an amount of money in some currency
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq)]
pub struct CashAmount {
    pub amount: f64,
    pub currency: Currency,
}

pub fn round2digits(x: f64, digits: i32) -> f64 {
    (x * 10.0_f64.powi(digits)).round() / 10.0_f64.powi(digits)
}

impl CashAmount {
    pub async fn add(
        &mut self,
        cash_amount: CashAmount,
        time: DateTime<Utc>,
        currency_converter: &mut dyn CurrencyConverter,
        with_rounding: bool,
    ) -> Result<&mut Self, CurrencyError> {
        if self.currency == cash_amount.currency {
            self.amount += cash_amount.amount;
            Ok(self)
        } else {
            let fx_rate = currency_converter.fx_rate(cash_amount.currency, self.currency, time).await?;
            self.amount += fx_rate * cash_amount.amount;
            if with_rounding {
                let digits = self.currency.rounding_digits();
                self.amount = round2digits(self.amount, digits);
            }
            Ok(self)
        }
    }

    pub async fn add_opt(
        &mut self,
        cash_amount: Option<CashAmount>,
        time: DateTime<Utc>,
        currency_converter: &mut dyn CurrencyConverter,
        with_rounding: bool,
    ) -> Result<&mut Self, CurrencyError> {
        match cash_amount {
            None => Ok(self),
            Some(cash_amount) => self.add(cash_amount, time, currency_converter, with_rounding).await,
        }
    }

    pub async fn sub(
        &mut self,
        cash_amount: CashAmount,
        time: DateTime<Utc>,
        currency_converter: &mut dyn CurrencyConverter,
        with_rounding: bool,
    ) -> Result<&mut Self, CurrencyError> {
        if self.currency == cash_amount.currency {
            self.amount -= cash_amount.amount;
            Ok(self)
        } else {
            let fx_rate = currency_converter.fx_rate(cash_amount.currency, self.currency, time).await?;
            self.amount -= fx_rate * cash_amount.amount;
            if with_rounding {
                let digits = self.currency.rounding_digits();
                self.amount = round2digits(self.amount, digits);
            }
            Ok(self)
        }
    }

    pub async fn sub_opt(
        &mut self,
        cash_amount: Option<CashAmount>,
        time: DateTime<Utc>,
        currency_converter: &mut dyn CurrencyConverter,
        with_rounding: bool,
    ) -> Result<&mut Self, CurrencyError> {
        match cash_amount {
            None => Ok(self),
            Some(cash_amount) => self.sub(cash_amount, time, currency_converter, with_rounding).await,
        }
    }

    /// Round a cash amount to that number of decimals
    pub fn round(&self, digits: i32) -> CashAmount {
        CashAmount {
            amount: round2digits(self.amount, digits),
            currency: self.currency,
        }
    }

    /// Round Cash amount according to rounding conventions
    /// Lookup currency in rounding_conventions. If found, use the number of digits found for
    /// rounding to that number of decimals, otherwise round to two decimals.
    pub fn round_by_convention(&self, rounding_conventions: &BTreeMap<String, i32>) -> CashAmount {
        match rounding_conventions.get_key_value(&self.currency.to_string()) {
            Some((_, digits)) => self.round(*digits),
            None => self.round(2),
        }
    }
}

impl Display for CashAmount {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:16.4} {}", self.amount, self.currency)
    }
}

impl Neg for CashAmount {
    type Output = CashAmount;

    fn neg(self) -> Self::Output {
        CashAmount {
            amount: -self.amount,
            currency: self.currency,
        }
    }
}

/// Container for a single cash flow
#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct CashFlow {
    pub amount: CashAmount,
    pub date: NaiveDate,
}

impl CashFlow {
    /// Construct new cash flow
    pub fn new(amount: f64, currency: Currency, date: NaiveDate) -> CashFlow {
        CashFlow {
            amount: CashAmount { amount, currency },
            date,
        }
    }
    /// Check, whether cash flows could be aggregated
    pub fn aggregatable(&self, cf: &CashFlow) -> bool {
        self.amount.currency == cf.amount.currency && self.date == cf.date
    }

    /// Compare to cash flows for equality within a given absolute tolerance
    pub fn fuzzy_cash_flows_cmp_eq(&self, cf: &CashFlow, tol: f64) -> bool {
        self.aggregatable(cf) 
            && !self.amount.amount.is_nan()
            && !cf.amount.amount.is_nan()
            && (self.amount.amount - cf.amount.amount).abs() <= tol
    }
}

impl Display for CashFlow {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.date, self.amount)
    }
}

impl Neg for CashFlow {
    type Output = CashFlow;

    fn neg(self) -> Self::Output {
        CashFlow {
            amount: -self.amount,
            date: self.date,
        }
    }
}
