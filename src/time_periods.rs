//! The module `time_periods` supports time periods of different lengths 
//! in terms of day, months or years that can be added to a given date.
//! Time periods may als be negative.

use std::fmt;
use chrono::{NaiveDate,Datelike,Duration};

pub trait TimePeriod {
    // Add time period to a given date. 
    // The function call will panic is the resulting year is out
    // of the valid range.
    fn add_to(&self, date: NaiveDate) -> NaiveDate;
}

// Some common time periods
pub static QUARTERLY: &'static MonthlyPeriod = &MonthlyPeriod{years:0, months:3};
pub static SEMI_ANNUAL: &'static MonthlyPeriod = &MonthlyPeriod{years:0, months:6};
pub static ANNUAL: &'static YearlyPeriod = &YearlyPeriod{years:1};
pub static WEEKLY: &'static DailyPeriod = &DailyPeriod{days: 7};

pub struct DailyPeriod {
    days: i32
}

impl DailyPeriod {
    pub fn new(periods: i32) -> DailyPeriod {
        DailyPeriod{ days: periods }
    }
}

impl TimePeriod for DailyPeriod {
    fn add_to(&self, date: NaiveDate) -> NaiveDate {
        date.checked_add_signed(Duration::days(self.days as i64)).unwrap()
    }
}

impl fmt::Display for DailyPeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.days%7 == 0 {
            write!(f,"{}W",self.days/7)
        } else {
            write!(f, "{}D", self.days)
        }
    }
}

pub struct YearlyPeriod {
    years: i32
}

impl YearlyPeriod {
    pub fn new(periods: i32) -> YearlyPeriod {
        YearlyPeriod{ years: periods }
    }
}

impl TimePeriod for YearlyPeriod {
    fn add_to(&self, date: NaiveDate) -> NaiveDate {
        NaiveDate::from_ymd(date.year()+self.years, date.month(), date.day())
    }
}

impl fmt::Display for YearlyPeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}Y",self.years)
    }
}

// If the original day of the data is larger than the length
// of the target month, the day is moved to the last day of the target month.
// Therefore, `MonthlyPeriod` is not in all cases reversible by adding
// the equivalent negative monthly period. 
pub struct MonthlyPeriod {
    years: i32,
    months: i32
}

impl MonthlyPeriod {
    pub fn new(periods: i32) -> MonthlyPeriod {
        MonthlyPeriod{ years: periods/12, months: periods%12 }
    }
}

impl TimePeriod for MonthlyPeriod {
    fn add_to(&self, date: NaiveDate) -> NaiveDate {
        let mut day = date.day();
        let mut month = date.month() as i32;
        let mut year = date.year();
        year += self.years;
        month += self.months;
        if month < 1 {
            year -= 1;
            month += 12;
        } else if month > 12{
            year += 1;
            month -= 12;
        }
        if day>28 {
            let last_date_of_month = get_days_from_month(year, month as u32);
            day = std::cmp::min(day, last_date_of_month);
        }
        NaiveDate::from_ymd(year, month as u32, day)
    }
}

impl fmt::Display for MonthlyPeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}M",self.years*12+self.months)
    }
} 

fn get_days_from_month(year: i32, month: u32) -> u32 {
    NaiveDate::from_ymd(
        match month {
            12 => year + 1,
            _ => year,
        },
        match month {
            12 => 1,
            _ => month + 1,
        },
        1,
    )
    .signed_duration_since(NaiveDate::from_ymd(year, month, 1))
    .num_days() as u32
    
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply_time_period(date: NaiveDate, tp: &dyn TimePeriod) -> NaiveDate {
        tp.add_to(date)
    }

    #[test]
    fn standard_periods() {
        let date = NaiveDate::from_ymd(2019,11,18);
        assert_eq!(apply_time_period(date, QUARTERLY), NaiveDate::from_ymd(2020,2,18));
        assert_eq!(apply_time_period(date, ANNUAL), NaiveDate::from_ymd(2020,11,18));
        assert_eq!(apply_time_period(date, SEMI_ANNUAL), NaiveDate::from_ymd(2020,5,18));
        assert_eq!(apply_time_period(date, WEEKLY), NaiveDate::from_ymd(2019,11,25));
        
        let date = NaiveDate::from_ymd(2019,11,30);
        assert_eq!(apply_time_period(date, QUARTERLY), NaiveDate::from_ymd(2020,2,29));
        assert_eq!(apply_time_period(date, ANNUAL), NaiveDate::from_ymd(2020,11,30));
        assert_eq!(apply_time_period(date, SEMI_ANNUAL), NaiveDate::from_ymd(2020,5,30));
        assert_eq!(apply_time_period(date, WEEKLY), NaiveDate::from_ymd(2019,12,7));
    }

    #[test]
    fn negative_periods() {
        let date = NaiveDate::from_ymd(2019,11,18);
        let neg_quarterly = MonthlyPeriod::new(-3);
        let neg_annual = YearlyPeriod::new(-1);
        let neg_weekly = DailyPeriod::new(-7);
        assert_eq!(apply_time_period(NaiveDate::from_ymd(2020,2,18), &neg_quarterly), date);
        assert_eq!(apply_time_period(NaiveDate::from_ymd(2020,11,18), &neg_annual), date);
        assert_eq!(apply_time_period(NaiveDate::from_ymd(2019,11,25), &neg_weekly), date);
        
        let date = NaiveDate::from_ymd(2019,11,30);
        assert_eq!(apply_time_period(NaiveDate::from_ymd(2020,2,29), &neg_quarterly), NaiveDate::from_ymd(2019,11,29));
        assert_eq!(apply_time_period(NaiveDate::from_ymd(2020,11,30), &neg_annual), date );
        assert_eq!(apply_time_period(NaiveDate::from_ymd(2019,12,7), &neg_weekly), date);
    }

    #[test]
    fn display_periods() {
        let daily = DailyPeriod::new(1);
        let neg_daily = DailyPeriod::new(-1);
        let neg_quarterly = MonthlyPeriod::new(-3);
        let neg_annual = YearlyPeriod::new(-1);
        let neg_weekly = DailyPeriod::new(-7);
        assert_eq!(format!("{}",QUARTERLY), "3M");
        assert_eq!(format!("{}",SEMI_ANNUAL), "6M");
        assert_eq!(format!("{}",ANNUAL), "1Y");
        assert_eq!(format!("{}",WEEKLY), "1W");
        assert_eq!(format!("{}",daily), "1D");
        assert_eq!(format!("{}",neg_quarterly), "-3M");
        assert_eq!(format!("{}",neg_annual), "-1Y");
        assert_eq!(format!("{}",neg_weekly), "-1W");
        assert_eq!(format!("{}",neg_daily), "-1D");
    }
}
