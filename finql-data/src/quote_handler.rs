///! Data handler trait for market quotes

use chrono::{DateTime, Utc};
use async_trait::async_trait;
use std::sync::Arc;

use super::AssetHandler;
use super::DataError;
use crate::currency::Currency;
use crate::quote::{Quote, Ticker};

/// Handler for globally available market quotes data
#[async_trait]
pub trait QuoteHandler: AssetHandler {
    fn into_arc_dispatch(self: Arc<Self>) -> Arc<dyn AssetHandler+Send+Sync>;

    // insert, get, update and delete for market data sources
    async fn insert_ticker(&self, ticker: &Ticker) -> Result<usize, DataError>;
    async fn get_ticker_id(&self, ticker: &str) -> Option<usize>;
    async fn insert_if_new_ticker(&self, ticker: &Ticker) -> Result<usize, DataError>;
    async fn get_ticker_by_id(&self, id: usize) -> Result<Ticker, DataError>;
    async fn get_all_ticker(&self) -> Result<Vec<Ticker>, DataError>;
    async fn get_all_ticker_for_source(
        &self,
        source: &str,
    ) -> Result<Vec<Ticker>, DataError>;

    /// Get all ticker that belong to a given asset specified by its asset ID
    async fn get_all_ticker_for_asset(
        &self,
        asset_id: usize,
    ) -> Result<Vec<Ticker>, DataError>;

    async fn update_ticker(&self, ticker: &Ticker) -> Result<(), DataError>;
    async fn delete_ticker(&self, id: usize) -> Result<(), DataError>;

    /// Insert, get, update and delete for market data sources
    async fn insert_quote(&self, quote: &Quote) -> Result<usize, DataError>;

    /// Get the last quote in database for a specific asset name on or before the given time
    async fn get_last_quote_before(
        &self,
        asset_name: &str,
        time: DateTime<Utc>,
    ) -> Result<(Quote, Currency), DataError>;

    /// Get the last quote in database for a specific asset id on or before the given time
    async fn get_last_quote_before_by_id(
        &self,
        asset_id: usize,
        time: DateTime<Utc>,
    ) -> Result<(Quote, Currency), DataError>;

    async fn get_all_quotes_for_ticker(&self, ticker_id: usize) -> Result<Vec<Quote>, DataError>;
    async fn update_quote(&self, quote: &Quote) -> Result<(), DataError>;
    async fn delete_quote(&self, id: usize) -> Result<(), DataError>;
    async fn remove_duplicates(&self) -> Result<(), DataError>;

    // Get and set cash rounding conventions by currency
    // This method never throws, if currency could not be found in table, return 2 by default instead
    async fn get_rounding_digits(&self, currency: Currency) -> i32;
    async fn set_rounding_digits(&self, currency: Currency, digits: i32) -> Result<(), DataError>;
}
