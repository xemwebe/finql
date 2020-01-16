///! Useful helper functions that do not belong to any other module

use chrono::{DateTime,Utc,Local,NaiveDateTime, NaiveDate};
use chrono::offset::TimeZone;

/// Given a date and time construct a UTC DateTime, assuming that 
/// the date belongs to local time zone
pub fn make_time(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> Option<DateTime<Utc>> {
    let time: NaiveDateTime = NaiveDate::from_ymd(year, month, day).and_hms(hour, minute, second);
    let time = Local.from_local_datetime(&time).single();
    match time {
        Some(time) => Some(DateTime::from(time)),
        None => None,
    }
}
