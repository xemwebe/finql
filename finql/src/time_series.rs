use std::error::Error;
use std::fmt;
use chrono::NaiveDate;

#[derive(Debug)]
pub enum TimeSeriesError {
    IsEmpty,
}

impl fmt::Display for TimeSeriesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeSeriesError::IsEmpty => write!(f, "Time series is empty."),
        }
    }
}

impl Error for TimeSeriesError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}


#[derive(Debug)]
pub struct TimeValue {
    pub date: NaiveDate,
    pub value: f64,
}

#[derive(Debug)]
pub struct TimeSeries {
    pub series: Vec<TimeValue>,
    pub title: String,
}

impl TimeSeries {
    pub fn min_max(&self) -> Result<(NaiveDate, NaiveDate, f64, f64), TimeSeriesError> {
        if self.series.len() == 0 {
            return Err(TimeSeriesError::IsEmpty)
        }
        let mut min_val = self.series[0].value;
        let mut max_val = min_val;
        let min_date = self.series[0].date;
        let max_date = self.series.last().unwrap().date;
        for v in &self.series {
            if min_val > v.value {
                min_val = v.value;
            } 
            if max_val < v.value {
                max_val = v.value;
            }
       }
       Ok((min_date,max_date, min_val, max_val))
    }
}