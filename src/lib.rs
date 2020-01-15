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

// macro exports
#[macro_use]
pub mod macros;

// module exports
pub mod asset;
pub mod bond;
pub mod calendar;
pub mod coupon_date;
pub mod currency;
pub mod data_handler;
pub mod day_adjust;
pub mod day_count_conv;
pub mod fixed_income;
pub mod market;
pub mod portfolio;
pub mod quote;
pub mod rates;
pub mod sqlite_handler;
pub mod time_period;
pub mod transaction;
