///! Implementation of a container for basic asset data
use crate::fixed_income::Amount;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketDataSource {
    pub id: Option<usize>,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ticker {
    pub id: Option<usize>,
    pub name: String,
    pub source: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Quote {
    pub id: Option<usize>,
    pub ticker: usize,
    pub price: Amount,
    pub time: DateTime<Utc>,
    pub volume: Option<f64>,
}
