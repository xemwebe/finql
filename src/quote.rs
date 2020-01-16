use crate::currency::Currency;
///! Implementation of a container for basic asset data
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
    pub currency: Currency,
    pub source: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Quote {
    pub id: Option<usize>,
    pub ticker: usize,
    pub price: f64,
    pub time: DateTime<Utc>,
    pub volume: Option<f64>,
}
