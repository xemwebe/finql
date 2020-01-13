///! Implementation of a container for basic asset data

use serde::{Serialize,Deserialize};
use crate::fixed_income::Amount;
use chrono::{Utc,DateTime};

#[derive(Debug,Serialize,Deserialize)]
pub struct MarketDataSource {
    id: u64,
    pub name: String,
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Ticker {
    id: u64,
    pub name: String,
    source: u64,
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Quote {
    id: u64,
    ticker: u64,
    pub price: Amount,
    pub time: DateTime<Utc>,
    pub volume: Option<u64>,
}
