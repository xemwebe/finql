use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use chrono::{DateTime, Local};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json;
use thiserror::Error;
use alpha_vantage;
use gurufocus_api;
use crate::datatypes::{QuoteHandler, CashFlow, Quote, Ticker};


pub mod alpha_vantage_wrapper;
pub mod comdirect;
pub mod eod_historical_data;
pub mod guru_focus;
pub mod yahoo;

#[derive(Error, Debug)]
pub enum MarketQuoteError {
    #[error("Storing quote in database failed")]
    StoringFailed(#[from] crate::datatypes::DataError),
    #[error("Fetching quote(s) from provider failed")]
    FetchFailed(#[from] reqwest::Error),
    #[error("Parsing quote date failed")]
    ParseDateFailed(#[from] chrono::format::ParseError),
    #[error("Parsing number failed")]
    ParseNumberFailed(#[from] std::num::ParseFloatError),
    #[error("Invalid currency")]
    InvalidCurrency(#[from] crate::datatypes::currency::CurrencyError),
    #[error("Conversion of date/time failed")]
    DateTimeError(#[from] crate::datatypes::date_time_helper::DateTimeError),
    #[error("Yahoo error")]
    YahooError(#[from] yahoo_finance_api::YahooError),
    #[error("EOD historical data error")]
    EodHistDataError(#[from] eodhistoricaldata_api::EodHistDataError),
    #[error("AlphaVantage error")]
    AlphaVantageError(#[from] alpha_vantage::error::Error),
    #[error("GuruFocus error")]
    GuruFocusError(#[from] gurufocus_api::GuruFocusError),
    #[error("JSON parsing error")]
    JSONError(#[from] serde_json::Error),
    #[error("Unexpected error: '{0}'")]
    UnexpectedError(String),
}


/// General interface for market data quotes provider
#[async_trait]
pub trait MarketQuoteProvider: Send+Sync {
    /// Fetch latest quote
    async fn fetch_latest_quote(&self, ticker: &Ticker) -> Result<Quote, MarketQuoteError>;
    /// Fetch historic quotes between start and end date
    async fn fetch_quote_history(
        &self,
        ticker: &Ticker,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> Result<Vec<Quote>, MarketQuoteError>;

    /// Fetch historic dividends (if any), returning a vector of cash flows, each cash flow representing a dividend payment per single stock
    async fn fetch_dividend_history(
        &self,
        ticker: &Ticker,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> Result<Vec<CashFlow>, MarketQuoteError>;
}

pub async fn update_ticker<'a>(
    provider: &(dyn MarketQuoteProvider + Send + Sync),
    ticker: &Ticker,
    db: Arc<dyn QuoteHandler+Send+Sync+'a>,
) -> Result<(), MarketQuoteError> {
    let mut quote = provider.fetch_latest_quote(ticker).await?;
    quote.price *= ticker.factor;
    db.insert_quote(&quote).await?;
    Ok(())
}


pub async fn update_ticker_history<'a>(
    provider: &(dyn MarketQuoteProvider + Send + Sync),
    ticker: &Ticker,
    db: Arc<dyn QuoteHandler+Send+Sync+'a>,
    start: DateTime<Local>,
    end: DateTime<Local>,
) -> Result<(), MarketQuoteError> {
    let mut quotes = provider.fetch_quote_history(ticker, start, end).await?;
    for mut quote in &mut quotes {
        quote.price *= ticker.factor;
        db.insert_quote(quote).await?;
    }
    Ok(())
}


#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum MarketDataSource {
    Manual,
    Yahoo,
    GuruFocus,
    EodHistData,
    AlphaVantage,
    Comdirect,
}

#[derive(Error, Debug, Clone)]
pub enum MarketDataSourceError {
    #[error("Parsing market data source failed")]
    ParseError
}

impl FromStr for MarketDataSource {
    type Err = MarketDataSourceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "manual" => Ok(Self::Manual),
            "yahoo" => Ok(Self::Yahoo),
            "gurufocus" => Ok(Self::GuruFocus),
            "eodhistdata" => Ok(Self::EodHistData),
            "alpha_vantage" => Ok(Self::AlphaVantage),
            "comdirect" => Ok(Self::Comdirect),
            _ => Err(MarketDataSourceError::ParseError),
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
            Self::AlphaVantage => write!(f, "alpha_vantage"),
            Self::Comdirect => write!(f, "comdirect"),
        }
    }
}

impl MarketDataSource {
    pub fn get_provider(
        &self,
        token: String,
    ) -> Option<Arc<dyn MarketQuoteProvider+Send+Sync>> {
        match self {
            Self::Yahoo => Some(Arc::new(yahoo::Yahoo {})),
            Self::GuruFocus => Some(Arc::new(guru_focus::GuruFocus::new(token))),
            Self::EodHistData => Some(Arc::new(
                eod_historical_data::EODHistData::new(token))),
            Self::AlphaVantage => Some(Arc::new(
                alpha_vantage_wrapper::AlphaVantage::new(token))),
            Self::Comdirect => Some(Arc::new(comdirect::Comdirect::new())),
            _ => None,
        }
    }

    pub fn extern_sources() -> Vec<String> {
        let v: Vec<String> = vec!["yahoo", "gurufocus", "eodhistdata", "alpha_vantage", "comdirect"]
            .into_iter().map(|x| x.to_string()).collect();
        v
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use std::str::FromStr;
    use chrono::offset::TimeZone;
    use chrono::{Duration, Local};
    use rand::Rng;

    use crate::datatypes::{Asset, Currency, QuoteHandler};
    use crate::postgres::PostgresDB;

    struct DummyProvider {}

    #[async_trait]
    impl MarketQuoteProvider for DummyProvider {
        async fn fetch_latest_quote(&self, ticker: &Ticker) -> Result<Quote, MarketQuoteError> {
            Ok(Quote {
                id: None,
                ticker: ticker.id.unwrap(),
                price: 1.23,
                time: Local.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
                volume: None,
            })
        }

        async fn fetch_quote_history(
            &self,
            ticker: &Ticker,
            start: DateTime<Local>,
            end: DateTime<Local>,
        ) -> Result<Vec<Quote>, MarketQuoteError> {
            let mut rng = rand::thread_rng();
            let mut quotes = Vec::new();
            let mut date = start;
            let mut price = 1.23;
            while date < end {
                quotes.push(Quote {
                    id: None,
                    ticker: ticker.id.unwrap(),
                    price,
                    time: date,
                    volume: None,
                });
                date = date + Duration::days(1);
                price *= (0.0001 + 0.2 * rng.gen::<f64>()).exp();
            }
            Ok(quotes)
        }

        async fn fetch_dividend_history(&self, _ticker: &Ticker, _start: DateTime<Local>, _end: DateTime<Local>) 
            -> Result<Vec<CashFlow>, MarketQuoteError> {
            Ok(Vec::new())
        }
    }

    async fn prepare_db(db: Arc<dyn QuoteHandler+Send+Sync>) -> Ticker {
        let asset_id = db
            .insert_asset(&Asset::Stock(Stock::new(
                None,
                "Apple AG".to_string(),
                Some("APPL".to_string()),
                None,
                None,
            )))
            .await.unwrap();

        let mut ticker = Ticker {
            id: None,
            asset: asset_id,
            name: "TestTicker".to_string(),
            currency: Currency::from_str("EUR").unwrap(),
            source: "manual".to_string(),
            priority: 1,
            factor: 1.0,
            tz: None,
            cal: None,
        };
        let ticker_id = db.insert_ticker(&ticker).await.unwrap();
        ticker.id = Some(ticker_id);
        ticker
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_fetch_latest_quote() {
        let tol = 1.0e-6;

        let db_url  = std::env::var("FINQL_TEST_DATABASE_URL");
        assert!(db_url.is_ok(), 
            "environment variable $FINQL_TEST_DATABASE_URL is not set");
        let db = PostgresDB::new(&db_url.unwrap()).await.unwrap();
        db.clean().await.unwrap();

        let db = Arc::new(db);
        let ticker = prepare_db(db.clone()).await;
        let provider = DummyProvider {};
        update_ticker(&provider, &ticker, db.clone()).await.unwrap();
        let quotes = db.get_all_quotes_for_ticker(ticker.id.unwrap()).await.unwrap();
        assert_eq!(quotes.len(), 1);
        assert_fuzzy_eq!(quotes[0].price, 1.23, tol);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_fetch_quote_history() {
        let tol = 1.0e-6;
        
        let db_url  = std::env::var("FINQL_TEST_DATABASE_URL");
        assert!(db_url.is_ok(), 
            "environment variable $FINQL_TEST_DATABASE_URL is not set");
        let db = PostgresDB::new(&db_url.unwrap()).await.unwrap();
        db.clean().await.unwrap();

        let db = Arc::new(db);
        let ticker = prepare_db(db.clone()).await;
        let provider = DummyProvider {};
        let start = Local.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0);
        let end = Local.ymd(2020, 1, 31).and_hms_milli(23, 59, 59, 999);
        update_ticker_history(&provider, &ticker, db.clone(), start, end).await.unwrap();
        let quotes = db.get_all_quotes_for_ticker(ticker.id.unwrap()).await.unwrap();
        assert_eq!(quotes.len(), 31);
        assert_fuzzy_eq!(quotes[0].price, 1.23, tol);
    }
}
