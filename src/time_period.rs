//! The module `time_periods` supports time periods of different lengths 
//! in terms of day, months or years that can be added to a given date.
//! Time periods may als be negative.

use std::fmt;
use std::error;

use chrono::{NaiveDate,Datelike,Duration};
use crate::calendar::{Calendar,SimpleCalendar};

// Error type related to the TimePeriod trait
#[derive(Debug, Clone)]
pub struct TimePeriodError {
    msg: &'static str
}

impl fmt::Display for TimePeriodError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TimePeriod error:  {}", self.msg)
    }
}

// This is important for other errors to wrap this one.
impl error::Error for TimePeriodError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

pub trait TimePeriod: fmt::Display {
    // Add time period to a given date. 
    // The function call will panic is the resulting year is out
    // of the valid range.
    fn add_to(&self, date: NaiveDate) -> NaiveDate;
}

// Transform a string into a TimePeriod
pub fn from_str(tp: &str) -> Result<Box<dyn TimePeriod>,TimePeriodError> {
    let len = tp.len();
    if len<2 {
        Err(TimePeriodError{ msg: "Couldn't parse time period, string is too short." })
    } else {
        let tpid = tp.chars().last().unwrap();
        let num = tp[..len-1].parse::<i32>();
        match (num, tpid) {
            (Ok(val), 'D') => Ok(Box::new(DailyPeriod::new(val))),
            (Ok(val), 'W') => Ok(Box::new(DailyPeriod::new(7*val))),
            (Ok(val), 'M') => Ok(Box::new(MonthlyPeriod::new(val))),
            (Ok(val), 'Y') => Ok(Box::new(YearlyPeriod::new(val))),
            (_, _) => Err(TimePeriodError{ msg: "Invalid format, use <signed num><unit> where <unit> is one of 'D', 'W', 'M', or 'Y'."})
        }
    }
}

// Some common time periods
pub static QUARTERLY: &'static MonthlyPeriod = &MonthlyPeriod{years:0, months:3};
pub static SEMI_ANNUAL: &'static MonthlyPeriod = &MonthlyPeriod{years:0, months:6};
pub static ANNUAL: &'static YearlyPeriod = &YearlyPeriod{years:1};
pub static WEEKLY: &'static DailyPeriod = &DailyPeriod{days: 7};

#[derive(Debug, Clone)]
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

#[derive(Debug)]
pub struct BusinessDailyPeriod {
    days: i32,
    cal: SimpleCalendar
}

impl BusinessDailyPeriod {
    pub fn new(periods: i32) -> DailyPeriod {
        DailyPeriod{ days: periods }
    }
}

impl TimePeriod for BusinessDailyPeriod {
    fn add_to(&self, mut date: NaiveDate) -> NaiveDate {
        let inc = if self.days<0 {
            Duration::days(-1)
        } else {
            Duration::days(1)
        };
        let n = self.days.abs();
        for _ in 0..n {
            date = date.checked_add_signed(inc).unwrap();
            while self.cal.is_holiday(date) {
                date = date.checked_add_signed(inc).unwrap();
            }
        }
        date
    }
}

impl fmt::Display for BusinessDailyPeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}B", self.days)
    }
}


#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
    use crate::calendar::SimpleCalendar;

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

    #[test]
    fn parse_periods() {
        assert_eq!("3M", &format!("{}",from_str("3M").unwrap()));
        assert_eq!("-3M", &format!("{}",from_str("-3M").unwrap()));
        assert_eq!("18M", &format!("{}",from_str("18M").unwrap()));
        assert_eq!("-18M", &format!("{}",from_str("-18M").unwrap()));
        assert_eq!("1D", &format!("{}",from_str("1D").unwrap()));
        assert_eq!("-1D", &format!("{}",from_str("-1D").unwrap()));
        assert_eq!("2W", &format!("{}",from_str("2W").unwrap()));
        assert_eq!("-3W", &format!("{}",from_str("-3W").unwrap()));
        assert_eq!("5Y", &format!("{}",from_str("5Y").unwrap()));
        assert_eq!("-10Y", &format!("{}",from_str("-10Y").unwrap()));
    }


    #[test]
    fn parse_business_daily() {
        let cal = SimpleCalendar::new(vec![NaiveDate::from_ymd(2019,11,21)]);
        let bdaily1 =  BusinessDailyPeriod{ days: 1, cal: cal };
        //let bdaily2 = BusinessDailyPeriod{ days:2, cal: cal };
        //let bdaily_1 = BusinessDailyPeriod{ days:-1, cal: cal };
      
        assert_eq!("1B", &format!("{}",bdaily1));
        //assert_eq!("2B", &format!("{}",bdaily1));
        //assert_eq!("-1B", &format!("{}",bdaily1));
        
        let date = NaiveDate::from_ymd(2019,11,20);
        assert_eq!(apply_time_period(date, &bdaily1), NaiveDate::from_ymd(2019,11,22));
        //assert_eq!(apply_time_period(date, &bdaily2), NaiveDate::from_ymd(2019,11,25));
        //assert_eq!(apply_time_period(date, &bdaily_1), NaiveDate::from_ymd(2020,11,19));
        
        let date = NaiveDate::from_ymd(2019,11,25);
        assert_eq!(apply_time_period(date, &bdaily1), NaiveDate::from_ymd(2019,11,26));
        //assert_eq!(apply_time_period(date, &bdaily2), NaiveDate::from_ymd(2019,11,27));
        //assert_eq!(apply_time_period(date, &bdaily_1), NaiveDate::from_ymd(2020,11,22));
    }
}
