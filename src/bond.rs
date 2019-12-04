use serde::{Deserialize, Serialize};
use chrono::NaiveDate;
use crate::calendar::{DayAdjust,DayCountConv};
use crate::time_period::TimePeriod;

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
    currency: String,
    coupon : Coupon,
    business_day_rule: DayAdjust,
    calendar: String,
    issue_date: NaiveDate,
    maturity: NaiveDate,
    denomination: u32,
    volume: f64
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
    coupon_date: CouponDate,
    period: TimePeriod,
    day_count_convention: DayCountConv,
}

