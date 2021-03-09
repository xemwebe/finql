use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use finql_data::QuoteHandler;
use finql_data::quote::{Quote, Ticker};


pub mod alpha_vantage;
pub mod comdirect;
pub mod eod_historical_data;
pub mod guru_focus;
pub mod yahoo;

#[derive(Debug)]
pub enum MarketQuoteError {
    StoringFailed(String),
    FetchFailed(String),
    ParseDateFailed(chrono::format::ParseError),
}

impl std::error::Error for MarketQuoteError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            Self::ParseDateFailed(err) => Some(err),
            _ => None,
        }
    }
}

impl std::convert::From<chrono::format::ParseError> for MarketQuoteError {
    fn from(error: chrono::format::ParseError) -> Self {
        Self::ParseDateFailed(error)
    }
}

impl fmt::Display for MarketQuoteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StoringFailed(err) => write!(f, "storing quote in database failed: {}", err),
            Self::FetchFailed(err) => write!(f, "fetching quote(s) from provider failed: {}", err),
            Self::ParseDateFailed(_) => write!(f, "parsing a quote date failed"),
        }
    }
}

/// General interface for market data quotes provider
#[async_trait]
pub trait MarketQuoteProvider: Send {
    /// Fetch latest quote
    async fn fetch_latest_quote(&self, ticker: &Ticker) -> Result<Quote, MarketQuoteError>;
    /// Fetch historic quotes between start and end date
    async fn fetch_quote_history(
        &self,
        ticker: &Ticker,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Quote>, MarketQuoteError>;
}

pub async fn update_ticker(
    provider: &dyn MarketQuoteProvider,
    ticker: &Ticker,
    db: &mut dyn QuoteHandler,
) -> Result<(), MarketQuoteError> {
    let mut quote = provider.fetch_latest_quote(&ticker).await?;
    quote.price *= ticker.factor;
    db.insert_quote(&quote).await
        .map_err(|e| MarketQuoteError::StoringFailed(e.to_string()))?;
    Ok(())
}

pub async fn update_ticker_history(
    provider: &dyn MarketQuoteProvider,
    ticker: &Ticker,
    db: &mut dyn QuoteHandler,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<(), MarketQuoteError> {
    let mut quotes = provider.fetch_quote_history(ticker, start, end).await?;
    for mut quote in &mut quotes {
        quote.price *= ticker.factor;
        db.insert_quote(&quote).await
            .map_err(|e| MarketQuoteError::StoringFailed(e.to_string()))?;
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
            "alpha_vantage" => Ok(Self::AlphaVantage),
            "comdirect" => Ok(Self::Comdirect),
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
            Self::AlphaVantage => write!(f, "alpha_vantage"),
            Self::Comdirect => write!(f, "comdirect"),
        }
    }
}

impl MarketDataSource {
    pub fn get_provider(
        &self,
        token: String,
    ) -> Option<Box<dyn MarketQuoteProvider>> {
        match self {
            Self::Yahoo => Some(Box::new(yahoo::Yahoo {})),
            Self::GuruFocus => Some(Box::new(guru_focus::GuruFocus::new(token))),
            Self::EodHistData => Some(Box::new(
                eod_historical_data::EODHistData::new(token))),
            Self::AlphaVantage => Some(Box::new(
                alpha_vantage::AlphaVantage::new(token))),
            Self::Comdirect => Some(Box::new(comdirect::Comdirect::new())),
            _ => None,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use std::str::FromStr;
    use chrono::offset::TimeZone;
    use chrono::{Duration, Utc};
    use rand::Rng;

    use finql_data::{Asset, Currency, QuoteHandler};
    use finql_sqlite::SqliteDB;

    struct DummyProvider {}

    #[async_trait]
    impl MarketQuoteProvider for DummyProvider {
        async fn fetch_latest_quote(&self, ticker: &Ticker) -> Result<Quote, MarketQuoteError> {
            Ok(Quote {
                id: None,
                ticker: ticker.id.unwrap(),
                price: 1.23,
                time: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
                volume: None,
            })
        }

        async fn fetch_quote_history(
            &self,
            ticker: &Ticker,
            start: DateTime<Utc>,
            end: DateTime<Utc>,
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
    }

    async fn prepare_db(db: &mut dyn QuoteHandler) -> Ticker {
        let asset_id = db
            .insert_asset(&Asset {
                id: None,
                name: "Apple AG".to_string(),
                wkn: None,
                isin: None,
                note: None,
            })
            .await.unwrap();

        let mut ticker = Ticker {
            id: None,
            asset: asset_id,
            name: "TestTicker".to_string(),
            currency: Currency::from_str("EUR").unwrap(),
            source: "manual".to_string(),
            priority: 1,
            factor: 1.0,
        };
        let ticker_id = db.insert_ticker(&ticker).await.unwrap();
        ticker.id = Some(ticker_id);
        ticker
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_fetch_latest_quote() {
        let mut db = SqliteDB::new("sqlite::memory:").await.unwrap();
        db.init().await.unwrap();
        let ticker = prepare_db(&mut db).await;
        let provider = DummyProvider {};
        update_ticker(&provider, &ticker, &mut db).await.unwrap();
        let quotes = db.get_all_quotes_for_ticker(ticker.id.unwrap()).await.unwrap();
        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0].price, 1.23);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_fetch_quote_history() {
        let mut db = SqliteDB::new("sqlite::memory:").await.unwrap();
        db.init().await.unwrap();
        let ticker = prepare_db(&mut db).await;
        let provider = DummyProvider {};
        let start = Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0);
        let end = Utc.ymd(2020, 1, 31).and_hms_milli(23, 59, 59, 999);
        update_ticker_history(&provider, &ticker, &mut db, start, end).await.unwrap();
        let quotes = db.get_all_quotes_for_ticker(ticker.id.unwrap()).await.unwrap();
        assert_eq!(quotes.len(), 31);
        assert_eq!(quotes[0].price, 1.23);
    }
}
