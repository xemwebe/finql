/// Definition of bonds and similar fixed income products
/// and functionality to rollout cashflows and calculate basic
/// valuation figures

use serde::{Deserialize, Serialize};
use chrono::{NaiveDate,Datelike};
use crate::day_adjust::{DayAdjust};
use crate::day_count_conv::{DayCountConv, DayCountConvError};
use crate::time_period::TimePeriod;
use crate::currency::Currency;
use crate::market::{Market, MarketError};
use std::error::Error;

/// Error related to bonds
#[derive(Debug)]
pub enum BondError {
        MissingCalendar,
        DayCountError(DayCountConvError)
}

impl fmt::Display for BondError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BondError::MissingCalendar => write!(f, "unknown calendar"),
            BondError::DayCountError(_) => write!(f, "invalid day count convention in this context"),
       }
    }
}

impl Error for BondError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            BondError::DayCountError(err) => Some(err),
            _ => None
        }
    }
}

impl From<DayCountConvError> for BondError {
    fn from(error: DayCountConvError) -> Self {
        BondError::DayCountError(error)
    }
}

impl From<MarketError> for BondError {
    fn from(_: MarketError) -> Self {
        BondError::MissingCalendar
    }
}

/// Container for bonds and similar fixed income assets
#[derive(Deserialize, Serialize, Debug)]
pub struct Bond {
    /// International security identification number
    isin: Option<String>,
    /// Local or national security identifier
    security_id: Option<String>,
    /// URL to bond prospectus, if available
    prospect_url: Option<String>,
    /// Issuer of the bond
    issuer: Option<Issuer>,
    bond_type: String,
    currency: Currency,
    coupon : Coupon,
    business_day_rule: DayAdjust,
    calendar: String,
    issue_date: NaiveDate,
    maturity: NaiveDate,
    /// Smallest purchasable unit
    denomination: u32,
    volume: Option<f64>
}

/// Information regarding the issuer of an asset
/// This is required for determination of some asset's credit worthiness.
#[derive(Deserialize, Serialize, Debug)]
struct Issuer {
    /// Minimal obligatory information is the name of the issuer
    name: String,
    address: Option<IssuerAddress>,
}

/// Address of an issuer, e.g. city and country of headquarter
#[derive(Deserialize, Serialize, Debug)]
struct IssuerAddress {
    city: String,
    country: String,
}

use super::coupon_date::CouponDate;

/// Coupon specification of fixed income instruments
#[derive(Deserialize, Serialize, Debug)]
struct Coupon {
    coupon_type: String,
    rate: f64,
    /// (Unadjusted) first coupon end date used as a basis for cash flow rollout
    coupon_date: CouponDate,
    period: TimePeriod,
    day_count_convention: DayCountConv,
}

impl Coupon {
    fn coupon_day(&self) -> u32 { self.coupon_date.day() }
    fn coupon_month(&self) -> u32 { self.coupon_date.month() }
    fn year_fraction(&self, start: NaiveDate, end: NaiveDate, 
        roll_date: NaiveDate) -> Result<f64, DayCountConvError> { 
        self.day_count_convention.year_fraction(start, end, Some(roll_date), Some(self.period))
    }
}

use std::f64;
use std::fmt;
use std::fmt::{Display,Formatter};

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

/// Convert bond in stream of cash flows
pub fn rollout_cash_flows(bond: Bond, position: f64, market: &Market) -> Result<Vec<CashFlow>, BondError> {
    let mut cfs = Vec::new();
    let start_date = bond.issue_date;
    let mut end_date = if bond.coupon.coupon_month()<=start_date.month() {
        NaiveDate::from_ymd(start_date.year()+1,
            bond.coupon.coupon_month(),
            bond.coupon.coupon_day())
    } else {
        NaiveDate::from_ymd(start_date.year(),
        bond.coupon.coupon_month(),
        bond.coupon.coupon_day())
    };
    let year_fraction = bond.coupon.year_fraction(start_date, end_date, end_date)?;
    let amount = position * (bond.denomination as f64) * bond.coupon.rate/100. * year_fraction;
    let cal = market.get_calendar(&bond.calendar)?;
    let pay_date = bond.business_day_rule.adjust_date(end_date, cal);
    let cf = CashFlow{ amount:amount, currency: bond.currency.clone(), date: pay_date };
    cfs.push(cf);
    let maturity = bond.maturity;
    while end_date < maturity {
        let start_date = end_date;
        end_date = bond.coupon.period.add_to(start_date, None);
        let year_fraction = bond.coupon.year_fraction(start_date, end_date, start_date)?;
        let amount = position * (bond.denomination as f64) * bond.coupon.rate/100. * year_fraction;
        let pay_date = bond.business_day_rule.adjust_date(end_date, cal);
        let cf = CashFlow{ amount:amount, currency: bond.currency.clone(), date: pay_date };
        cfs.push(cf);
    }
    // final nominal payment
    let cf = CashFlow{ 
        amount: position*(bond.denomination as f64),
        currency: bond.currency.clone(),
        date: bond.business_day_rule.adjust_date(maturity, cal)
    };
    cfs.push(cf);

    Ok(cfs)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn cash_flow_rollout_unadjusted () {
        let data = r#"{
            "bond_type": "bond",
            "currency": "EUR",
            "coupon" : {
                "coupon_type": "fixed",
                "rate": 5,
                "coupon_date": "01.04",
                "period": "6M",
                "day_count_convention": "act/365"
            },
            "business_day_rule": "none",
            "calendar": "TARGET",
            "issue_date": "2019-10-01",
            "maturity": "2021-10-01",
            "denomination": 1000
        }"#;
        let bond: Bond = serde_json::from_str(&data).unwrap(); 
        let market = Market::new();
        let cash_flows = rollout_cash_flows(bond, 1., &market).unwrap();
        assert_eq!(cash_flows.len(), 5);
        let curr = Currency::from_str("EUR").unwrap();
        let reference_cash_flows = vec![
            CashFlow{amount: 0.05*1000.*183./365., currency: curr.clone(), date: NaiveDate::from_ymd(2020,4,1) },
            CashFlow{amount: 0.05*1000.*183./365., currency: curr.clone(), date: NaiveDate::from_ymd(2020,10,1) },
            CashFlow{amount: 0.05*1000.*182./365., currency: curr.clone(), date: NaiveDate::from_ymd(2021,4,1) },
            CashFlow{amount: 0.05*1000.*183./365., currency: curr, date: NaiveDate::from_ymd(2021,10,1) },
            CashFlow{amount: 1000., currency: curr, date: NaiveDate::from_ymd(2021,10,1) },
        ];
        let tol = 1e-11;
        assert!(reference_cash_flows[0].fuzzy_cash_flows_cmp_eq(&cash_flows[0], tol));
        assert!(reference_cash_flows[1].fuzzy_cash_flows_cmp_eq(&cash_flows[1], tol));
        assert!(reference_cash_flows[2].fuzzy_cash_flows_cmp_eq(&cash_flows[2], tol));
        assert!(reference_cash_flows[3].fuzzy_cash_flows_cmp_eq(&cash_flows[3], tol));
        assert!(reference_cash_flows[4].fuzzy_cash_flows_cmp_eq(&cash_flows[4], tol));
    }

    #[test]
    fn cash_flow_rollout_adjusted () {
        let data = r#"{
            "bond_type": "bond",
            "currency": "EUR",
            "coupon" : {
                "coupon_type": "fixed",
                "rate": 5,
                "coupon_date": "01.04",
                "period": "6M",
                "day_count_convention": "icma"
            },
            "business_day_rule": "modified",
            "calendar": "TARGET",
            "issue_date": "2020-10-01",
            "maturity": "2022-10-01",
            "denomination": 1000
        }"#;
        let bond: Bond = serde_json::from_str(&data).unwrap(); 
        let market = Market::new();
        let cash_flows = rollout_cash_flows(bond, 1., &market).unwrap();
        assert_eq!(cash_flows.len(), 5);
        let curr = Currency::from_str("EUR").unwrap();
        let reference_cash_flows = vec![
            CashFlow{amount: 0.05*1000./2., currency: curr.clone(), date: NaiveDate::from_ymd(2021,4,1) },
            CashFlow{amount: 0.05*1000./2., currency: curr.clone(), date: NaiveDate::from_ymd(2021,10,1) },
            CashFlow{amount: 0.05*1000./2., currency: curr.clone(), date: NaiveDate::from_ymd(2022,4,1) },
            CashFlow{amount: 0.05*1000./2., currency: curr.clone(), date: NaiveDate::from_ymd(2022,10,3) },
            CashFlow{amount: 1000., currency: curr, date: NaiveDate::from_ymd(2022,10,3) },
        ];
        let tol = 1e-11;
        assert!(reference_cash_flows[0].fuzzy_cash_flows_cmp_eq(&cash_flows[0], tol));
        assert!(reference_cash_flows[1].fuzzy_cash_flows_cmp_eq(&cash_flows[1], tol));
        assert!(reference_cash_flows[2].fuzzy_cash_flows_cmp_eq(&cash_flows[2], tol));
        assert!(reference_cash_flows[3].fuzzy_cash_flows_cmp_eq(&cash_flows[3], tol));
        assert!(reference_cash_flows[4].fuzzy_cash_flows_cmp_eq(&cash_flows[4], tol));
    }
}