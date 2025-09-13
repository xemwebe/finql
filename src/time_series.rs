use cal_calc::Calendar;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;
use time::{Date, OffsetDateTime};

use crate::datatypes::date_time_helper::DateTimeError;

#[derive(Error, Debug)]
pub enum TimeSeriesError {
    #[error("Time series is empty.")]
    IsEmpty,
    #[error("Date conversion failure")]
    DayConversionError(#[from] DateTimeError),
    #[error("Calendar error")]
    CalendarError(#[from] cal_calc::CalendarError),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeValue {
    pub time: OffsetDateTime,
    pub value: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeSeries {
    pub series: Vec<TimeValue>,
    pub title: String,
}

impl TimeSeries {
    pub fn new(title: &str) -> Self {
        TimeSeries {
            series: vec![],
            title: title.to_owned(),
        }
    }
    pub fn min_max(&self) -> Result<(Date, Date, f64, f64), TimeSeriesError> {
        if self.series.is_empty() {
            return Err(TimeSeriesError::IsEmpty);
        }
        let mut min_val = self.series[0].value;
        let mut max_val = min_val;
        let min_time = self.series[0].time;
        let max_time = self.series.last().unwrap().time;
        for v in &self.series {
            if min_val > v.value {
                min_val = v.value;
            }
            if max_val < v.value {
                max_val = v.value;
            }
        }
        Ok((min_time.date(), max_time.date(), min_val, max_val))
    }

    pub fn find_gaps(
        &self,
        cal: &Calendar,
        min_size: usize,
    ) -> Result<Vec<(Date, Date)>, TimeSeriesError> {
        let mut gaps = Vec::new();
        let (min_date, _, _, _) = self.min_max()?;
        let today = OffsetDateTime::now_utc().date();
        let dates: HashSet<Date> = self.series.iter().map(|t| t.time.date()).collect();
        let mut gap_begin = None;
        let mut date = min_date;
        let mut gap_size = 0;
        while date <= today {
            match gap_begin {
                None => {
                    if !dates.contains(&date) {
                        gap_begin = Some(date);
                        gap_size = 1;
                    }
                }

                Some(d) => {
                    if dates.contains(&date) {
                        if gap_size >= min_size {
                            let prev_date = cal.prev_bday(date)?;
                            gaps.push((d, prev_date));
                        }
                        gap_begin = None;
                    } else {
                        gap_size += 1;
                    }
                }
            }
            date = cal.next_bday(date)?;
        }

        if let Some(d) = gap_begin {
            gaps.push((d, today));
        }

        Ok(gaps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datatypes::date_time_helper::make_time;
    use cal_calc::Holiday;
    use time::Weekday;

    #[test]
    fn finding_gaps() {
        let holidays = vec![
            Holiday::SingularDay(Date::from_calendar_date(2021, time::Month::November, 4).unwrap()),
            Holiday::SingularDay(Date::from_calendar_date(2021, time::Month::November, 5).unwrap()),
            Holiday::SingularDay(Date::from_calendar_date(2021, time::Month::November, 8).unwrap()),
            Holiday::WeekDay(Weekday::Sat),
            Holiday::WeekDay(Weekday::Sun),
        ];
        let today = OffsetDateTime::now_utc().date();
        let cal = Calendar::calc_calendar(&holidays, 2021, today.year());

        let mut ts = TimeSeries {
            title: "test".to_string(),
            series: Vec::new(),
        };
        ts.series.push(TimeValue {
            time: make_time(2021, 10, 28, 20, 0, 0).unwrap(),
            value: 1.0,
        });
        ts.series.push(TimeValue {
            time: make_time(2021, 11, 1, 20, 0, 0).unwrap(),
            value: 1.0,
        });
        ts.series.push(TimeValue {
            time: make_time(2021, 11, 8, 20, 0, 0).unwrap(),
            value: 1.0,
        });
        ts.series.push(TimeValue {
            time: make_time(2021, 11, 9, 20, 0, 0).unwrap(),
            value: 1.0,
        });

        let gaps = ts.find_gaps(&cal, 1).unwrap();
        assert_eq!(gaps.len(), 3);
        assert_eq!(
            gaps[0].0,
            Date::from_calendar_date(2021, time::Month::October, 29).unwrap()
        );
        assert_eq!(
            gaps[0].1,
            Date::from_calendar_date(2021, time::Month::October, 29).unwrap()
        );
        assert_eq!(
            gaps[1].0,
            Date::from_calendar_date(2021, time::Month::November, 2).unwrap()
        );
        assert_eq!(
            gaps[1].1,
            Date::from_calendar_date(2021, time::Month::November, 3).unwrap()
        );
        assert_eq!(
            gaps[2].0,
            Date::from_calendar_date(2021, time::Month::November, 10).unwrap()
        );
        assert_eq!(gaps[2].1, today);
    }
}
