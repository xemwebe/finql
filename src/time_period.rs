//! The module `time_periods` supports time periods of different lengths
//! in terms of day, months or years that can be added to a given date.
//! Time periods may als be negative.

use std::error;
use std::fmt;

use crate::calendar::{last_day_of_month, Calendar};
use chrono::{Datelike, Duration, NaiveDate};

/// Error type related to the TimePeriod trait
#[derive(Debug, Clone)]
pub struct TimePeriodError {
    msg: &'static str,
}

impl fmt::Display for TimePeriodError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TimePeriod error:  {}", self.msg)
    }
}

/// This is important for other errors to wrap this one.
impl error::Error for TimePeriodError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

/// Possible units of a time period
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct TimePeriod {
    num: i32,
    unit: TimePeriodUnit,
}

impl fmt::Display for TimePeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.num, self.unit)
    }
}

/// Transform a string into a TimePeriod
impl TimePeriod {
    pub fn from_str(tp: &str) -> Result<TimePeriod, TimePeriodError> {
        let len = tp.len();
        if len < 2 {
            Err(TimePeriodError {
                msg: "Couldn't parse time period, string is too short.",
            })
        } else {
            let unit = match tp.chars().last() {
                Some('D') => TimePeriodUnit::Daily,
                Some('B') => TimePeriodUnit::BusinessDaily,
                Some('W') => TimePeriodUnit::Weekly,
                Some('M') => TimePeriodUnit::Monthly,
                Some('Y') => TimePeriodUnit::Annual,
                _ => {
                    return Err(TimePeriodError {
                        msg: "Invalid time period unit, use one of 'D', 'B', 'W', 'M', or 'Y'.",
                    })
                }
            };
            let num = match tp[..len - 1].parse::<i32>() {
                Ok(val) => val,
                _ => {
                    return Err(TimePeriodError {
                        msg: "Invalid number of periods.",
                    })
                }
            };

            Ok(TimePeriod { num, unit })
        }
    }

    /// Add time period to a given date.
    /// The function call will panic is the resulting year is out
    /// of the valid range or if not calendar is provided in case of BusinessDaily time periods
    pub fn add_to(&self, mut date: NaiveDate, cal: Option<&Calendar>) -> NaiveDate {
        match self.unit {
            TimePeriodUnit::Daily => date + Duration::days(self.num as i64),
            TimePeriodUnit::BusinessDaily => {
                let is_neg = self.num<0;
                let n = self.num.abs();
                let cal = cal.unwrap();
                for _ in 0..n {
                    date = if is_neg { cal.prev_bday(date) } else { cal.next_bday(date) };
                }
                date
            }
            TimePeriodUnit::Weekly => date
                .checked_add_signed(Duration::days(7 * self.num as i64))
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
                    let last_date_of_month = last_day_of_month(year, month as u32);
                    day = std::cmp::min(day, last_date_of_month);
                }
                NaiveDate::from_ymd(year, month as u32, day)
            }
            TimePeriodUnit::Annual => {
                NaiveDate::from_ymd(date.year() + self.num, date.month(), date.day())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::{Calendar, Holiday};
    use chrono::Weekday;

    #[test]
    fn standard_periods() {
        let date = NaiveDate::from_ymd(2019, 11, 18);
        assert_eq!(
            TimePeriod::from_str("3M").unwrap().add_to(date, None),
            NaiveDate::from_ymd(2020, 2, 18)
        );
        assert_eq!(
            TimePeriod::from_str("1Y").unwrap().add_to(date, None),
            NaiveDate::from_ymd(2020, 11, 18)
        );
        assert_eq!(
            TimePeriod::from_str("6M").unwrap().add_to(date, None),
            NaiveDate::from_ymd(2020, 5, 18)
        );
        assert_eq!(
            TimePeriod::from_str("1W").unwrap().add_to(date, None),
            NaiveDate::from_ymd(2019, 11, 25)
        );

        let date = NaiveDate::from_ymd(2019, 11, 30);
        assert_eq!(
            TimePeriod::from_str("3M").unwrap().add_to(date, None),
            NaiveDate::from_ymd(2020, 2, 29)
        );
        assert_eq!(
            TimePeriod::from_str("1Y").unwrap().add_to(date, None),
            NaiveDate::from_ymd(2020, 11, 30)
        );
        assert_eq!(
            TimePeriod::from_str("6M").unwrap().add_to(date, None),
            NaiveDate::from_ymd(2020, 5, 30)
        );
        assert_eq!(
            TimePeriod::from_str("1W").unwrap().add_to(date, None),
            NaiveDate::from_ymd(2019, 12, 7)
        );
    }

    #[test]
    fn negative_periods() {
        let date = NaiveDate::from_ymd(2019, 11, 18);
        let neg_quarterly = TimePeriod::from_str("-3M").unwrap();
        let neg_annual = TimePeriod::from_str("-1Y").unwrap();
        let neg_weekly = TimePeriod::from_str("-1W").unwrap();
        assert_eq!(
            neg_quarterly.add_to(NaiveDate::from_ymd(2020, 2, 18), None),
            date
        );
        assert_eq!(
            neg_annual.add_to(NaiveDate::from_ymd(2020, 11, 18), None),
            date
        );
        assert_eq!(
            neg_weekly.add_to(NaiveDate::from_ymd(2019, 11, 25), None),
            date
        );

        let date = NaiveDate::from_ymd(2019, 11, 30);
        assert_eq!(
            neg_quarterly.add_to(NaiveDate::from_ymd(2020, 2, 29), None),
            NaiveDate::from_ymd(2019, 11, 29)
        );
        assert_eq!(
            neg_annual.add_to(NaiveDate::from_ymd(2020, 11, 30), None),
            date
        );
        assert_eq!(
            neg_weekly.add_to(NaiveDate::from_ymd(2019, 12, 7), None),
            date
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
            Holiday::SingularDay(NaiveDate::from_ymd(2019, 11, 21)),
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

        let date = NaiveDate::from_ymd(2019, 11, 20);
        assert_eq!(
            bdaily1.add_to(date, Some(&cal)),
            NaiveDate::from_ymd(2019, 11, 22)
        );
        assert_eq!(
            bdaily2.add_to(date, Some(&cal)),
            NaiveDate::from_ymd(2019, 11, 25)
        );
        assert_eq!(
            bdaily_1.add_to(date, Some(&cal)),
            NaiveDate::from_ymd(2019, 11, 19)
        );

        let date = NaiveDate::from_ymd(2019, 11, 25);
        assert_eq!(
            bdaily1.add_to(date, Some(&cal)),
            NaiveDate::from_ymd(2019, 11, 26)
        );
        assert_eq!(
            bdaily2.add_to(date, Some(&cal)),
            NaiveDate::from_ymd(2019, 11, 27)
        );
        assert_eq!(
            bdaily_1.add_to(date, Some(&cal)),
            NaiveDate::from_ymd(2019, 11, 22)
        );
    }
}
