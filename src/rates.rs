use serde::{Deserialize,Serialize};
use crate::day_count_conv::DayCountConv;
use chrono::NaiveDate;

/// Methods for compounding interest rates
#[derive(Deserialize, Serialize, Debug)]
pub enum Compounding {
    #[serde(rename = "simple")]
    Simple,
    #[serde(rename = "annual")]
    Annual,
    #[serde(rename = "semi-annual")]
    SemiAnnual,
    #[serde(rename = "quarterly")]
    Quarterly,
    #[serde(rename = "monthly")]
    Monthly,
    #[serde(rename = "continuous")]
    Continuous,
}

/// The `Discounter` trait provides a method for calculating discount factors.
/// This could be applied to falt raters, rate curves, or more complex models.
pub trait Discounter {
    /// Calculate the factor to discount a cash flow at `pay_date` to `today`.
    fn discount_factor(&self, today: NaiveDate, pay_date: NaiveDate) -> f64;
}

#[derive(Deserialize, Serialize, Debug)]
struct FlatRate {
    rate: f64,
    day_count_conv: DayCountConv,
    compounding: Compounding,
}

impl Discounter for FlatRate {
    fn discount_factor(&self, today: NaiveDate, pay_date: NaiveDate) -> f64 {
        let yf = self.day_count_conv.year_fraction(today, pay_date, None, None).unwrap();
        match self.compounding {
            Compounding::Simple => 1./(self.rate * yf),
            Compounding::Annual => {
                (1.+1./self.rate).powf(-yf)
            },
            Compounding::SemiAnnual =>  {
                (1.+0.5/self.rate).powf(-2.*yf)
            },
            Compounding::Quarterly =>  {
                (1.+0.25/self.rate).powf(-4.*yf)
            },
            Compounding::Monthly =>  {
                (1.+1./12./self.rate).powf(-12.*yf)
            },
            Compounding::Continuous => {
                (-self.rate * yf).exp()
            }
        }
    }
}