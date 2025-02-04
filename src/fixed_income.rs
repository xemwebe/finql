use std::f64;

use argmin::core::{CostFunction, Error, Executor};
use argmin::solver::brent::BrentRoot;
use chrono::NaiveDate;

use crate::datatypes::CashFlow;

use crate::day_count_conv::DayCountConv;
use crate::rates::{Compounding, DiscountError, Discounter, FlatRate};
use cal_calc::CalendarProvider;

/// Get all future cash flows with respect to a given date
pub fn get_cash_flows_after(cash_flows: &[CashFlow], date: NaiveDate) -> Vec<CashFlow> {
    let mut new_cash_flows = Vec::new();
    for cf in cash_flows {
        if cf.date > date {
            new_cash_flows.push(*cf);
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
        calendar_provider: &dyn CalendarProvider,
    ) -> Result<Vec<CashFlow>, Self::Error>;

    /// Calculate accrued interest for current coupon period
    fn accrued_interest(&self, today: NaiveDate) -> Result<f64, Self::Error>;

    /// Calculate the yield to maturity (YTM) given a purchase price and date
    fn calculate_ytm(
        &self,
        purchase_cash_flow: &CashFlow,
        calendar_provider: &dyn CalendarProvider,
    ) -> Result<f64, Self::Error> {
        let cash_flows = self.rollout_cash_flows(1., calendar_provider)?;
        let value = calculate_cash_flows_ytm(&cash_flows, purchase_cash_flow)?;
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
    cash_flows: &[CashFlow],
    init_cash_flow: &CashFlow,
) -> Result<f64, DiscountError> {
    let rate = FlatRate::new(
        0.05,
        DayCountConv::Act365,
        Compounding::Annual,
        init_cash_flow.amount.currency,
    );
    let init_param = 0.5;
    let solver = BrentRoot::new(0., 0.5, 1e-11);
    let func = FlatRateDiscounter {
        init_cash_flow,
        cash_flows,
        rate,
    };
    let res = Executor::new(func, solver)
        .configure(|state| state.max_iters(100).param(init_param))
        .run();
    match res {
        Ok(mut val) => match val.state.take_param() {
            Some(param) => Ok(param),
            None => Err(DiscountError),
        },
        Err(_) => Err(DiscountError),
    }
}

/// Calculate discounted value for given flat rate
#[derive(Clone)]
struct FlatRateDiscounter<'a> {
    init_cash_flow: &'a CashFlow,
    cash_flows: &'a [CashFlow],
    rate: FlatRate,
}

impl CostFunction for FlatRateDiscounter<'_> {
    // one dimensional problem, no vector needed
    type Param = f64;
    type Output = f64;

    fn cost(&self, p: &Self::Param) -> Result<Self::Output, Error> {
        let mut discount_rate = self.rate;
        discount_rate.rate = *p;
        let mut sum = self.init_cash_flow.amount.amount;
        let today = self.init_cash_flow.date;
        for cf in self.cash_flows {
            if cf.date > today {
                sum += discount_rate.discount_cash_flow(cf, today)?.amount;
            }
        }
        Ok(sum)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Local, TimeZone};
    use std::collections::BTreeMap;
    use std::str::FromStr;

    use crate::datatypes::{CashAmount, CashFlow, Currency};

    use super::*;
    use crate::fx_rates::SimpleCurrencyConverter;

    #[test]
    fn yield_to_maturity() {
        let tol = 1e-11;
        let curr = Currency::from_str("EUR").unwrap();
        let cash_flows = vec![CashFlow::new(
            1050.,
            curr,
            NaiveDate::from_ymd_opt(2021, 10, 1),
        )];
        let init_cash_flow = CashFlow::new(-1000., curr, NaiveDate::from_ymd_opt(2020, 10, 1));

        let ytm = calculate_cash_flows_ytm(&cash_flows, &init_cash_flow).unwrap();
        assert_fuzzy_eq!(ytm, 0.05, tol);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn cash_amount_arithmetic_simple() {
        let tol = 1e-11;
        let time = Local.ymd(2020, 4, 6).and_hms_milli(18, 0, 0, 0);

        let eur = Currency::from_str("EUR").unwrap();
        let jpy = Currency::from_str("JPY").unwrap();

        let fx_rate = 81.2345;
        // temporary storage for fx rates
        let mut fx_converter = SimpleCurrencyConverter::new();
        fx_converter.insert_fx_rate(eur, jpy, fx_rate);

        let eur_amount = CashAmount {
            amount: 100.0,
            currency: eur,
        };
        let jpy_amount = CashAmount {
            amount: 7500.0,
            currency: jpy,
        };
        let eur2_amount = CashAmount {
            amount: 200.0,
            currency: eur,
        };

        let mut tmp = CashAmount {
            amount: 0.0,
            currency: eur,
        };
        // Simple addition, same currency
        tmp.add(eur_amount, time, &fx_converter, false)
            .await
            .unwrap();
        assert_fuzzy_eq!(tmp.amount, 100.0, tol);
        // Adding optional cash amount
        tmp.add_opt(Some(eur2_amount), time, &fx_converter, false)
            .await
            .unwrap();
        assert_fuzzy_eq!(tmp.amount, 300.0, tol);
        // Adding optional cash amount that is none
        tmp.add_opt(None, time, &fx_converter, false).await.unwrap();
        assert_fuzzy_eq!(tmp.amount, 300.0, tol);
        // Adding optional foreign cash amount
        tmp.add_opt(Some(jpy_amount), time, &fx_converter, false)
            .await
            .unwrap();
        assert_fuzzy_eq!(tmp.amount, 300.0 + 7500.0 / fx_rate, tol);
        // Substract foreign cash amount
        tmp.sub(jpy_amount, time, &fx_converter, false)
            .await
            .unwrap();
        assert_fuzzy_eq!(tmp.amount, 300.0, tol);
        // Substract optional None cash amount
        tmp.sub_opt(None, time, &fx_converter, false).await.unwrap();
        assert_fuzzy_eq!(tmp.amount, 300.0, tol);
        // Substract optional cash amount, same currency
        tmp.sub_opt(Some(eur_amount), time, &fx_converter, false)
            .await
            .unwrap();
        assert_fuzzy_eq!(tmp.amount, 200.0, tol);

        // Sum must be in EUR, since tmp was originally in EUR
        assert_eq!(tmp.currency.to_string(), "EUR");
        let mut curr_rounding_conventions = BTreeMap::new();
        curr_rounding_conventions.insert("JPY".to_string(), 0);

        let mut tmp = eur_amount;
        tmp.add(jpy_amount, time, &fx_converter, false)
            .await
            .unwrap();
        let tmp = tmp.round_by_convention(&curr_rounding_conventions);
        assert_fuzzy_eq!(
            tmp.amount,
            ((100.0 + 7500.0 / fx_rate) * 100.0_f64).round() / 100.0,
            tol
        );

        let mut tmp = jpy_amount;
        tmp.add(eur_amount, time, &fx_converter, false)
            .await
            .unwrap();
        // Sum must be in EUR, since tmp was originally in EUR
        assert_eq!(tmp.currency.to_string(), "JPY");
        assert_fuzzy_eq!(tmp.amount, 7500.0 + 100.0 * fx_rate, tol);
        let tmp = tmp.round_by_convention(&curr_rounding_conventions);
        assert_fuzzy_eq!(tmp.amount, (7500.0 + 100.0 * fx_rate).round(), tol);

        // With automatic rounding according to conventions
        let mut tmp = eur_amount;
        tmp.add(jpy_amount, time, &fx_converter, true)
            .await
            .unwrap();
        assert_fuzzy_eq!(
            tmp.amount,
            ((100.0 + 7500.0 / fx_rate) * 100.0_f64).round() / 100.0,
            tol
        );

        // With automatic rounding according to conventions
        let mut tmp = jpy_amount;
        tmp.add(eur_amount, time, &fx_converter, true)
            .await
            .unwrap();
        assert_fuzzy_eq!(tmp.amount, (7500.0 + 100.0 * fx_rate).round(), tol);
    }
}
