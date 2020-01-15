///! Implementation of a container for basic asset data
use crate::fixed_income::Amount;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketDataSource {
    pub id: usize,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ticker {
    pub id: usize,
    pub name: String,
    pub source: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Quote {
    pub id: usize,
    pub ticker: usize,
    pub price: Amount,
    pub time: DateTime<Utc>,
    pub volume: Option<usize>,
}
