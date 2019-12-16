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
        let yf= self.day_count_conv.year_fraction(today, pay_date, None, None).unwrap();
        match self.compounding {
            Compounding::Simple => 1./(1. + self.rate * yf),
            Compounding::Annual => {
                (1.+self.rate).powf(-yf)
            },
            Compounding::SemiAnnual =>  {
                (1.+0.5*self.rate).powf(-2.*yf)
            },
            Compounding::Quarterly =>  {
                (1.+0.25*self.rate).powf(-4.*yf)
            },
            Compounding::Monthly =>  {
                (1.+self.rate/12.).powf(-12.*yf)
            },
            Compounding::Continuous => {
                (-self.rate * yf).exp()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::fuzzy_eq_absolute;
    use crate::time_period::TimePeriod;
    use std::str::FromStr;
    use std::f64;

    #[test]
    fn compounding_methods() {
        let tol = 1e-11;
        let rate = FlatRate{ rate: 0.05, day_count_conv: DayCountConv::Act365, compounding: Compounding::Annual };
        let start_date = NaiveDate::from_ymd(2019,12,16);
        let end_date = start_date + TimePeriod::from_str("6M").unwrap();
        let yf: f64 = DayCountConv::Act365.year_fraction(start_date, end_date, None, None).unwrap();
        assert!( fuzzy_eq_absolute(
            rate.discount_factor(start_date, end_date),
            f64::powf(1.0+0.05, -yf), tol) 
        );

        let rate = FlatRate{ rate: 0.05, day_count_conv: DayCountConv::Act365, compounding: Compounding::SemiAnnual };
        assert!( fuzzy_eq_absolute(
            rate.discount_factor(start_date, end_date),
            f64::powf(1.0+0.025, -yf*2.), tol) 
        );

        let rate = FlatRate{ rate: 0.05, day_count_conv: DayCountConv::Act365, compounding: Compounding::Quarterly };
        assert!( fuzzy_eq_absolute(
            rate.discount_factor(start_date, end_date),
            f64::powf(1.0+0.0125, -yf*4.), tol) 
        );
    
        let rate = FlatRate{ rate: 0.05, day_count_conv: DayCountConv::Act365, compounding: Compounding::Monthly };
        println!( "{},{}",
            rate.discount_factor(start_date, end_date),
            f64::powf(1.0+0.05/12., -yf*12.)
        );      
        assert!( fuzzy_eq_absolute(
            rate.discount_factor(start_date, end_date),
            f64::powf(1.0+0.05/12., -yf*12.), tol) 
        );    
    
        let rate = FlatRate{ rate: 0.05, day_count_conv: DayCountConv::Act365, compounding: Compounding::Continuous };
        assert!( fuzzy_eq_absolute(
            rate.discount_factor(start_date, end_date),
            f64::exp(-0.05*yf), tol) 
        );    
    
        let rate = FlatRate{ rate: 0.05, day_count_conv: DayCountConv::Act365, compounding: Compounding::Simple };
        assert!( fuzzy_eq_absolute(
            rate.discount_factor(start_date, end_date),
            1./(1. + 0.05*yf), tol) 
        );    
    }
}