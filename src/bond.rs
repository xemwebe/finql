//! Definition of bonds and similar fixed income products
//! and functionality to rollout cashflows and calculate basic
//! valuation figures

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::datatypes::cash_flow::CashFlow;
use crate::datatypes::currency::Currency;

use crate::day_adjust::{AdjustDateError, DayAdjust};
use crate::day_count_conv::{DayCountConv, DayCountConvError};
use crate::fixed_income::FixedIncome;
use crate::rates::DiscountError;
use crate::time_period::TimePeriod;
use cal_calc::{CalendarError, CalendarProvider};

/// Error related to bonds
#[derive(Error, Debug)]
pub enum BondError {
    #[error("discounting cash flows failed")]
    DiscountingFailure(#[from] DiscountError),
    #[error("unknown or invalid calendar")]
    MissingCalendar(#[from] CalendarError),
    #[error("invalid day count convention in this context")]
    DayCountError(#[from] DayCountConvError),
    #[error("failed to adjust day to business day")]
    DayAdjustError(#[from] AdjustDateError),
    #[error("failed to apply time period")]
    TimePeriodError(#[from] crate::time_period::TimePeriodError),
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
    coupon: Coupon,
    business_day_rule: DayAdjust,
    calendar: String,
    issue_date: NaiveDate,
    maturity: NaiveDate,
    /// Smallest purchasable unit
    pub denomination: u32,
    volume: Option<f64>,
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
    fn coupon_day(&self) -> u32 {
        self.coupon_date.day()
    }
    fn coupon_month(&self) -> u32 {
        self.coupon_date.month()
    }
    fn year_fraction(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        roll_date: NaiveDate,
    ) -> Result<f64, DayCountConvError> {
        self.day_count_convention
            .year_fraction(start, end, Some(roll_date), Some(self.period))
    }
}

impl Bond {
    /// Calculate first coupon period end date
    fn first_coupon_end(&self, start_date: NaiveDate) -> NaiveDate {
        if self.coupon.coupon_month() <= start_date.month() {
            NaiveDate::from_ymd(
                start_date.year() + 1,
                self.coupon.coupon_month(),
                self.coupon.coupon_day(),
            )
        } else {
            NaiveDate::from_ymd(
                start_date.year(),
                self.coupon.coupon_month(),
                self.coupon.coupon_day(),
            )
        }
    }
}

impl FixedIncome for Bond {
    type Error = BondError;

    /// Convert bond in stream of cash flows
    fn rollout_cash_flows(
        &self,
        position: f64,
        calendar_provider: &dyn CalendarProvider,
    ) -> Result<Vec<CashFlow>, BondError> {
        let mut cfs = Vec::new();
        let start_date = self.issue_date;
        let mut end_date = self.first_coupon_end(start_date);
        let year_fraction = self.coupon.year_fraction(start_date, end_date, end_date)?;
        let amount =
            position * (self.denomination as f64) * self.coupon.rate / 100. * year_fraction;
        let cal = calendar_provider.get_calendar(&self.calendar)?;
        let pay_date = self.business_day_rule.adjust_date(end_date, cal)?;
        let cf = CashFlow::new(amount, self.currency, pay_date);
        cfs.push(cf);
        let maturity = self.maturity;
        while end_date < maturity {
            let start_date = end_date;
            end_date = self.coupon.period.add_to(start_date, None)?;
            let year_fraction = self
                .coupon
                .year_fraction(start_date, end_date, start_date)?;
            let amount =
                position * (self.denomination as f64) * self.coupon.rate / 100. * year_fraction;
            let pay_date = self.business_day_rule.adjust_date(end_date, cal)?;
            let cf = CashFlow::new(amount, self.currency, pay_date);
            cfs.push(cf);
        }
        // final nominal payment
        let cf = CashFlow::new(
            position * (self.denomination as f64),
            self.currency,
            self.business_day_rule.adjust_date(maturity, cal)?,
        );
        cfs.push(cf);

        Ok(cfs)
    }

    fn accrued_interest(&self, today: NaiveDate) -> Result<f64, BondError> {
        let mut start_date = self.issue_date;
        if today < start_date {
            return Ok(0.);
        }
        let mut end_date = self.first_coupon_end(start_date);
        while today > end_date && end_date < self.maturity {
            start_date = end_date;
            end_date = self.coupon.period.add_to(start_date, None)?;
        }
        if end_date >= self.maturity {
            return Ok(0.);
        }
        let year_fraction = self
            .coupon
            .year_fraction(start_date, end_date, start_date)?;
        let amount = (self.denomination as f64) * self.coupon.rate / 100. * year_fraction;
        let fraction = today.signed_duration_since(start_date).num_days() as f64
            / end_date.signed_duration_since(start_date).num_days() as f64;

        Ok(amount * fraction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market::generate_calendars;
    use cal_calc::SimpleCalendar;
    use std::str::FromStr;

    #[test]
    fn cash_flow_rollout_unadjusted() {
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
        let calendar = SimpleCalendar::default();
        let cash_flows = bond.rollout_cash_flows(1., &calendar).unwrap();
        assert_eq!(cash_flows.len(), 5);
        let curr = Currency::from_str("EUR").unwrap();
        let reference_cash_flows = vec![
            CashFlow::new(
                0.05 * 1000. * 183. / 365.,
                curr,
                NaiveDate::from_ymd_opt(2020, 4, 1).unwrap(),
            ),
            CashFlow::new(
                0.05 * 1000. * 183. / 365.,
                curr,
                NaiveDate::from_ymd_opt(2020, 10, 1).unwrap(),
            ),
            CashFlow::new(
                0.05 * 1000. * 182. / 365.,
                curr,
                NaiveDate::from_ymd_opt(2021, 4, 1).unwrap(),
            ),
            CashFlow::new(
                0.05 * 1000. * 183. / 365.,
                curr,
                NaiveDate::from_ymd_opt(2021, 10, 1).unwrap(),
            ),
            CashFlow::new(1000., curr, NaiveDate::from_ymd_opt(2021, 10, 1).unwrap()),
        ];
        let tol = 1e-11;
        assert!(reference_cash_flows[0].fuzzy_cash_flows_cmp_eq(&cash_flows[0], tol));
        assert!(reference_cash_flows[1].fuzzy_cash_flows_cmp_eq(&cash_flows[1], tol));
        assert!(reference_cash_flows[2].fuzzy_cash_flows_cmp_eq(&cash_flows[2], tol));
        assert!(reference_cash_flows[3].fuzzy_cash_flows_cmp_eq(&cash_flows[3], tol));
        assert!(reference_cash_flows[4].fuzzy_cash_flows_cmp_eq(&cash_flows[4], tol));
    }

    #[test]
    fn cash_flow_rollout_adjusted() {
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
        let sample_calendars = generate_calendars();
        let calendar = SimpleCalendar::new(&sample_calendars["TARGET"]);
        let cash_flows = bond.rollout_cash_flows(1., &calendar).unwrap();
        assert_eq!(cash_flows.len(), 5);
        let curr = Currency::from_str("EUR").unwrap();
        let reference_cash_flows = vec![
            CashFlow::new(
                0.05 * 1000. / 2.,
                curr,
                NaiveDate::from_ymd_opt(2021, 4, 1).unwrap(),
            ),
            CashFlow::new(
                0.05 * 1000. / 2.,
                curr,
                NaiveDate::from_ymd_opt(2021, 10, 1).unwrap(),
            ),
            CashFlow::new(
                0.05 * 1000. / 2.,
                curr,
                NaiveDate::from_ymd_opt(2022, 4, 1).unwrap(),
            ),
            CashFlow::new(
                0.05 * 1000. / 2.,
                curr,
                NaiveDate::from_ymd_opt(2022, 10, 3).unwrap(),
            ),
            CashFlow::new(1000., curr, NaiveDate::from_ymd_opt(2022, 10, 3).unwrap()),
        ];
        let tol = 1e-11;
        assert!(reference_cash_flows[0].fuzzy_cash_flows_cmp_eq(&cash_flows[0], tol));
        assert!(reference_cash_flows[1].fuzzy_cash_flows_cmp_eq(&cash_flows[1], tol));
        assert!(reference_cash_flows[2].fuzzy_cash_flows_cmp_eq(&cash_flows[2], tol));
        assert!(reference_cash_flows[3].fuzzy_cash_flows_cmp_eq(&cash_flows[3], tol));
        assert!(reference_cash_flows[4].fuzzy_cash_flows_cmp_eq(&cash_flows[4], tol));
    }
}
