//! Implementation of (bank) holidays.
//! This is required to verify whether an exchange is open, a specific
//! date is a calendar, or to calculate the amount of business days between to dates.
//! Bank holidays have also an impact on how to rollout cash flows from fixed income
//! products, which has an impact on their fair value.

use chrono::{NaiveDate,Duration,Weekday,Datelike};
use std::collections::BTreeSet;

// Trait specifying (bank) holidays
pub trait Calendar {
    // Returns true if given day is a bank holiday
    fn is_holiday(&self, date: NaiveDate) -> bool;
    // Calculate the next business day
    fn next_bday(&self, mut date: NaiveDate) -> NaiveDate {
        while self.is_holiday(date) {
            date = date + Duration::days(1);
        }
        date
    }
    // Calculate the next business day
    fn prev_bday(&self, mut date: NaiveDate) -> NaiveDate {
        while self.is_holiday(date) {
            date = date + Duration::days(1);
        }
        date
    }
    // Pre-compute holidays for a given range of years
    fn compute_holidays(&self, start_year: i32, end_year: i32);
}

// Simple calendar implementation as a set of given dates
#[derive(Debug, Clone)]
pub struct SimpleCalendar {
    holidays: BTreeSet<NaiveDate>
}

impl SimpleCalendar {
    // Construct new calendar and automatically sort dates given
    // as a vector
    pub fn new(days: Vec<NaiveDate>) -> SimpleCalendar {
        let mut holidays = BTreeSet::new();
        for d in days {
            holidays.insert(d);
        }
        SimpleCalendar{ holidays: holidays }
    }
    // Nothing to do here for `SimpleCalendar` 
    pub fn compute_holidays(&self, _: i32, _: i32) {}

    fn is_weekend(day: &NaiveDate) -> bool {
        if day.weekday() == Weekday::Sat || day.weekday() == Weekday::Sun {
            true 
        } else {
            false
        }
    }
}

impl Calendar for SimpleCalendar {
    // Nothing to do here for `SimpleCalendar` 
    fn compute_holidays(&self, _: i32, _: i32) {}
    // Just check if the date is in our set of holidays
    fn is_holiday(&self, date: NaiveDate) -> bool {
        if Self::is_weekend(&date) {
            true
        } else {
            match self.holidays.get(&date) {
                Some(_) => true,
                None => false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_calendar() {
        let days = vec![
            NaiveDate::from_ymd(2019,11,20),
            NaiveDate::from_ymd(2019,11,24),
            NaiveDate::from_ymd(2019,11,25),
            ];
        let cal = SimpleCalendar::new( days);
        assert_eq!(true, cal.is_holiday(NaiveDate::from_ymd(2019,11,20)));
        assert_eq!(false, cal.is_holiday(NaiveDate::from_ymd(2019,11,21)));
        assert_eq!(false, cal.is_holiday(NaiveDate::from_ymd(2019,11,22)));
        // weekend
        assert_eq!(true, cal.is_holiday(NaiveDate::from_ymd(2019,11,23)));
        // weekend and holiday
        assert_eq!(true, cal.is_holiday(NaiveDate::from_ymd(2019,11,24)));
        assert_eq!(true, cal.is_holiday(NaiveDate::from_ymd(2019,11,25)));
        assert_eq!(false, cal.is_holiday(NaiveDate::from_ymd(2019,11,26)));
    }

}