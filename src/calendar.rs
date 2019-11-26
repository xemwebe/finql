//! Implementation of (bank) holidays.
//! This is required to verify whether an exchange is open, a specific
//! date is a calendar, or to calculate the amount of business days between to dates.
//! Bank holidays have also an impact on how to rollout cash flows from fixed income
//! products, which has an impact on their fair value.

use chrono::{Datelike, Duration, NaiveDate, Weekday};
use std::collections::BTreeSet;
extern crate computus;

pub enum Holiday {
    /// A week day that is a bank holiday, in most countries, this will be Sat and Sun
    WeekDay(Weekday),
    /// `start` and `end` are the first and last year this day is a holiday
    YearlyDay{month: u32, day: u32, first: Option<i32>, last: Option<i32>},
    /// Occurs every year, but is moved to next non-weekend day if it falls on a weekday
    /// `start` and `end` are the first and last year this day is a holiday
    MovableYearlyDay{month: u32, day: u32, first: Option<i32>, last: Option<i32>},
    /// A single holiday valid only once in time
    SingularDay(NaiveDate),
    /// A holiday that is define relative to Easter Monday
    EasterOffset(i32),
}

// Calendar for arbitrary complex holiday rules
#[derive(Debug, Clone)]
pub struct Calendar {
    holidays: BTreeSet<NaiveDate>,
    weekdays: Vec<Weekday>,
}

impl Calendar {
    // Pre-compute holidays for a given range of years based on a set of holiday rules
    pub fn calc_calendar(holiday_rules: &Vec<Holiday>, start: i32, end: i32) -> Calendar {
        let mut holidays = BTreeSet::new();
        let mut weekdays = Vec::new();
       
        for rule in holiday_rules {
            match rule {
                Holiday::SingularDay(date) => {
                    let year = date.year();
                    if year >= start && year <= end {
                        holidays.insert(date.clone());
                    }
                },
                Holiday::WeekDay(weekday) => {
                    weekdays.push(weekday.clone());
                },
                Holiday::YearlyDay{month, day, first, last} => {
                    let (first,last) = Self::calc_first_and_last(start, end, first, last);
                    for year in first..last+1 {
                        holidays.insert(NaiveDate::from_ymd(year, *month, *day));
                    }
                },
                Holiday::MovableYearlyDay{month, day, first, last} => {
                    let (first,last) = Self::calc_first_and_last(start, end, first, last);
                    for year in first..last+1 {
                        let date = NaiveDate::from_ymd(year, *month, *day);
                        let date = match date.weekday() {
                            Weekday::Sat => date.checked_add_signed(Duration::days(2)).unwrap(),
                            Weekday::Sun => date.checked_add_signed(Duration::days(1)).unwrap(),
                            _ => date
                        };
                        holidays.insert(date);
                    }
                },
                Holiday::EasterOffset(offset) => {
                    for year in start..end+1 {
                        let easter = computus::gregorian(year).unwrap();
                        let easter = NaiveDate::from_ymd(easter.year, easter.month, easter.day);
                        let date = easter.checked_add_signed(Duration::days(*offset as i64)).unwrap();
                        holidays.insert(date);
                    }
                },
            }
        }
        Calendar{holidays: holidays, weekdays: weekdays}
    }

        /// Calculate the next business day
        pub fn next_bday(&self, mut date: NaiveDate) -> NaiveDate {
            while self.is_holiday(date) {
                date = date + Duration::days(1);
            }
            date
        }

        /// Calculate the next business day
        pub fn prev_bday(&self, mut date: NaiveDate) -> NaiveDate {
            while self.is_holiday(date) {
                date = date + Duration::days(1);
            }
            date
        }
    
    fn calc_first_and_last(start: i32, end: i32, first: &Option<i32>, last: &Option<i32>) -> (i32,i32) {
        let first = match first {
            Some(year) => std::cmp::max(start, *year),
            _ => start
        };
        let last = match last {
            Some(year) => std::cmp::min(end, *year),
            _ => end
        };
        (first, last)
    }

    fn is_weekend(&self, day: &NaiveDate) -> bool {
        let weekday = day.weekday();
        for w_day in &self.weekdays {
            if weekday == *w_day { return true; }
        }
        false
    }

    /// Check for weekends and if date is in precomputed set of holidays
    pub fn is_holiday(&self, date: NaiveDate) -> bool {
        if self.is_weekend(&date) {
            true
        } else {
            match self.holidays.get(&date) {
                Some(_) => true,
                None => false,
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_dates_calendar() {
        let holidays = vec![
            Holiday::SingularDay(NaiveDate::from_ymd(2019, 11, 20)),
            Holiday::SingularDay(NaiveDate::from_ymd(2019, 11, 24)),
            Holiday::SingularDay(NaiveDate::from_ymd(2019, 11, 25)),
            Holiday::WeekDay(Weekday::Sat),
            Holiday::WeekDay(Weekday::Sun),           
        ];
        let cal = Calendar::calc_calendar(&holidays, 2019, 2019);

        assert_eq!(true, cal.is_holiday(NaiveDate::from_ymd(2019, 11, 20)));
        assert_eq!(false, cal.is_holiday(NaiveDate::from_ymd(2019, 11, 21)));
        assert_eq!(false, cal.is_holiday(NaiveDate::from_ymd(2019, 11, 22)));
        // weekend
        assert_eq!(true, cal.is_holiday(NaiveDate::from_ymd(2019, 11, 23)));
        // weekend and holiday
        assert_eq!(true, cal.is_holiday(NaiveDate::from_ymd(2019, 11, 24)));
        assert_eq!(true, cal.is_holiday(NaiveDate::from_ymd(2019, 11, 25)));
        assert_eq!(false, cal.is_holiday(NaiveDate::from_ymd(2019, 11, 26)));
    }
}
