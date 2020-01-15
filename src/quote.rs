///! Implementation of a container for basic asset data
use crate::fixed_income::Amount;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketDataSource {
    id: usize,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ticker {
    id: usize,
    pub name: String,
    source: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Quote {
    id: usize,
    ticker: usize,
    pub price: Amount,
    pub time: DateTime<Utc>,
    pub volume: Option<usize>,
}
