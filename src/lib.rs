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
//! cash flows, and calculating valuation and risk figures like internal yield or duration that are useful
//! for an investor in these products.
//!
//! Functionality to calculate of figures like fair values which are primarily interesting in scenarios
//! where one is fully hedged are not in the initial focus, since an investor is by definition not
//! fully hedged. Nevertheless, they might be added later for comparison and estimating market prices.
//!
//! The library also supports storing data, like market data, e.g. market quote information, data related
//! to portfolio and transaction management to be able to support portfolio analysis (e.g. calculation
//! of risk figures), and generic storage of product details (e.g. bond specification). This is done by
//! defining data handler traits for various data categories, with concrete implementations supporting
//! storage in memory or in a databases (supporting `sqlite3` and `postgreSQL`).
//!

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
pub mod helpers;
pub mod market;
pub mod memory_handler;
pub mod portfolio;
pub mod postgres_handler;
pub mod quote;
pub mod rates;
pub mod sqlite_handler;
pub mod time_period;
pub mod transaction;
pub mod fx_rates;

pub use currency::Currency;
pub use fixed_income::CashAmount;
pub use fixed_income::CashFlow;
