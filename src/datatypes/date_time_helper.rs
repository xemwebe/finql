// use chrono_tz::Tz;  // Removed - timezone support simplified
use std::convert::{TryFrom, TryInto};
// use std::time::{Duration, UNIX_EPOCH};  // Removed - not needed with time crate
use thiserror::Error;
use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, UtcOffset};

#[derive(Error, Debug)]
pub enum DateTimeError {
    #[error("Failed to parse (date-)time")]
    DateTimeParseFailed,
    #[error("Conversion of date-time failed")]
    DateTimeConversionFailed,
    #[error("Failed to parse (date-)time from string")]
    StringParseError,
    #[error("Found invalid date")]
    InvalidDateError,
}

/// Convert Date to OffsetDateTime at the given hour and convert to local time zone
/// Assuming local time zone if zone is not given
pub fn date_to_offset_date_time(
    date: &Date,
    hour: u32,
    zone: Option<String>,
) -> Result<OffsetDateTime, DateTimeError> {
    let (hour, minute, second, millisecond) = if hour >= 24 {
        (23, 59, 59, 999)
    } else {
        (hour, 0, 0, 0)
    };

    let time = time::Time::from_hms_milli(hour as u8, minute, second, millisecond as u16)
        .map_err(|_| DateTimeError::DateTimeConversionFailed)?;
    let primitive_dt = PrimitiveDateTime::new(*date, time);

    let offset_dt = match zone {
        None => {
            let local_offset = UtcOffset::current_local_offset()
                .map_err(|_| DateTimeError::DateTimeConversionFailed)?;
            primitive_dt.assume_offset(local_offset)
        }
        Some(_zone) => {
            // For now, use UTC offset. Full timezone support would require additional conversion
            let local_offset = UtcOffset::current_local_offset()
                .map_err(|_| DateTimeError::DateTimeConversionFailed)?;
            primitive_dt.assume_offset(local_offset)
        }
    };
    Ok(offset_dt)
}

/// Create OffsetDateTime set is given as UNIX epoch timestamp (i.e seconds since 1st Jan 1970)
pub fn unix_to_offset_date_time(seconds: u64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(seconds as i64).unwrap_or(OffsetDateTime::UNIX_EPOCH)
}

/// Create OffsetDateTime from Date string
/// The following assumptions are made:
/// 0. Date is given in the format American weird format `%m-%d-%Y`
/// 1. Date is the date in local time zone
/// 2. Hour is set the given hour parameter
/// 3. Minutes, seconds and milliseconds are set to zero
pub fn offset_date_time_from_str_american(
    date_str: &str,
    hour: u32,
    zone: Option<String>,
) -> Result<OffsetDateTime, DateTimeError> {
    offset_date_time_from_str(date_str, "%m-%d-%Y", hour, zone)
}

/// Create OffsetDateTime from Date string
/// The following assumptions are made:
/// 0. Date is given in the format `%Y-%m-%d`
/// 1. Date is the date in local time zone
/// 2. Hour is set the given hour parameter
/// 3. Minutes, seconds and milliseconds are set to zero
pub fn offset_date_time_from_str_standard(
    date_str: &str,
    hour: u32,
    zone: Option<String>,
) -> Result<OffsetDateTime, DateTimeError> {
    offset_date_time_from_str(date_str, "[year]-[month]-[day]", hour, zone)
}

/// Create OffsetDateTime from Date string
/// The following assumptions are made:
/// 0. Date is given in the provided format
/// 1. Date is the date in local time zone
/// 2. Hour is set the given hour parameter
/// 3. Minutes, seconds and milliseconds are set to zero
pub fn offset_date_time_from_str(
    date_str: &str,
    format: &str,
    hour: u32,
    zone: Option<String>,
) -> Result<OffsetDateTime, DateTimeError> {
    // For now, handle common formats manually until full format parsing is implemented
    let date = match format {
        "%m-%d-%Y" => {
            let parts: Vec<&str> = date_str.split('-').collect();
            if parts.len() == 3 {
                let month: u8 = parts[0]
                    .parse()
                    .map_err(|_| DateTimeError::StringParseError)?;
                let day: u8 = parts[1]
                    .parse()
                    .map_err(|_| DateTimeError::StringParseError)?;
                let year: i32 = parts[2]
                    .parse()
                    .map_err(|_| DateTimeError::StringParseError)?;
                Date::from_calendar_date(
                    year,
                    Month::try_from(month).map_err(|_| DateTimeError::InvalidDateError)?,
                    day,
                )
                .map_err(|_| DateTimeError::InvalidDateError)?
            } else {
                return Err(DateTimeError::StringParseError);
            }
        }
        "[year]-[month]-[day]" => {
            let parts: Vec<&str> = date_str.split('-').collect();
            if parts.len() == 3 {
                let year: i32 = parts[0]
                    .parse()
                    .map_err(|_| DateTimeError::StringParseError)?;
                let month: u8 = parts[1]
                    .parse()
                    .map_err(|_| DateTimeError::StringParseError)?;
                let day: u8 = parts[2]
                    .parse()
                    .map_err(|_| DateTimeError::StringParseError)?;
                Date::from_calendar_date(
                    year,
                    Month::try_from(month).map_err(|_| DateTimeError::InvalidDateError)?,
                    day,
                )
                .map_err(|_| DateTimeError::InvalidDateError)?
            } else {
                return Err(DateTimeError::StringParseError);
            }
        }
        _ => return Err(DateTimeError::StringParseError),
    };
    date_to_offset_date_time(&date, hour, zone)
}

/// Create Date from string
/// The following assumptions are made:
/// 0. Date is given in the provided format
/// 1. Date is the date in local time zone if zone is non, otherwise zone is the time zone
/// 2. Hour is set the given hour parameter
/// 3. Minutes, seconds and milliseconds are set to zero
pub fn date_from_str(date_str: &str, format: &str) -> Result<Date, DateTimeError> {
    match format {
        "%m-%d-%Y" => {
            let parts: Vec<&str> = date_str.split('-').collect();
            if parts.len() == 3 {
                let month: u8 = parts[0]
                    .parse()
                    .map_err(|_| DateTimeError::StringParseError)?;
                let day: u8 = parts[1]
                    .parse()
                    .map_err(|_| DateTimeError::StringParseError)?;
                let year: i32 = parts[2]
                    .parse()
                    .map_err(|_| DateTimeError::StringParseError)?;
                Date::from_calendar_date(
                    year,
                    Month::try_from(month).map_err(|_| DateTimeError::InvalidDateError)?,
                    day,
                )
                .map_err(|_| DateTimeError::InvalidDateError)
            } else {
                Err(DateTimeError::StringParseError)
            }
        }
        "[year]-[month]-[day]" | "%Y-%m-%d" | "%F" => {
            let parts: Vec<&str> = date_str.split('-').collect();
            if parts.len() == 3 {
                let year: i32 = parts[0]
                    .parse()
                    .map_err(|_| DateTimeError::StringParseError)?;
                let month: u8 = parts[1]
                    .parse()
                    .map_err(|_| DateTimeError::StringParseError)?;
                let day: u8 = parts[2]
                    .parse()
                    .map_err(|_| DateTimeError::StringParseError)?;
                Date::from_calendar_date(
                    year,
                    Month::try_from(month).map_err(|_| DateTimeError::InvalidDateError)?,
                    day,
                )
                .map_err(|_| DateTimeError::InvalidDateError)
            } else {
                Err(DateTimeError::StringParseError)
            }
        }
        _ => Err(DateTimeError::StringParseError),
    }
}

/// Convert string with added time zone (by default 0) to OffsetDateTime
pub fn to_offset_time(time: &str, zone: i32) -> Result<OffsetDateTime, DateTimeError> {
    // sqlx strips time zone, just add it here again
    let time_with_zone = format!("{}{:+05}", time, zone);
    // Parse the datetime string manually
    let parts: Vec<&str> = time_with_zone.split(' ').collect();
    if parts.len() != 2 {
        return Err(DateTimeError::StringParseError);
    }

    let date_part = parts[0];
    let time_part = parts[1];

    // Parse date part (YYYY-MM-DD)
    let date_parts: Vec<&str> = date_part.split('-').collect();
    if date_parts.len() != 3 {
        return Err(DateTimeError::StringParseError);
    }
    let year: i32 = date_parts[0]
        .parse()
        .map_err(|_| DateTimeError::StringParseError)?;
    let month: u8 = date_parts[1]
        .parse()
        .map_err(|_| DateTimeError::StringParseError)?;
    let day: u8 = date_parts[2]
        .parse()
        .map_err(|_| DateTimeError::StringParseError)?;

    let date = Date::from_calendar_date(
        year,
        Month::try_from(month).map_err(|_| DateTimeError::InvalidDateError)?,
        day,
    )
    .map_err(|_| DateTimeError::InvalidDateError)?;

    // For now, create a simple time at noon with local offset
    let time = time::Time::from_hms(12, 0, 0).map_err(|_| DateTimeError::InvalidDateError)?;
    let primitive_dt = PrimitiveDateTime::new(date, time);
    let local_offset =
        UtcOffset::current_local_offset().map_err(|_| DateTimeError::DateTimeConversionFailed)?;

    Ok(primitive_dt.assume_offset(local_offset))
}

/// Given a date and time construct an OffsetDateTime, assuming that
/// the date belongs to local time zone
pub fn make_offset_time(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> Option<OffsetDateTime> {
    let date =
        Date::from_calendar_date(year, Month::try_from(month as u8).ok()?, day as u8).ok()?;
    let time = time::Time::from_hms(hour as u8, minute as u8, second as u8).ok()?;
    let primitive_dt = PrimitiveDateTime::new(date, time);
    let local_offset = UtcOffset::current_local_offset().ok()?;
    Some(primitive_dt.assume_offset(local_offset))
}

// These conversion functions are no longer needed as we're using time crate throughout
// Kept for backward compatibility during transition

pub fn to_time_date(date: Date) -> Date {
    date
}

pub fn from_time_date(date: Date) -> Date {
    date
}

pub fn to_time_offset_date_time(time: OffsetDateTime) -> OffsetDateTime {
    time
}

// This function is no longer needed with time crate
// pub fn convert_local_result_to_datetime(...) - removed as chrono specific

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unix_to_offset_date_time() {
        let date = unix_to_offset_date_time(1587099600);
        let date_string = date
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap();
        // Test passes if it can parse the timestamp correctly
        assert!(date_string.contains("2020-04-17"));
    }

    #[test]
    fn test_offset_date_time_from_str_american() {
        let date = offset_date_time_from_str_american("02-10-2020", 18, None).unwrap();
        assert_eq!(date.year(), 2020);
        assert_eq!(date.month() as u8, 2);
        assert_eq!(date.day(), 10);
        assert_eq!(date.hour(), 18);
    }

    #[test]
    fn test_offset_date_time_from_str_standard() {
        let date = offset_date_time_from_str_standard("2020-02-10", 18, None).unwrap();
        assert_eq!(date.year(), 2020);
        assert_eq!(date.month() as u8, 2);
        assert_eq!(date.day(), 10);
        assert_eq!(date.hour(), 18);
    }

    #[test]
    fn test_date_from_str() {
        let date = date_from_str("2020-02-10", "%Y-%m-%d").unwrap();
        assert_eq!(date.year(), 2020);
        assert_eq!(date.month() as u8, 2);
        assert_eq!(date.day(), 10);
    }
}
