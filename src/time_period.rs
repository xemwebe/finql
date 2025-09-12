//! The module `time_period` supports time periods of different lengths
//! in terms of day, months or years that can be added to a given date.
//! Time periods may als be negative.

use std::fmt;
use std::str::FromStr;

use cal_calc::{last_day_of_month, Calendar};
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::TryFrom;
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};
use thiserror::Error;
use time::{Date, Duration};

use crate::datatypes::date_time_helper::{from_time_date, to_time_date, DateTimeError};

/// Error type related to the TimePeriod struct
#[derive(Error, Debug)]
pub enum TimePeriodError {
    #[error("couldn't parse time period, string is too short")]
    ParseError,
    #[error("invalid time period unit, use one of 'D', 'B', 'W', 'M', or 'Y'")]
    InvalidUnit,
    #[error("parsing number of periods for time period failed")]
    InvalidPeriod,
    #[error("the time period can't be converted to frequency")]
    NoFrequency,
    #[error("conversion between differen date objects failed")]
    InvalidDateCovnersion(#[from] DateTimeError),
    #[error("calendar error")]
    CalendarError(#[from] cal_calc::CalendarError),
    #[error("invalid date")]
    InvalidDate,
}

/// Possible units of a time period
#[derive(Debug, Copy, Clone, PartialEq)]
enum TimePeriodUnit {
    Daily,
    BusinessDaily,
    Weekly,
    Monthly,
    Annual,
}

impl fmt::Display for TimePeriodUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Daily => write!(f, "D"),
            Self::BusinessDaily => write!(f, "B"),
            Self::Weekly => write!(f, "W"),
            Self::Monthly => write!(f, "M"),
            Self::Annual => write!(f, "Y"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TimePeriod {
    num: i32,
    unit: TimePeriodUnit,
}

/// Transform a string into a TimePeriod
impl TimePeriod {
    /// Add time period to a given date.
    /// The function call will panic is the resulting year is out
    /// of the valid range or if not calendar is provided in case of BusinessDaily time periods
    pub fn add_to(&self, mut date: Date, cal: Option<&Calendar>) -> Result<Date, TimePeriodError> {
        Ok(match self.unit {
            TimePeriodUnit::Daily => date + Duration::days(self.num as i64),
            TimePeriodUnit::BusinessDaily => {
                let is_neg = self.num < 0;
                let n = self.num.abs();
                let cal = cal.unwrap();
                for _ in 0..n {
                    date = if is_neg {
                        from_time_date(cal.prev_bday(to_time_date(date))?)
                    } else {
                        from_time_date(cal.next_bday(to_time_date(date))?)
                    };
                }
                date
            }
            TimePeriodUnit::Weekly => date
                .checked_add(Duration::days(7 * self.num as i64))
                .unwrap(),
            // If the original day of the data is larger than the length
            // of the target month, the day is moved to the last day of the target month.
            // Therefore, `MonthlyPeriod` is not in all cases reversible by adding
            // the equivalent negative monthly period.
            TimePeriodUnit::Monthly => {
                let mut day = date.day();
                let mut month = date.month() as i32;
                let mut year = date.year();
                year += self.num / 12;
                month += self.num % 12;
                if month < 1 {
                    year -= 1;
                    month += 12;
                } else if month > 12 {
                    year += 1;
                    month -= 12;
                }
                if day > 28 {
                    let last_date_of_month = last_day_of_month(year, month as u8);
                    day = std::cmp::min(day, last_date_of_month as u8);
                }
                Date::from_calendar_date(year, time::Month::try_from(month as u8).unwrap(), day)
                    .map_err(|_| TimePeriodError::InvalidDate)?
            }
            TimePeriodUnit::Annual => {
                Date::from_calendar_date(date.year() + self.num, date.month(), date.day())
                    .map_err(|_| TimePeriodError::InvalidDate)?
            }
        })
    }

    /// Substract time period from a given date.
    pub fn sub_from(&self, date: Date, cal: Option<&Calendar>) -> Result<Date, TimePeriodError> {
        self.inverse().add_to(date, cal)
    }

    /// Substract time period from a given date.
    pub fn inverse(&self) -> TimePeriod {
        TimePeriod {
            num: -self.num,
            unit: self.unit,
        }
    }

    /// Returns the frequency per year, if this is possible,
    /// otherwise return error
    pub fn frequency(&self) -> Result<u16, TimePeriodError> {
        match self.unit {
            TimePeriodUnit::Daily | TimePeriodUnit::BusinessDaily | TimePeriodUnit::Weekly => {
                Err(TimePeriodError::NoFrequency)
            }
            TimePeriodUnit::Monthly => match self.num.abs() {
                1 => Ok(12),
                3 => Ok(4),
                6 => Ok(2),
                12 => Ok(1),
                _ => Err(TimePeriodError::NoFrequency),
            },
            TimePeriodUnit::Annual => {
                if self.num.abs() == 1 {
                    Ok(1)
                } else {
                    Err(TimePeriodError::NoFrequency)
                }
            }
        }
    }
}

impl fmt::Display for TimePeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.num, self.unit)
    }
}

/// Transform a string into a TimePeriod
impl FromStr for TimePeriod {
    type Err = TimePeriodError;

    fn from_str(tp: &str) -> Result<TimePeriod, TimePeriodError> {
        let len = tp.len();
        if len < 2 {
            Err(TimePeriodError::ParseError)
        } else {
            let unit = match tp.chars().last() {
                Some('D') => TimePeriodUnit::Daily,
                Some('B') => TimePeriodUnit::BusinessDaily,
                Some('W') => TimePeriodUnit::Weekly,
                Some('M') => TimePeriodUnit::Monthly,
                Some('Y') => TimePeriodUnit::Annual,
                _ => return Err(TimePeriodError::InvalidUnit),
            };
            let num = match tp[..len - 1].parse::<i32>() {
                Ok(val) => val,
                _ => return Err(TimePeriodError::InvalidPeriod),
            };

            Ok(TimePeriod { num, unit })
        }
    }
}

impl Serialize for TimePeriod {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", &self))
    }
}

struct TimePeriodVisitor;

impl Visitor<'_> for TimePeriodVisitor {
    type Value = TimePeriod;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a time period of the format [+|-]<int><unit>")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match TimePeriod::from_str(value) {
            Ok(val) => Ok(val),
            Err(err) => Err(E::custom(err.to_string())),
        }
    }
}

impl<'de> Deserialize<'de> for TimePeriod {
    fn deserialize<D>(deserializer: D) -> Result<TimePeriod, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(TimePeriodVisitor)
    }
}

impl Add<TimePeriod> for Date {
    type Output = Date;

    fn add(self, period: TimePeriod) -> Date {
        period.add_to(self, None).unwrap_or(self)
    }
}

impl Add<&TimePeriod> for Date {
    type Output = Date;

    fn add(self, period: &TimePeriod) -> Date {
        period.add_to(self, None).unwrap_or(self)
    }
}

impl AddAssign<TimePeriod> for Date {
    fn add_assign(&mut self, period: TimePeriod) {
        *self = period.add_to(*self, None).unwrap_or(*self)
    }
}

impl AddAssign<&TimePeriod> for Date {
    fn add_assign(&mut self, period: &TimePeriod) {
        *self = period.add_to(*self, None).unwrap_or(*self)
    }
}

impl Sub<TimePeriod> for Date {
    type Output = Date;

    fn sub(self, period: TimePeriod) -> Date {
        period.sub_from(self, None).unwrap_or(self)
    }
}

impl Sub<&TimePeriod> for Date {
    type Output = Date;

    fn sub(self, period: &TimePeriod) -> Date {
        period.sub_from(self, None).unwrap_or(self)
    }
}

impl SubAssign<TimePeriod> for Date {
    fn sub_assign(&mut self, period: TimePeriod) {
        *self = period.sub_from(*self, None).unwrap_or(*self)
    }
}

impl SubAssign<&TimePeriod> for Date {
    fn sub_assign(&mut self, period: &TimePeriod) {
        *self = period.sub_from(*self, None).unwrap_or(*self)
    }
}

impl Neg for TimePeriod {
    type Output = TimePeriod;

    fn neg(self) -> TimePeriod {
        self.inverse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cal_calc::{Calendar, Holiday};
    use time::Weekday;

    #[test]
    fn standard_periods() {
        let date = Date::from_calendar_date(2019, time::Month::November, 18).unwrap();
        assert_eq!(
            TimePeriod::from_str("3M").unwrap().add_to(date, None),
            Ok(Date::from_calendar_date(2020, time::Month::February, 18).unwrap())
        );
        assert_eq!(
            TimePeriod::from_str("1Y").unwrap().add_to(date, None),
            Ok(Date::from_calendar_date(2020, time::Month::November, 18).unwrap())
        );
        assert_eq!(
            TimePeriod::from_str("6M").unwrap().add_to(date, None),
            Ok(Date::from_calendar_date(2020, time::Month::May, 18).unwrap())
        );
        assert_eq!(
            TimePeriod::from_str("1W").unwrap().add_to(date, None),
            Ok(Date::from_calendar_date(2019, time::Month::November, 25).unwrap())
        );

        let date = Date::from_calendar_date(2019, time::Month::November, 30).unwrap();
        assert_eq!(
            TimePeriod::from_str("3M").unwrap().add_to(date, None),
            Ok(Date::from_calendar_date(2020, time::Month::February, 29).unwrap())
        );
        assert_eq!(
            TimePeriod::from_str("1Y").unwrap().add_to(date, None),
            Ok(Date::from_calendar_date(2020, time::Month::November, 30).unwrap())
        );
        assert_eq!(
            TimePeriod::from_str("6M").unwrap().add_to(date, None),
            Ok(Date::from_calendar_date(2020, time::Month::May, 30).unwrap())
        );
        assert_eq!(
            TimePeriod::from_str("1W").unwrap().add_to(date, None),
            Ok(Date::from_calendar_date(2019, time::Month::December, 7).unwrap())
        );
    }

    #[test]
    fn negative_periods() {
        let date = Date::from_calendar_date(2019, time::Month::November, 18).unwrap();
        let neg_quarterly = TimePeriod::from_str("-3M").unwrap();
        let neg_annual = TimePeriod::from_str("-1Y").unwrap();
        let neg_weekly = TimePeriod::from_str("-1W").unwrap();
        assert_eq!(
            neg_quarterly.add_to(
                Date::from_calendar_date(2020, time::Month::February, 18).unwrap(),
                None
            ),
            Ok(date)
        );
        assert_eq!(
            neg_annual.add_to(
                Date::from_calendar_date(2020, time::Month::November, 18).unwrap(),
                None
            ),
            Ok(date)
        );
        assert_eq!(
            neg_weekly.add_to(
                Date::from_calendar_date(2019, time::Month::November, 25).unwrap(),
                None
            ),
            Ok(date)
        );

        let date = Date::from_calendar_date(2019, time::Month::November, 30).unwrap();
        assert_eq!(
            neg_quarterly.add_to(
                Date::from_calendar_date(2020, time::Month::February, 29).unwrap(),
                None
            ),
            Ok(Date::from_calendar_date(2019, time::Month::November, 29).unwrap())
        );
        assert_eq!(
            neg_annual.add_to(
                Date::from_calendar_date(2020, time::Month::November, 30).unwrap(),
                None
            ),
            Ok(date)
        );
        assert_eq!(
            neg_weekly.add_to(
                Date::from_calendar_date(2019, time::Month::December, 7).unwrap(),
                None
            ),
            Ok(date)
        );
    }

    #[test]
    fn display_periods() {
        assert_eq!(format!("{}", TimePeriod::from_str("3M").unwrap()), "3M");
        assert_eq!(format!("{}", TimePeriod::from_str("6M").unwrap()), "6M");
        assert_eq!(format!("{}", TimePeriod::from_str("1Y").unwrap()), "1Y");
        assert_eq!(format!("{}", TimePeriod::from_str("1W").unwrap()), "1W");
        assert_eq!(format!("{}", TimePeriod::from_str("1D").unwrap()), "1D");
        assert_eq!(format!("{}", TimePeriod::from_str("-3M").unwrap()), "-3M");
        assert_eq!(format!("{}", TimePeriod::from_str("-1Y").unwrap()), "-1Y");
        assert_eq!(format!("{}", TimePeriod::from_str("-1W").unwrap()), "-1W");
        assert_eq!(format!("{}", TimePeriod::from_str("-1D").unwrap()), "-1D");
    }

    #[test]
    fn parse_business_daily() {
        let holiday_rules = vec![
            Holiday::SingularDay(
                Date::from_calendar_date(2019, time::Month::November, 21).unwrap(),
            ),
            Holiday::WeekDay(Weekday::Sat),
            Holiday::WeekDay(Weekday::Sun),
        ];

        let cal = Calendar::calc_calendar(&holiday_rules, 2019, 2020);
        let bdaily1 = TimePeriod::from_str("1B").unwrap();
        let bdaily2 = TimePeriod::from_str("2B").unwrap();
        let bdaily_1 = TimePeriod::from_str("-1B").unwrap();

        assert_eq!("1B", &format!("{}", bdaily1));
        assert_eq!("2B", &format!("{}", bdaily2));
        assert_eq!("-1B", &format!("{}", bdaily_1));

        let date = Date::from_calendar_date(2019, time::Month::November, 20).unwrap();
        assert_eq!(
            bdaily1.add_to(date, Some(&cal)),
            Ok(Date::from_calendar_date(2019, time::Month::November, 22).unwrap())
        );
        assert_eq!(
            bdaily2.add_to(date, Some(&cal)),
            Ok(Date::from_calendar_date(2019, time::Month::November, 25).unwrap())
        );
        assert_eq!(
            bdaily_1.add_to(date, Some(&cal)),
            Ok(Date::from_calendar_date(2019, time::Month::November, 19).unwrap())
        );

        let date = Date::from_calendar_date(2019, time::Month::November, 25).unwrap();
        assert_eq!(
            bdaily1.add_to(date, Some(&cal)),
            Ok(Date::from_calendar_date(2019, time::Month::November, 26).unwrap())
        );
        assert_eq!(
            bdaily2.add_to(date, Some(&cal)),
            Ok(Date::from_calendar_date(2019, time::Month::November, 27).unwrap())
        );
        assert_eq!(
            bdaily_1.add_to(date, Some(&cal)),
            Ok(Date::from_calendar_date(2019, time::Month::November, 22).unwrap())
        );
    }

    #[test]
    fn deserialize_time_period() {
        let input = r#""6M""#;

        let tp: TimePeriod = serde_json::from_str(input).unwrap();
        assert_eq!(tp.num, 6);
        assert_eq!(tp.unit, TimePeriodUnit::Monthly);
        let tpt = TimePeriod {
            num: 6,
            unit: TimePeriodUnit::Monthly,
        };
        assert_eq!(tp, tpt);
    }

    #[test]
    fn serialize_time_period() {
        let tp = TimePeriod {
            num: -2,
            unit: TimePeriodUnit::Annual,
        };
        let json = serde_json::to_string(&tp).unwrap();
        assert_eq!(json, r#""-2Y""#);
    }

    #[test]
    fn operator_add_period() {
        let period_6m = TimePeriod::from_str("6M").unwrap();
        let start = Date::from_calendar_date(2019, time::Month::December, 16).unwrap();
        let end = Date::from_calendar_date(2020, time::Month::June, 16).unwrap();
        assert_eq!(start + period_6m, end);
        assert_eq!(end - period_6m, start);
        let minus_period_6m = -period_6m;
        assert_eq!(end + minus_period_6m, start);
        assert_eq!(start - minus_period_6m, end);
        let mut new_start = Date::from_calendar_date(2019, time::Month::December, 16).unwrap();
        new_start += period_6m;
        assert_eq!(new_start, end);
        let mut new_end = Date::from_calendar_date(2020, time::Month::June, 16).unwrap();
        new_end -= period_6m;
        assert_eq!(start, new_end);
    }
}
