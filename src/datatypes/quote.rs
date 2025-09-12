//! Implementation of a container for basic asset data
use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::Currency;
use super::{DataError, DataItem};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub id: Option<i32>,
    pub asset: i32,
    pub name: String,
    pub currency: Currency,
    pub source: String,
    pub priority: i32,
    pub factor: f64,
    pub tz: Option<String>,
    pub cal: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub id: Option<i32>,
    pub ticker: i32,
    pub price: f64,
    pub time: OffsetDateTime,
    pub volume: Option<f64>,
}

impl Ord for Quote {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.time, &self.ticker).cmp(&(other.time, &other.ticker))
    }
}

impl PartialOrd for Quote {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Quote {
    fn eq(&self, other: &Self) -> bool {
        (self.time, &self.ticker) == (other.time, &other.ticker)
    }
}

impl Eq for Quote {}

impl DataItem for Quote {
    // get id or return error if id hasn't been set yet
    fn get_id(&self) -> Result<i32, DataError> {
        match self.id {
            Some(id) => Ok(id),
            None => Err(DataError::DataAccessFailure(
                "tried to get id of temporary quote".to_string(),
            )),
        }
    }
    // set id or return error if id has already been set
    fn set_id(&mut self, id: i32) -> Result<(), DataError> {
        match self.id {
            Some(_) => Err(DataError::DataAccessFailure(
                "tried to change valid quote id".to_string(),
            )),
            None => {
                self.id = Some(id);
                Ok(())
            }
        }
    }
}

impl DataItem for Ticker {
    // get id or return error if id hasn't been set yet
    fn get_id(&self) -> Result<i32, DataError> {
        match self.id {
            Some(id) => Ok(id),
            None => Err(DataError::DataAccessFailure(
                "tried to get id of temporary ticker".to_string(),
            )),
        }
    }
    // set id or return error if id has already been set
    fn set_id(&mut self, id: i32) -> Result<(), DataError> {
        match self.id {
            Some(_) => Err(DataError::DataAccessFailure(
                "tried to change valid ticker id".to_string(),
            )),
            None => {
                self.id = Some(id);
                Ok(())
            }
        }
    }
}
