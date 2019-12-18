/// A market is either a container to store market data or
/// an adapter to receive and send market data from an external
/// source, e.g a database, files, or REST service.
/// Market data consist of non-static data, like interest rates,
/// asset prices, or foreign exchange rates.
/// Currently, this is only a stub implementation.
use crate::calendar::{Calendar, Holiday, NthWeek};
use chrono::{NaiveDate, Weekday};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

/// Error related to market data object
#[derive(Debug)]
pub enum MarketError {
    CalendarNotFound,
}

impl fmt::Display for MarketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown calendar")
    }
}

impl Error for MarketError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

/// Container or adapter to market data
pub struct Market {
    calendars: HashMap<String, Calendar>,
}

impl Market {
    /// For now, market data statically generated and stored in memory
    pub fn new() -> Market {
        Market {
            calendars: generate_calendars(),
        }
    }

    /// Get calendar from market
    pub fn get_calendar(&self, name: &str) -> Result<&Calendar, MarketError> {
        if self.calendars.contains_key(name) {
            Ok(&self.calendars[name])
        } else {
            Err(MarketError::CalendarNotFound)
        }
    }
}

/// Generate fixed set of some calendars for testing purposes only
fn generate_calendars() -> HashMap<String, Calendar> {
    let mut calendars = HashMap::new();
    let uk_settlement_holidays = vec![
        // Saturdays
        Holiday::WeekDay(Weekday::Sat),
        // Sundays
        Holiday::WeekDay(Weekday::Sun),
        // New Year's day
        Holiday::MovableYearlyDay {
            month: 1,
            day: 1,
            first: None,
            last: None,
        },
        // Good Friday
        Holiday::EasterOffset {
            offset: -2,
            first: None,
            last: None,
        },
        // Easter Monday
        Holiday::EasterOffset {
            offset: 1,
            first: None,
            last: None,
        },
        // first Monday of May, moved two times in history to 8th of May
        Holiday::MonthWeekday {
            month: 5,
            weekday: Weekday::Mon,
            nth: NthWeek::First,
            first: None,
            last: Some(1994),
        },
        Holiday::SingularDay(NaiveDate::from_ymd(1995, 5, 8)),
        Holiday::MonthWeekday {
            month: 5,
            weekday: Weekday::Mon,
            nth: NthWeek::First,
            first: Some(1996),
            last: Some(2019),
        },
        Holiday::SingularDay(NaiveDate::from_ymd(2020, 5, 8)),
        Holiday::MonthWeekday {
            month: 5,
            weekday: Weekday::Mon,
            nth: NthWeek::First,
            first: Some(2021),
            last: None,
        },
        // last Monday of May (Spring Bank Holiday), has been skipped two times
        Holiday::MonthWeekday {
            month: 5,
            weekday: Weekday::Mon,
            nth: NthWeek::Last,
            first: None,
            last: Some(2001),
        },
        Holiday::MonthWeekday {
            month: 5,
            weekday: Weekday::Mon,
            nth: NthWeek::Last,
            first: Some(2003),
            last: Some(2011),
        },
        Holiday::MonthWeekday {
            month: 5,
            weekday: Weekday::Mon,
            nth: NthWeek::Last,
            first: Some(2013),
            last: None,
        },
        // last Monday of August (Summer Bank Holiday)
        Holiday::MonthWeekday {
            month: 8,
            weekday: Weekday::Mon,
            nth: NthWeek::Last,
            first: None,
            last: None,
        },
        // Christmas
        Holiday::MovableYearlyDay {
            month: 12,
            day: 25,
            first: None,
            last: None,
        },
        // Boxing Day
        Holiday::MovableYearlyDay {
            month: 12,
            day: 26,
            first: None,
            last: None,
        },
        // Golden Jubilee
        Holiday::SingularDay(NaiveDate::from_ymd(2002, 6, 3)),
        // Special Spring Holiday
        Holiday::SingularDay(NaiveDate::from_ymd(2002, 6, 4)),
        // Royal Wedding
        Holiday::SingularDay(NaiveDate::from_ymd(2011, 4, 29)),
        // Diamond Jubilee
        Holiday::SingularDay(NaiveDate::from_ymd(2012, 6, 4)),
        // Special Spring Holiday
        Holiday::SingularDay(NaiveDate::from_ymd(2012, 6, 5)),
        // Introduction of EUR
        Holiday::SingularDay(NaiveDate::from_ymd(1999, 12, 31)),
    ];
    let uk_cal = Calendar::calc_calendar(&uk_settlement_holidays, 1990, 2050);
    calendars.insert("uk".to_string(), uk_cal);

    let target_holidays = vec![
        // Saturdays
        Holiday::WeekDay(Weekday::Sat),
        // Sundays
        Holiday::WeekDay(Weekday::Sun),
        // New Year's day
        Holiday::YearlyDay {
            month: 1,
            day: 1,
            first: None,
            last: None,
        },
        // Good Friday
        Holiday::EasterOffset {
            offset: -2,
            first: Some(2000),
            last: None,
        },
        // Easter Monday
        Holiday::EasterOffset {
            offset: 1,
            first: Some(2000),
            last: None,
        },
        // Labour Day
        Holiday::YearlyDay {
            month: 5,
            day: 1,
            first: Some(2000),
            last: None,
        },
        // Labour Day
        Holiday::YearlyDay {
            month: 5,
            day: 1,
            first: Some(2000),
            last: None,
        },
        // Labour Day
        Holiday::YearlyDay {
            month: 5,
            day: 1,
            first: Some(2000),
            last: None,
        },
        Holiday::SingularDay(NaiveDate::from_ymd(1995, 5, 8)),
    ];
    let target_cal = Calendar::calc_calendar(&target_holidays, 1990, 2050);
    calendars.insert("TARGET".to_string(), target_cal);

    calendars
}
