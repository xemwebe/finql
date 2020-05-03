use crate::data_handler::QuoteHandler;
use crate::quote::{Quote, Ticker};
use chrono::{DateTime, Utc};
use std::fmt;

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
pub trait MarketQuoteProvider {
    /// Fetch latest quote
    fn fetch_latest_quote(&self, ticker: &Ticker) -> Result<Quote, MarketQuoteError>;
    /// Fetch historic quotes between start and end date
    fn fetch_quote_history(
        &self,
        ticker: &Ticker,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Quote>, MarketQuoteError>;
}

pub fn update_ticker(
    provider: &dyn MarketQuoteProvider,
    ticker: &Ticker,
    db: &mut dyn QuoteHandler,
) -> Result<(), MarketQuoteError> {
    let mut quote = provider.fetch_latest_quote(ticker)?;
    quote.price *= ticker.factor;
    db.insert_quote(&quote)
        .map_err(|e| MarketQuoteError::StoringFailed(e.to_string()))?;
    Ok(())
}

pub fn update_ticker_history(
    provider: &dyn MarketQuoteProvider,
    ticker: &Ticker,
    db: &mut dyn QuoteHandler,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<(), MarketQuoteError> {
    let mut quotes = provider.fetch_quote_history(ticker, start, end)?;
    for mut quote in &mut quotes {
        quote.price *= ticker.factor;
        db.insert_quote(&quote)
            .map_err(|e| MarketQuoteError::StoringFailed(e.to_string()))?;
    }
    Ok(())
}

pub mod alpha_vantage;
pub mod comdirect;
pub mod eod_historical_data;
pub mod guru_focus;
pub mod yahoo;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::Asset;
    use crate::currency::Currency;
    use crate::data_handler::QuoteHandler;
    use crate::quote::MarketDataSource;
    use crate::sqlite_handler::SqliteDB;
    use chrono::offset::TimeZone;
    use chrono::{Duration, Utc};
    use rand::Rng;
    use std::str::FromStr;
    use rusqlite::Connection;

    struct DummyProvider {}

    impl MarketQuoteProvider for DummyProvider {
        fn fetch_latest_quote(&self, ticker: &Ticker) -> Result<Quote, MarketQuoteError> {
            Ok(Quote {
                id: None,
                ticker: ticker.id.unwrap(),
                price: 1.23,
                time: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
                volume: None,
            })
        }

        fn fetch_quote_history(
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

    fn prepare_db(db: &mut dyn QuoteHandler) -> Ticker {
        let asset_id = db
            .insert_asset(&Asset {
                id: None,
                name: "Apple AG".to_string(),
                wkn: None,
                isin: None,
                note: None,
            })
            .unwrap();

        let mut ticker = Ticker {
            id: None,
            asset: asset_id,
            name: "TestTicker".to_string(),
            currency: Currency::from_str("EUR").unwrap(),
            source: MarketDataSource::Manual,
            priority: 1,
            factor: 1.0,
        };
        let ticker_id = db.insert_ticker(&ticker).unwrap();
        ticker.id = Some(ticker_id);
        ticker
    }

    #[test]
    fn test_fetch_latest_quote() {
        let mut conn = Connection::open(":memory:").unwrap();
        let mut db = SqliteDB{ conn: &mut conn };
        db.init().unwrap();
        let ticker = prepare_db(&mut db);
        let provider = DummyProvider {};
        update_ticker(&provider, &ticker, &mut db).unwrap();
        let quotes = db.get_all_quotes_for_ticker(ticker.id.unwrap()).unwrap();
        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0].price, 1.23);
    }

    #[test]
    fn test_fetch_quote_history() {
        let mut conn = Connection::open(":memory:").unwrap();
        let mut db = SqliteDB{ conn: &mut conn };
        db.init().unwrap();
        let ticker = prepare_db(&mut db);
        let provider = DummyProvider {};
        let start = Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0);
        let end = Utc.ymd(2020, 1, 31).and_hms_milli(23, 59, 59, 999);
        update_ticker_history(&provider, &ticker, &mut db, start, end).unwrap();
        let quotes = db.get_all_quotes_for_ticker(ticker.id.unwrap()).unwrap();
        assert_eq!(quotes.len(), 31);
        assert_eq!(quotes[0].price, 1.23);
    }
}
