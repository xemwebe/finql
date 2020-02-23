use crate::currency::Currency;
use crate::data_handler::{DataError, QuoteHandler};
use crate::day_count_conv::DayCountConv;
use crate::fx_rates::get_fx_rate;
use crate::market::Market;
use crate::rates::{Compounding, DiscountError, Discounter, FlatRate};
use argmin::prelude::*;
use argmin::solver::brent::Brent;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::f64;
use std::fmt;
use std::fmt::{Display, Formatter};

/// Container for an amount of money in some currency
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq)]
pub struct CashAmount {
    pub amount: f64,
    pub currency: Currency,
}

impl CashAmount {
    pub fn add(
        &mut self,
        cash_amount: CashAmount,
        time: DateTime<Utc>,
        quotes: &mut dyn QuoteHandler,
    ) -> Result<&mut Self, DataError> {
        if self.currency == cash_amount.currency {
            self.amount += cash_amount.amount;
            Ok(self)
        } else {
            let fx_rate = get_fx_rate(cash_amount.currency, self.currency, time, quotes)?;
            self.amount += fx_rate * cash_amount.amount;
            Ok(self)
        }
    }

    pub fn add_opt(
        &mut self,
        cash_amount: Option<CashAmount>,
        time: DateTime<Utc>,
        quotes: &mut dyn QuoteHandler,
    ) -> Result<&mut Self, DataError> {
        match cash_amount {
            None => Ok(self),
            Some(cash_amount) => self.add(cash_amount, time, quotes),
        }
    }

    pub fn sub(
        &mut self,
        cash_amount: CashAmount,
        time: DateTime<Utc>,
        quotes: &mut dyn QuoteHandler,
    ) -> Result<&mut Self, DataError> {
        if self.currency == cash_amount.currency {
            self.amount -= cash_amount.amount;
            Ok(self)
        } else {
            let fx_rate = get_fx_rate(cash_amount.currency, self.currency, time, quotes)?;
            self.amount -= fx_rate * cash_amount.amount;
            Ok(self)
        }
    }

    pub fn sub_opt(
        &mut self,
        cash_amount: Option<CashAmount>,
        time: DateTime<Utc>,
        quotes: &mut dyn QuoteHandler,
    ) -> Result<&mut Self, DataError> {
        match cash_amount {
            None => Ok(self),
            Some(cash_amount) => self.sub(cash_amount, time, quotes),
        }
    }
}

impl Display for CashAmount {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:16.4} {}", self.amount, self.currency)
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
    type Error: std::convert::From<DiscountError>;

    /// Transform product into series of cash flows
    fn rollout_cash_flows(
        &self,
        position: f64,
        market: &Market,
    ) -> Result<Vec<CashFlow>, Self::Error>;

    /// Calculate accrued interest for current coupon period
    fn accrued_interest(&self, today: NaiveDate) -> Result<f64, Self::Error>;

    /// Calculate the yield to maturity (YTM) given a purchase price and date
    fn calculate_ytm(
        &self,
        purchase_cash_flow: &CashFlow,
        market: &Market,
    ) -> Result<f64, Self::Error> {
        let cash_flows = self.rollout_cash_flows(1., market)?;
        let value = calculate_cash_flows_ytm(&cash_flows, &purchase_cash_flow)?;
        Ok(value)
    }
}

/// Calculate the internal rate of return of a stream of cash flows
/// The calculation assumes, that the notional payments and beginning and end are
/// included and calculates that annual rate, that gives total aggregate zero value
/// of all cash flows provided as `cash_flows`, if discounted to the payment date
/// of the first cash flow. It is assumed that all cash flow are in the same currency,
/// otherwise a `DiscountError` will be returned.
pub fn calculate_cash_flows_ytm(
    cash_flows: &Vec<CashFlow>,
    init_cash_flow: &CashFlow,
) -> Result<f64, DiscountError> {
    let rate = FlatRate::new(
        0.05,
        DayCountConv::Act365,
        Compounding::Annual,
        init_cash_flow.amount.currency,
    );
    let init_param = 0.5;
    let solver = Brent::new(0., 0.5, 1e-11);
    let func = FlatRateDiscounter {
        init_cash_flow: init_cash_flow,
        cash_flows: cash_flows,
        rate,
    };
    let res = Executor::new(func, solver, init_param).max_iters(100).run();
    match res {
        Ok(val) => Ok(val.state.get_param()),
        Err(_) => Err(DiscountError),
    }
}

/// Calculate discounted value for given flat rate
/// Since `argmin` requires `Serialize` and `Deserialize`,
/// we can't use reference here but must clone all data to this struct
#[derive(Clone)]
struct FlatRateDiscounter<'a> {
    init_cash_flow: &'a CashFlow,
    cash_flows: &'a Vec<CashFlow>,
    rate: FlatRate,
}

impl<'a> ArgminOp for FlatRateDiscounter<'a> {
    // one dimensional problem, no vector needed
    type Param = f64;
    type Output = f64;
    type Hessian = ();
    type Jacobian = ();

    fn apply(&self, p: &Self::Param) -> Result<Self::Output, Error> {
        let mut discount_rate = self.rate.clone();
        discount_rate.rate = *p;
        let mut sum = self.init_cash_flow.amount.amount;
        let today = self.init_cash_flow.date;
        for cf in self.cash_flows.clone() {
            if cf.date > today {
                sum += discount_rate.discount_cash_flow(&cf, today)?.amount;
            }
        }
        Ok(sum)
    }
}

/// Dummy implementation of Serialize
impl<'a> Serialize for FlatRateDiscounter<'a> {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Err(serde::ser::Error::custom(format!(
            "serialization is disabled"
        )))
    }
}

/// Dummy implementation fo Deserialize
impl<'de> Deserialize<'de> for FlatRateDiscounter<'de> {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Err(serde::de::Error::custom(format!(
            "deserialization is disabled"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn yield_to_maturity() {
        let tol = 1e-11;
        let curr = Currency::from_str("EUR").unwrap();
        let cash_flows = vec![CashFlow::new(1050., curr, NaiveDate::from_ymd(2021, 10, 1))];
        let init_cash_flow = CashFlow::new(-1000., curr, NaiveDate::from_ymd(2020, 10, 1));

        let ytm = calculate_cash_flows_ytm(&cash_flows, &init_cash_flow).unwrap();
        assert_fuzzy_eq!(ytm, 0.05, tol);
    }
}
