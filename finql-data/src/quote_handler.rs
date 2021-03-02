///! Data handler trait for market quotes

use chrono::{DateTime, Utc};
use async_trait::async_trait;

use super::AssetHandler;
use super::DataError;
use crate::currency::{Currency, CurrencyConverter, CurrencyError};
use crate::quote::{Quote, Ticker};

/// Handler for globally available market quotes data
#[async_trait]
pub trait QuoteHandler: AssetHandler {
    // insert, get, update and delete for market data sources
    async fn insert_ticker(&mut self, ticker: &Ticker) -> Result<usize, DataError>;
    async fn get_ticker_id(&mut self, ticker: &str) -> Option<usize>;
    async fn insert_if_new_ticker(&mut self, ticker: &Ticker) -> Result<usize, DataError>;
    async fn get_ticker_by_id(&mut self, id: usize) -> Result<Ticker, DataError>;
    async fn get_all_ticker(&mut self) -> Result<Vec<Ticker>, DataError>;
    async fn get_all_ticker_for_source(
        &mut self,
        source: &str,
    ) -> Result<Vec<Ticker>, DataError>;

    /// Get all ticker that belong to a given asset specified by its asset ID
    async fn get_all_ticker_for_asset(
        &mut self,
        asset_id: usize,
    ) -> Result<Vec<Ticker>, DataError>;

    async fn update_ticker(&mut self, ticker: &Ticker) -> Result<(), DataError>;
    async fn delete_ticker(&mut self, id: usize) -> Result<(), DataError>;

    /// Insert, get, update and delete for market data sources
    async fn insert_quote(&mut self, quote: &Quote) -> Result<usize, DataError>;

    /// Get the last quote in database for a specific asset name on or before the given time
    async fn get_last_quote_before(
        &mut self,
        asset_name: &str,
        time: DateTime<Utc>,
    ) -> Result<(Quote, Currency), DataError>;

    /// Get the last quote in database for a specific asset id on or before the given time
    async fn get_last_quote_before_by_id(
        &mut self,
        asset_id: usize,
        time: DateTime<Utc>,
    ) -> Result<(Quote, Currency), DataError>;

    async fn get_all_quotes_for_ticker(&mut self, ticker_id: usize) -> Result<Vec<Quote>, DataError>;
    async fn update_quote(&mut self, quote: &Quote) -> Result<(), DataError>;
    async fn delete_quote(&mut self, id: usize) -> Result<(), DataError>;

    // Get and set cash rounding conventions by currency
    // This method never throws, if currency could not be found in table, return 2 by default instead
    async fn get_rounding_digits(&mut self, currency: Currency) -> i32;
    async fn set_rounding_digits(&mut self, currency: Currency, digits: i32) -> Result<(), DataError>;
}

#[async_trait]
impl CurrencyConverter for dyn QuoteHandler + Send {
    async fn fx_rate(&mut self, foreign: Currency, base: Currency, time: DateTime<Utc>) -> Result<f64, CurrencyError> {
        if foreign == base {
            return Ok(1.0);
        } else {
            let (fx_quote, quote_currency) =
                self.get_last_quote_before(&format!("{}", foreign), time).await
                    .map_err(|_| CurrencyError::ConversionFailed)?;
            if quote_currency == base {
                return Ok(fx_quote.price);
            }
        }
        Err(CurrencyError::ConversionFailed)    
    }
}