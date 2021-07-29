use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};
use std::time::{Duration, UNIX_EPOCH};


/// Convert NaiveDate to DateTime at the given hour, assuming local time zone 
pub fn naive_date_to_date_time(
    date: &NaiveDate,
    hour: u32
) -> DateTime<Utc> {
    let time = date.and_hms_milli(hour, 0, 0, 0);
    let time = Local.from_local_datetime(&time).single().unwrap();
    DateTime::from(time)
}

/// Create UTC time set is given as UNIX epoch timestamp (i.e seconds since 1st Jan 1970)
pub fn unix_to_date_time(seconds: u64) -> DateTime<Utc> {
    // Creates a new SystemTime from the specified number of whole seconds
    let d = UNIX_EPOCH + Duration::from_secs(seconds);
    // Create DateTime from SystemTime
    DateTime::<Utc>::from(d)
}

/// Create UTC time from NaiveDate string
/// The following assumptions are made:
/// 0. Date is given in the format American weird format `%m-%d-%Y`
/// 1. Date is the date in local time zone
/// 2. Hour is set the given hour parameter
/// 3. Minutes, seconds and milliseconds are set to zero
pub fn date_time_from_str_american(
    date_str: &str,
    hour: u32,
) -> Result<DateTime<Utc>, chrono::format::ParseError> {
    date_time_from_str(date_str, "%m-%d-%Y", hour)
}

/// Create UTC time from NaiveDate string
/// The following assumptions are made:
/// 0. Date is given in the format `%Y-%m-%d`
/// 1. Date is the date in local time zone
/// 2. Hour is set the given hour parameter
/// 3. Minutes, seconds and milliseconds are set to zero
pub fn date_time_from_str_standard(
    date_str: &str,
    hour: u32,
) -> Result<DateTime<Utc>, chrono::format::ParseError> {
    date_time_from_str(date_str, "%F", hour)
}

/// Create UTC time from NaiveDate string
/// The following assumptions are made:
/// 0. Date is given in the provided format
/// 1. Date is the date in local time zone
/// 2. Hour is set the given hour parameter
/// 3. Minutes, seconds and milliseconds are set to zero
pub fn date_time_from_str(
    date_str: &str,
    format: &str,
    hour: u32,
) -> Result<DateTime<Utc>, chrono::format::ParseError> {
    let date = NaiveDate::parse_from_str(date_str, format)?;
    Ok(naive_date_to_date_time(&date, hour))
}

/// Create UTC time from NaiveDate string
/// The following assumptions are made:
/// 0. Date is given in the provided format
/// 1. Date is the date in local time zone
/// 2. Hour is set the given hour parameter
/// 3. Minutes, seconds and milliseconds are set to zero
pub fn date_from_str(
    date_str: &str,
    format: &str
) -> Result<NaiveDate, chrono::format::ParseError> {
   NaiveDate::parse_from_str(date_str, format)
}

/// Convert string with added time zone (by default 0) to DateTime<Utc>
pub fn to_time(time: &str, zone: i32) -> Result<DateTime<Utc>, chrono::format::ParseError> {
    // sqlx strips time zone, just add it here again
    let time = format!("{}{:+05}",time, zone);
    let time =
        DateTime::parse_from_str(&time,"%Y-%m-%d %H:%M:%S%.3f%z")?;
    let time: DateTime<Utc> = DateTime::from(time);
    Ok(time)
}

/// Given a date and time construct a UTC DateTime, assuming that
/// the date belongs to local time zone
pub fn make_time(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> Option<DateTime<Utc>> {
    let time: NaiveDateTime = NaiveDate::from_ymd(year, month, day).and_hms(hour, minute, second);
    let time = Local.from_local_datetime(&time).single();
    match time {
        Some(time) => Some(DateTime::from(time)),
        None => None,
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unix_to_date_time() {
        let date = unix_to_date_time(1587099600);
        let date_string = date.format("%Y-%m-%d %H:%M:%S").to_string();
        assert_eq!("2020-04-17 05:00:00", &date_string);
    }

    #[test]
    fn test_date_time_from_str_american() {
        let date = date_time_from_str_american("02-10-2020", 18).unwrap();
        let date: DateTime<Local> = DateTime::from(date);
        let date_string = date.format("%Y-%m-%d %H:%M:%S").to_string();
        assert_eq!("2020-02-10 18:00:00", &date_string);
    }

    #[test]
    fn test_date_date_time_from_str_standard() {
        let date = date_time_from_str_standard("2020-02-10", 18).unwrap();
        let date: DateTime<Local> = DateTime::from(date);
        let date_string = date.format("%Y-%m-%d %H:%M:%S").to_string();
        assert_eq!("2020-02-10 18:00:00", &date_string);
    }

    #[test]
    fn test_date_time_from_str() {
        let date = date_time_from_str("10-2020-02", "%d-%Y-%m", 18).unwrap();
        let date: DateTime<Local> = DateTime::from(date);
        let date_string = date.format("%Y-%m-%d %H:%M:%S").to_string();
        assert_eq!("2020-02-10 18:00:00", &date_string);
    }
}
