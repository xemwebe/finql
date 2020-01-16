///! Data handler trait for market quotes

use super::DataError;
use crate::quote::{MarketDataSource, Ticker, Quote};
use chrono::{DateTime, Utc};
use crate::currency::Currency;

/// Handler for globally available market quotes data
pub trait QuoteHandler {
    // insert, get, update and delete for market data sources
    fn insert_md_source(&self, source: &MarketDataSource) -> Result<usize, DataError>;
    fn get_md_source_by_id(&self, id: usize) -> Result<MarketDataSource, DataError>;
    fn get_all_md_sources(&self) -> Result<Vec<MarketDataSource>, DataError>;
    fn update_md_source(&self, source: &MarketDataSource) -> Result<(), DataError>;
    fn delete_md_source(&self, id: usize) -> Result<(), DataError>;

    // insert, get, update and delete for market data sources
    fn insert_ticker(&self, ticker: &Ticker) -> Result<usize, DataError>;
    fn get_ticker_by_id(&self, id: usize) -> Result<Ticker, DataError>;
    fn get_all_ticker_for_source(&self, source_id: usize) -> Result<Vec<Ticker>, DataError>;
    fn update_ticker(&self, ticker: &Ticker) -> Result<(), DataError>;
    fn delete_ticker(&self, id: usize) -> Result<(), DataError>;
   
    // insert, get, update and delete for market data sources
    fn insert_quote(&self, quote: &Quote) -> Result<usize, DataError>;
    fn get_last_quote_before(&self, ticker_id: usize, time: DateTime<Utc>) -> Result<(Quote,Currency), DataError>;
    fn get_all_quotes_for_ticker(&self, ticker_id: usize) -> Result<Vec<Quote>, DataError>;
    fn update_quote(&self, quote: &Quote) -> Result<(), DataError>;
    fn delete_quote(&self, id: usize) -> Result<(), DataError>;
}
