///! Implementation of a container for basic asset data
use crate::currency::Currency;
use crate::data_handler::{DataError, DataItem};
use crate::market_quotes;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum MarketDataSource {
    Manual,
    Yahoo,
    GuruFocus,
    EodHistData,
}

#[derive(Debug, Clone)]
pub struct ParseMarketDataSourceError {}

impl fmt::Display for ParseMarketDataSourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parsing market data source failed")
    }
}

impl FromStr for MarketDataSource {
    type Err = ParseMarketDataSourceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "manual" => Ok(Self::Manual),
            "yahoo" => Ok(Self::Yahoo),
            "gurufocus" => Ok(Self::GuruFocus),
            "eodhistdata" => Ok(Self::EodHistData),
            _ => Err(ParseMarketDataSourceError {}),
        }
    }
}

impl fmt::Display for MarketDataSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Manual => write!(f, "manual"),
            Self::Yahoo => write!(f, "yahoo"),
            Self::GuruFocus => write!(f, "gurufocus"),
            Self::EodHistData => write!(f, "eodhistdata"),
        }
    }
}

impl MarketDataSource {
    pub fn get_provider(&self, token: String) -> Option<Box<dyn market_quotes::MarketQuoteProvider>> {
        match self {
            Self::Yahoo => Some(Box::new(market_quotes::yahoo::Yahoo{})),
            Self::GuruFocus => Some(Box::new(market_quotes::guru_focus::GuruFocus::new(token))),
            Self::EodHistData => Some(Box::new(market_quotes::eod_historical_data::EODHistData::new(token))),
            _ => None, 
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub id: Option<usize>,
    pub asset: usize,
    pub name: String,
    pub currency: Currency,
    pub source: MarketDataSource,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub id: Option<usize>,
    pub ticker: usize,
    pub price: f64,
    pub time: DateTime<Utc>,
    pub volume: Option<f64>,
}

impl DataItem for Quote {
    // get id or return error if id hasn't been set yet
    fn get_id(&self) -> Result<usize, DataError> {
        match self.id {
            Some(id) => Ok(id),
            None => Err(DataError::DataAccessFailure(
                "tried to get id of temporary quote".to_string(),
            )),
        }
    }
    // set id or return error if id has already been set
    fn set_id(&mut self, id: usize) -> Result<(), DataError> {
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
    fn get_id(&self) -> Result<usize, DataError> {
        match self.id {
            Some(id) => Ok(id),
            None => Err(DataError::DataAccessFailure(
                "tried to get id of temporary ticker".to_string(),
            )),
        }
    }
    // set id or return error if id has already been set
    fn set_id(&mut self, id: usize) -> Result<(), DataError> {
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
