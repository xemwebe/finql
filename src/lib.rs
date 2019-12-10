//! # finql
//!
//! Purpose of this library is to provide over time a comprehensive toolbox
//! for quantitative analysis of financial assets in rust.
//! The project is licensed under Apache 2.0 or MIT license (see files LICENSE-Apache2.0 and LICENSE-MIT)
//! at the option of the user.
//!
//! The goal is to provide a toolbox for pricing various financial products, like bonds options or maybe
//! even more complex products. In the near term, calculation of the discounted cash flow value of bonds
//! is in the focus, based on what information is given by a standard prospect. Building blocks to achieve
//! this target include time periods (e.g. "3M" or "10Y"), bank holiday calendars, business day adjustment
//! rules, calculation of year fraction with respect to typical day count convention methods, roll-out of
//! cash flows and setup of interest rate curves for calculating the discounted cash flow value.

pub mod utility;

pub mod time_period;
pub use time_period::*;

pub mod calendar;
pub use calendar::*;

pub mod day_count_conv;
pub use day_count_conv::DayCountConv;

pub mod bond;
pub use bond::*;

pub mod coupon_date;

pub mod currency;
pub use currency::Currency;
