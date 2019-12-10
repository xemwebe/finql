use serde::{Deserialize, Serialize};
use chrono::{NaiveDate,Datelike};
use crate::calendar::{DayAdjust};
use crate::day_count_conv::DayCountConv;
use crate::time_period::TimePeriod;
use crate::currency::Currency;

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
    fn year_fraction(&self, start: NaiveDate, end: NaiveDate) -> f64 { 
        self.day_count_convention.year_fraction(start, end).unwrap()
    }
}

use std::f64;
use std::fmt;
use std::fmt::{Display,Formatter};

/// Container for a single cash flow
#[derive(Deserialize, Serialize, Debug)]
pub struct CashFlow {
    amount: f64,
    date: NaiveDate,
    currency: Currency,
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
        write!(f, "{} {} {}", self.date, self.amount, self.currency)
    }
}

/// Convert bond in stream of cash flows
pub fn rollout_cash_flows(bond: Bond, position: f64) -> Vec<CashFlow> {
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
    let year_fraction = bond.coupon.year_fraction(start_date, end_date);
    let amount = position * (bond.denomination as f64) * bond.coupon.rate/100. * year_fraction;
    let cf = CashFlow{ amount:amount, currency: bond.currency.clone(), date: end_date.clone() };
    cfs.push(cf);
    let maturity = bond.maturity;
    while end_date < maturity {
        let start_date = end_date;
        end_date = bond.coupon.period.add_to(start_date, None);
        let year_fraction = bond.coupon.year_fraction(start_date, end_date);
        let amount = position * (bond.denomination as f64) * bond.coupon.rate/100. * year_fraction;
        let cf = CashFlow{ amount:amount, currency: bond.currency.clone(), date: end_date.clone() };
        cfs.push(cf);
    }
    cfs
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
        let cash_flows = rollout_cash_flows(bond, 1.);
        assert_eq!(cash_flows.len(), 4);
        let curr = Currency::from_str("EUR").unwrap();
        let reference_cash_flows = vec![
            CashFlow{amount: 0.05*1000.*183./365., currency: curr.clone(), date: NaiveDate::from_ymd(2020,4,1) },
            CashFlow{amount: 0.05*1000.*183./365., currency: curr.clone(), date: NaiveDate::from_ymd(2020,10,1) },
            CashFlow{amount: 0.05*1000.*182./365., currency: curr.clone(), date: NaiveDate::from_ymd(2021,4,1) },
            CashFlow{amount: 0.05*1000.*183./365., currency: curr, date: NaiveDate::from_ymd(2021,10,1) },
        ];
        let tol = 1e-11;
        assert!(reference_cash_flows[0].fuzzy_cash_flows_cmp_eq(&cash_flows[0], tol));
        assert!(reference_cash_flows[1].fuzzy_cash_flows_cmp_eq(&cash_flows[1], tol));
        assert!(reference_cash_flows[2].fuzzy_cash_flows_cmp_eq(&cash_flows[2], tol));
        assert!(reference_cash_flows[3].fuzzy_cash_flows_cmp_eq(&cash_flows[3], tol));
    }
}