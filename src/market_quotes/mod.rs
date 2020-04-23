use crate::data_handler::QuoteHandler;
use crate::quote::{Quote, Ticker};
use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};
use std::fmt;
use std::time::{Duration, UNIX_EPOCH};

#[derive(Debug)]
pub enum MarketQuoteError {
    StoringFailed(String),
    FetchFailed(String),
}

impl std::error::Error for MarketQuoteError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        Some(self)
    }
}

impl fmt::Display for MarketQuoteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StoringFailed(err) => write!(f, "storing quote in database failed: {}", err),
            Self::FetchFailed(err) => write!(f, "fetching quote(s) from provider failed: {}", err),
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

pub mod eod_historical_data;
pub mod guru_focus;
pub mod yahoo;

fn unix_to_date_time(seconds: u64) -> DateTime<Utc> {
    // Creates a new SystemTime from the specified number of whole seconds
    let d = UNIX_EPOCH + Duration::from_secs(seconds);
    // Create DateTime from SystemTime
    DateTime::<Utc>::from(d)
}

// Create UTC time from NaiveDate string, assume UTC 6pm (close of business) time
fn naive_date_string_to_time(date_string: &str) -> Result<DateTime<Utc>, MarketQuoteError> {
    let date = NaiveDate::parse_from_str(date_string, "%Y-%m-%d")
        .map_err(|e| MarketQuoteError::FetchFailed(e.to_string()))?;
    Ok(Utc
        .ymd(date.year(), date.month(), date.day())
        .and_hms_milli(18, 0, 0, 0))
}

// Create UTC time from NaiveDate string, assume UTC 6pm (close of business) time (English convention)
fn naive_date_string_to_time_english(date_string: &str) -> Result<DateTime<Utc>, MarketQuoteError> {
    let date = NaiveDate::parse_from_str(date_string, "%m-%d-%Y")
        .map_err(|e| MarketQuoteError::FetchFailed(e.to_string()))?;
    Ok(Utc
        .ymd(date.year(), date.month(), date.day())
        .and_hms_milli(18, 0, 0, 0))
}

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
        };
        let ticker_id = db.insert_ticker(&ticker).unwrap();
        ticker.id = Some(ticker_id);
        ticker
    }

    #[test]
    fn test_fetch_latest_quote() {
        let mut db = SqliteDB::create(":memory:").unwrap();
        let ticker = prepare_db(&mut db);
        let provider = DummyProvider {};
        update_ticker(&provider, &ticker, &mut db).unwrap();
        let quotes = db.get_all_quotes_for_ticker(ticker.id.unwrap()).unwrap();
        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0].price, 1.23);
    }

    #[test]
    fn test_fetch_quote_history() {
        let mut db = SqliteDB::create(":memory:").unwrap();
        let ticker = prepare_db(&mut db);
        let provider = DummyProvider {};
        let start = Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0);
        let end = Utc.ymd(2020, 1, 31).and_hms_milli(23, 59, 59, 999);
        update_ticker_history(&provider, &ticker, &mut db, start, end).unwrap();
        let quotes = db.get_all_quotes_for_ticker(ticker.id.unwrap()).unwrap();
        assert_eq!(quotes.len(), 31);
        assert_eq!(quotes[0].price, 1.23);
    }

    #[test]
    fn test_unix_to_date_time() {
        let date = unix_to_date_time(1587099600);
        let date_string = date.format("%Y-%m-%d %H:%M:%S").to_string();
        assert_eq!("2020-04-17 05:00:00", &date_string);
    }

    #[test]
    fn test_naive_date_string_to_time_english() {
        let date = naive_date_string_to_time_english("02-10-2020").unwrap();
        let date_string = date.format("%Y-%m-%d %H:%M:%S").to_string();
        assert_eq!("2020-02-10 18:00:00", &date_string);
    }

    #[test]
    fn test_naive_date_string_to_time() {
        let date = naive_date_string_to_time("2020-02-10").unwrap();
        let date_string = date.format("%Y-%m-%d %H:%M:%S").to_string();
        assert_eq!("2020-02-10 18:00:00", &date_string);
    }
}
