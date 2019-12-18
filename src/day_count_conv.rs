//! Implementation of day count conventions to calculate year fractions between to dates.

use crate::time_period::TimePeriod;
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// Specify a day count method
#[derive(Deserialize, Serialize, Debug)]
pub enum DayCountConv {
    #[serde(rename = "icma")]
    #[serde(alias = "act/act icma")]
    #[serde(alias = "Act/Act")]
    #[serde(alias = "Act/Act ICMA")]
    ActActICMA,
    #[serde(rename = "act/365")]
    #[serde(alias = "Act/365f")]
    Act365,
    #[serde(rename = "act/365l")]
    #[serde(alias = "act/365leap")]
    Act365l,
    #[serde(rename = "act/360")]
    Act360,
    #[serde(rename = "30/360")]
    D30_360,
    #[serde(rename = "30E/360")]
    D30E360,
}

/// Specify a day count method error,
/// e.g. missing parameters in calculation of year fraction
#[derive(Debug)]
pub enum DayCountConvError {
    Impossible360,
    IcmaMissingTimePeriod,
    IcmaMissingRollDate,
    IcmaNoFrequency,
}

impl Display for DayCountConvError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            DayCountConvError::Impossible360 => write!(
                f,
                "day count convention in 30/x style are not applicable periods from 30th to 31st"
            ),
            DayCountConvError::IcmaMissingTimePeriod => {
                write!(f, "missing time period required for Act/Act ICMA")
            }
            DayCountConvError::IcmaMissingRollDate => {
                write!(f, "missing roll date required for Act/Act ICMA")
            }
            DayCountConvError::IcmaNoFrequency => write!(
                f,
                "time period can't be converted to frequency as required by Act/Act ICMA"
            ),
        }
    }
}

impl Error for DayCountConvError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl DayCountConv {
    /// The implementation of this function is in line with the
    /// definitions in "Derivatives and Internal Models", 5th edition,
    /// H.-P. Deutsch and M. W. Beinker, Palgrave-Macmillan 2019
    pub fn year_fraction(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        roll_date: Option<NaiveDate>,
        time_period: Option<TimePeriod>,
    ) -> Result<f64, DayCountConvError> {
        let since = NaiveDate::signed_duration_since;
        match self {
            DayCountConv::Act365 => Ok(since(end, start).num_days() as f64 / 365.),
            DayCountConv::Act365l => Ok(DayCountConv::calc_act_365_leap(start, end)),
            DayCountConv::Act360 => Ok(since(end, start).num_days() as f64 / 360.),
            // Check that this method is not applied to scenarios where it does not yield sensible results.
            // E.g. for one-day periods from 30th to 31st of the same month, with zero result
            DayCountConv::D30_360 => {
                let yf = DayCountConv::calc_30_360(start, end);
                if yf == 0. && start != end {
                    Err(DayCountConvError::Impossible360)
                } else {
                    Ok(yf)
                }
            }
            // Same as above
            DayCountConv::D30E360 => {
                let yf = DayCountConv::calc_30_e_360(start, end);
                if yf == 0. && start != end {
                    Err(DayCountConvError::Impossible360)
                } else {
                    Ok(yf)
                }
            }
            DayCountConv::ActActICMA => match roll_date {
                None => Err(DayCountConvError::IcmaMissingRollDate),
                Some(roll_date) => match time_period {
                    None => Err(DayCountConvError::IcmaMissingTimePeriod),
                    Some(time_period) => {
                        DayCountConv::calc_act_act_icma(start, end, roll_date, time_period)
                    }
                },
            },
        }
    }

    /// Implementation of act/365leap day count method
    fn calc_act_365_leap(start: NaiveDate, end: NaiveDate) -> f64 {
        let mut yf = (end.year() - start.year()) as f64;
        yf +=
            DayCountConv::days_to_date(end) as f64 / DayCountConv::days_in_year(end.year()) as f64;
        yf - DayCountConv::days_to_date(start) as f64
            / DayCountConv::days_in_year(start.year()) as f64
    }

    /// Implementation of 30/360 day count method
    fn calc_30_360(start: NaiveDate, end: NaiveDate) -> f64 {
        let yf = (end.year() - start.year()) as f64 + (end.month() - start.month()) as f64 / 12.;
        let start_day = std::cmp::min(start.day(), 30) as i32;
        let end_day = if start_day == 30 && end.day() == 31 {
            30
        } else {
            end.day()
        } as i32;
        yf + (end_day - start_day) as f64 / 360.
    }

    /// Implementation of 30E/360 day count method
    fn calc_30_e_360(start: NaiveDate, end: NaiveDate) -> f64 {
        (end.year() - start.year()) as f64
            + (end.month() - start.month()) as f64 / 12.
            + (std::cmp::min(end.day(), 30) as i32 - std::cmp::min(start.day(), 30) as i32) as f64
                / 360.
    }

    fn calc_act_act_icma(
        start: NaiveDate,
        end: NaiveDate,
        roll_date: NaiveDate,
        time_period: TimePeriod,
    ) -> Result<f64, DayCountConvError> {
        let frequency = time_period.frequency();
        match frequency {
            Err(_) => Err(DayCountConvError::IcmaNoFrequency),
            Ok(frequency) => {
                let freq: f64 = frequency as f64;
                let mut base = roll_date;
                while base < start {
                    base = time_period.add_to(base, None);
                }
                while base > end {
                    base = time_period.sub_from(base, None);
                }
                if base < start {
                    // Period between start and end is shorter than natural period
                    let days = end.signed_duration_since(start).num_days() as f64;
                    let period_days = time_period
                        .add_to(base, None)
                        .signed_duration_since(base)
                        .num_days() as f64;
                    return Ok(days / (period_days * freq));
                }
                let mut periods = 0;
                let mut b = base;
                let mut yf = 0.;
                while b > start {
                    b = time_period.sub_from(b, None);
                    if b >= start {
                        periods += 1;
                    }
                }
                if b < start {
                    // first period is broken, add fraction
                    let be = time_period.add_to(b, None);
                    let days = be.signed_duration_since(start).num_days() as f64;
                    let period_days = be.signed_duration_since(b).num_days() as f64;
                    yf += days / period_days;
                };
                while base < end {
                    base = time_period.add_to(base, None);
                    if base <= end {
                        periods += 1;
                    }
                }
                if base > end {
                    // last period is broken, add fraction
                    let bs = time_period.sub_from(base, None);
                    let days = end.signed_duration_since(bs).num_days() as f64;
                    let period_days = base.signed_duration_since(bs).num_days() as f64;
                    yf += days / period_days;
                };
                Ok((yf + periods as f64) / freq)
            }
        }
    }

    /// Calculate the number of day in a given year.
    fn days_in_year(year: i32) -> u32 {
        if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            366
        } else {
            365
        }
    }

    /// Calculate the number of days since January 1st this year
    fn days_to_date(date: NaiveDate) -> i64 {
        date.signed_duration_since(NaiveDate::from_ymd(date.year(), 1, 1))
            .num_days()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days_this_year() {
        assert_eq!(DayCountConv::days_in_year(2019), 365);
        assert_eq!(DayCountConv::days_in_year(2020), 366);
        assert_eq!(DayCountConv::days_in_year(2000), 366);
        assert_eq!(DayCountConv::days_in_year(2100), 365);
    }

    #[test]
    fn days_since_date() {
        assert_eq!(
            DayCountConv::days_to_date(NaiveDate::from_ymd(2020, 10, 1)),
            274
        );
        assert_eq!(
            DayCountConv::days_to_date(NaiveDate::from_ymd(2019, 10, 1)),
            273
        );
        assert_eq!(
            DayCountConv::days_to_date(NaiveDate::from_ymd(2019, 1, 1)),
            0
        );
        assert_eq!(
            DayCountConv::days_to_date(NaiveDate::from_ymd(2019, 12, 31)),
            364
        );
        assert_eq!(
            DayCountConv::days_to_date(NaiveDate::from_ymd(2020, 12, 31)),
            365
        );
    }

    #[test]
    fn calc_year_fractions_act_x() {
        let tol = 1e-11;
        let dcc365 = DayCountConv::Act365;
        let dcc365l = DayCountConv::Act365l;
        let dcc360 = DayCountConv::Act360;

        let start = NaiveDate::from_ymd(2019, 10, 1);
        let end = NaiveDate::from_ymd(2020, 10, 1);
        assert_fuzzy_eq!(
            dcc365.year_fraction(start, end, None, None).unwrap(),
            366. / 365.,
            tol
        );
        assert_fuzzy_eq!(
            dcc365l.year_fraction(start, end, None, None).unwrap(),
            92. / 365. + 274. / 366.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            366. / 360.,
            tol
        );

        let start = NaiveDate::from_ymd(2019, 10, 1);
        let end = NaiveDate::from_ymd(2019, 11, 1);
        assert_fuzzy_eq!(
            dcc365.year_fraction(start, end, None, None).unwrap(),
            31. / 365.,
            tol
        );
        assert_fuzzy_eq!(
            dcc365l.year_fraction(start, end, None, None).unwrap(),
            31. / 365.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );
    }

    #[test]
    fn calc_year_fractions_30_360() {
        let tol = 1e-11;
        let dcc360 = DayCountConv::D30_360;
        let dcc360e = DayCountConv::D30E360;
        let start = NaiveDate::from_ymd(2019, 7, 29);
        let end = NaiveDate::from_ymd(2019, 8, 29);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2019, 8, 30);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2019, 8, 31);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            32. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2019, 9, 1);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            32. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            32. / 360.,
            tol
        );

        let start = NaiveDate::from_ymd(2019, 7, 30);
        let end = NaiveDate::from_ymd(2019, 8, 29);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            29. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            29. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2019, 8, 30);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2019, 8, 31);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2019, 9, 1);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );

        let start = NaiveDate::from_ymd(2019, 7, 31);
        let end = NaiveDate::from_ymd(2019, 8, 29);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            29. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            29. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2019, 8, 30);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2019, 8, 31);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2019, 9, 1);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );

        let start = NaiveDate::from_ymd(2019, 6, 29);
        let end = NaiveDate::from_ymd(2019, 7, 29);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2019, 7, 30);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2019, 7, 31);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            32. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2019, 8, 1);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            32. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            32. / 360.,
            tol
        );

        let start = NaiveDate::from_ymd(2019, 6, 30);
        let end = NaiveDate::from_ymd(2019, 7, 29);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            29. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            29. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2019, 7, 30);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2019, 7, 31);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2019, 8, 1);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );

        let start = NaiveDate::from_ymd(2019, 8, 29);
        let end = NaiveDate::from_ymd(2019, 9, 29);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2019, 9, 30);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2019, 10, 1);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            32. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            32. / 360.,
            tol
        );

        let start = NaiveDate::from_ymd(2019, 8, 30);
        let end = NaiveDate::from_ymd(2019, 9, 29);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            29. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            29. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2019, 9, 30);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2019, 10, 1);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );

        let start = NaiveDate::from_ymd(2019, 8, 31);
        let end = NaiveDate::from_ymd(2019, 9, 29);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            29. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            29. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2019, 9, 30);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2019, 10, 1);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );

        let start = NaiveDate::from_ymd(2020, 1, 29);
        let end = NaiveDate::from_ymd(2020, 2, 28);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            29. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            29. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2020, 2, 29);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );

        let start = NaiveDate::from_ymd(2020, 1, 30);
        let end = NaiveDate::from_ymd(2020, 2, 28);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            28. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            28. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2020, 2, 29);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            29. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            29. / 360.,
            tol
        );

        let start = NaiveDate::from_ymd(2020, 1, 31);
        let end = NaiveDate::from_ymd(2020, 2, 28);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            28. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            28. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2020, 2, 29);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            29. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            29. / 360.,
            tol
        );

        let start = NaiveDate::from_ymd(2020, 2, 28);
        let end = NaiveDate::from_ymd(2020, 3, 28);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2020, 3, 29);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2020, 3, 30);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            32. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            32. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2020, 3, 31);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            33. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            32. / 360.,
            tol
        );

        let start = NaiveDate::from_ymd(2020, 2, 29);
        let end = NaiveDate::from_ymd(2020, 3, 28);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            29. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            29. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2020, 3, 29);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2020, 3, 30);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2020, 3, 31);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            32. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );

        let start = NaiveDate::from_ymd(2019, 1, 28);
        let end = NaiveDate::from_ymd(2019, 2, 28);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );

        let start = NaiveDate::from_ymd(2020, 1, 29);
        let end = NaiveDate::from_ymd(2020, 2, 28);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            29. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            29. / 360.,
            tol
        );

        let start = NaiveDate::from_ymd(2020, 1, 30);
        let end = NaiveDate::from_ymd(2020, 2, 28);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            28. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            28. / 360.,
            tol
        );

        let start = NaiveDate::from_ymd(2020, 1, 31);
        let end = NaiveDate::from_ymd(2020, 2, 28);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            28. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            28. / 360.,
            tol
        );

        let start = NaiveDate::from_ymd(2020, 2, 28);
        let end = NaiveDate::from_ymd(2020, 3, 28);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            30. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2020, 3, 29);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            31. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2020, 3, 30);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            32. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            32. / 360.,
            tol
        );
        let end = NaiveDate::from_ymd(2020, 3, 31);
        assert_fuzzy_eq!(
            dcc360.year_fraction(start, end, None, None).unwrap(),
            33. / 360.,
            tol
        );
        assert_fuzzy_eq!(
            dcc360e.year_fraction(start, end, None, None).unwrap(),
            32. / 360.,
            tol
        );
    }
    #[test]
    fn calc_year_fractions_icma() {
        let tol = 1e-11;
        let start = NaiveDate::from_ymd(2019, 10, 1);
        let end = NaiveDate::from_ymd(2019, 11, 1);
        let dcc = DayCountConv::ActActICMA;
        let tp = Some("1M".parse::<TimePeriod>().unwrap());

        // Natural period
        let roll_date = Some(NaiveDate::from_ymd(2019, 10, 1));
        assert_fuzzy_eq!(
            dcc.year_fraction(start, end, roll_date, tp).unwrap(),
            1. / 12.,
            tol
        );
        let roll_date = Some(NaiveDate::from_ymd(2019, 11, 1));
        assert_fuzzy_eq!(
            dcc.year_fraction(start, end, roll_date, tp).unwrap(),
            1. / 12.,
            tol
        );
        let roll_date = Some(NaiveDate::from_ymd(2019, 9, 1));
        assert_fuzzy_eq!(
            dcc.year_fraction(start, end, roll_date, tp).unwrap(),
            1. / 12.,
            tol
        );
        let roll_date = Some(NaiveDate::from_ymd(2019, 12, 1));
        assert_fuzzy_eq!(
            dcc.year_fraction(start, end, roll_date, tp).unwrap(),
            1. / 12.,
            tol
        );

        // Shifted natural period
        let roll_date = Some(NaiveDate::from_ymd(2019, 9, 15));
        let yf = (14. / 30. + 17. / 31.) / 12.;
        assert_fuzzy_eq!(
            dcc.year_fraction(start, end, roll_date, tp).unwrap(),
            yf,
            tol
        );
        let roll_date = Some(NaiveDate::from_ymd(2019, 10, 15));
        assert_fuzzy_eq!(
            dcc.year_fraction(start, end, roll_date, tp).unwrap(),
            yf,
            tol
        );
        let roll_date = Some(NaiveDate::from_ymd(2019, 11, 15));
        assert_fuzzy_eq!(
            dcc.year_fraction(start, end, roll_date, tp).unwrap(),
            yf,
            tol
        );

        // Short period
        let start = NaiveDate::from_ymd(2019, 10, 5);
        let end = NaiveDate::from_ymd(2019, 10, 15);
        let roll_date = Some(NaiveDate::from_ymd(2019, 10, 1));
        let yf = 10. / 31. / 12.;
        assert_fuzzy_eq!(
            dcc.year_fraction(start, end, roll_date, tp).unwrap(),
            yf,
            tol
        );
        let roll_date = Some(NaiveDate::from_ymd(2019, 11, 1));
        assert_fuzzy_eq!(
            dcc.year_fraction(start, end, roll_date, tp).unwrap(),
            yf,
            tol
        );
        let roll_date = Some(NaiveDate::from_ymd(2019, 10, 5));
        assert_fuzzy_eq!(
            dcc.year_fraction(start, end, roll_date, tp).unwrap(),
            yf,
            tol
        );
        let roll_date = Some(NaiveDate::from_ymd(2019, 10, 15));
        let yf = 10. / 30. / 12.;
        assert_fuzzy_eq!(
            dcc.year_fraction(start, end, roll_date, tp).unwrap(),
            yf,
            tol
        );
        let roll_date = Some(NaiveDate::from_ymd(2019, 10, 10));
        let yf = (5. / 31. + 5. / 30.) / 12.;
        assert_fuzzy_eq!(
            dcc.year_fraction(start, end, roll_date, tp).unwrap(),
            yf,
            tol
        );

        // Long period
        let start = NaiveDate::from_ymd(2019, 10, 1);
        let end = NaiveDate::from_ymd(2020, 1, 1);
        let roll_date = Some(NaiveDate::from_ymd(2019, 9, 1));
        let yf = 3. / 12.;
        assert_fuzzy_eq!(
            dcc.year_fraction(start, end, roll_date, tp).unwrap(),
            yf,
            tol
        );
        let roll_date = Some(NaiveDate::from_ymd(2019, 10, 1));
        assert_fuzzy_eq!(
            dcc.year_fraction(start, end, roll_date, tp).unwrap(),
            yf,
            tol
        );
        let roll_date = Some(NaiveDate::from_ymd(2019, 12, 1));
        assert_fuzzy_eq!(
            dcc.year_fraction(start, end, roll_date, tp).unwrap(),
            yf,
            tol
        );
        let roll_date = Some(NaiveDate::from_ymd(2020, 1, 1));
        assert_fuzzy_eq!(
            dcc.year_fraction(start, end, roll_date, tp).unwrap(),
            yf,
            tol
        );

        let roll_date = Some(NaiveDate::from_ymd(2019, 9, 15));
        let yf = (14. / 30. + 2. + 17. / 31.) / 12.;
        assert_fuzzy_eq!(
            dcc.year_fraction(start, end, roll_date, tp).unwrap(),
            yf,
            tol
        );
        let roll_date = Some(NaiveDate::from_ymd(2019, 10, 15));
        assert_fuzzy_eq!(
            dcc.year_fraction(start, end, roll_date, tp).unwrap(),
            yf,
            tol
        );
        let roll_date = Some(NaiveDate::from_ymd(2020, 2, 15));
        assert_fuzzy_eq!(
            dcc.year_fraction(start, end, roll_date, tp).unwrap(),
            yf,
            tol
        );

        let start = NaiveDate::from_ymd(2019, 10, 5);
        let end = NaiveDate::from_ymd(2019, 12, 20);
        let roll_date = Some(NaiveDate::from_ymd(2019, 9, 1));
        let yf = (27. / 31. + 1. + 19. / 31.) / 12.;
        assert_fuzzy_eq!(
            dcc.year_fraction(start, end, roll_date, tp).unwrap(),
            yf,
            tol
        );
        let roll_date = Some(NaiveDate::from_ymd(2019, 10, 1));
        assert_fuzzy_eq!(
            dcc.year_fraction(start, end, roll_date, tp).unwrap(),
            yf,
            tol
        );
        let roll_date = Some(NaiveDate::from_ymd(2019, 12, 1));
        assert_fuzzy_eq!(
            dcc.year_fraction(start, end, roll_date, tp).unwrap(),
            yf,
            tol
        );
        let roll_date = Some(NaiveDate::from_ymd(2020, 1, 1));
        assert_fuzzy_eq!(
            dcc.year_fraction(start, end, roll_date, tp).unwrap(),
            yf,
            tol
        );
    }
}
