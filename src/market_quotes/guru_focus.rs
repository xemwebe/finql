use super::{MarketQuoteError, MarketQuoteProvider};
use crate::quote::{Quote, Ticker};
use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};
use gurufocus_api;
use std::time::{Duration, UNIX_EPOCH};

fn unix_to_date_time(seconds: u64) -> DateTime<Utc> {
    // Creates a new SystemTime from the specified number of whole seconds
    let d = UNIX_EPOCH + Duration::from_secs(seconds);
    // Create DateTime from SystemTime
    DateTime::<Utc>::from(d)
}

// Create UTC time from NaiveDate string, assume UTC 6pm (close of business) time
fn naive_date_string_to_time(date_string: &str) -> Result<DateTime<Utc>, MarketQuoteError> {
    let date = NaiveDate::parse_from_str(date_string, "%m-%d-%Y")
        .map_err(|e| MarketQuoteError::FetchFailed(e.to_string()))?;
    Ok(Utc
        .ymd(date.year(), date.month(), date.day())
        .and_hms_milli(18, 0, 0, 0))
}

pub struct GuruFocus {
    connector: gurufocus_api::GuruFocusConnector,
}

impl GuruFocus {
    pub fn new(token: String) -> GuruFocus {
        GuruFocus {
            connector: gurufocus_api::GuruFocusConnector::new(token),
        }
    }
}

impl MarketQuoteProvider for GuruFocus {
    /// Fetch latest quote
    fn fetch_latest_quote(&self, ticker: &Ticker) -> Result<Quote, MarketQuoteError> {
        let prices = self
            .connector
            .get_quotes(&[&ticker.name])
            .map_err(|e| MarketQuoteError::FetchFailed(e.to_string()))?;

        println!("prices: {:#?}", prices);
        let quote: gurufocus_api::Quote = serde_json::from_value(prices)
            .map_err(|e| MarketQuoteError::FetchFailed(e.to_string()))?;

        let time = unix_to_date_time(quote.timestamp as u64);
        Ok(Quote {
            id: None,
            ticker: ticker.id.unwrap(),
            price: quote.price.into(),
            time,
            volume: Some(quote.todays_volume.into()),
        })
    }
    /// Fetch historic quotes between start and end date
    fn fetch_quote_history(
        &self,
        ticker: &Ticker,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Quote>, MarketQuoteError> {
        let gf_quotes = self
            .connector
            .get_price_hist(&ticker.name)
            .map_err(|e| MarketQuoteError::FetchFailed(e.to_string()))?;

        let gf_quotes: Vec<(String, f64)> = serde_json::from_value(gf_quotes)
            .map_err(|e| MarketQuoteError::FetchFailed(e.to_string()))?;

        let mut quotes = Vec::new();
        for (timestamp, price) in &gf_quotes {
            let time = naive_date_string_to_time(timestamp)?;
            if time < start || time > end {
                continue;
            }
            quotes.push(Quote {
                id: None,
                ticker: ticker.id.unwrap(),
                price: *price,
                time,
                volume: None,
            })
        }
        Ok(quotes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::Currency;
    use chrono::offset::TimeZone;
    use std::env;
    use std::str::FromStr;

    #[test]
    fn test_gf_fetch_quote() {
        let token = env::var("GURUFOCUS_TOKEN").unwrap();
        let gf = GuruFocus::new(token);
        let ticker = Ticker {
            id: Some(1),
            asset: 1,
            name: "AAPL".to_string(),
            currency: Currency::from_str("EUR").unwrap(),
            source: 1,
            priority: 1,
        };
        let quote = gf.fetch_latest_quote(&ticker).unwrap();
        assert!(quote.price != 0.0);
    }

    #[test]
    fn test_gf_fetch_history() {
        let token = env::var("GURUFOCUS_TOKEN").unwrap();
        let gf = GuruFocus::new(token.to_string());
        let ticker = Ticker {
            id: Some(1),
            asset: 1,
            name: "AAPL".to_string(),
            currency: Currency::from_str("EUR").unwrap(),
            source: 1,
            priority: 1,
        };
        let start = Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0);
        let end = Utc.ymd(2020, 1, 31).and_hms_milli(23, 59, 59, 999);
        let quotes = gf.fetch_quote_history(&ticker, start, end).unwrap();
        assert_eq!(quotes.len(), 23);
        assert!(quotes[0].price != 0.0);
    }

    #[test]
    fn test_unix_to_date_time() {
        let date = unix_to_date_time(1587099600);
        let date_string = date.format("%Y-%m-%d %H:%M:%S").to_string();
        assert_eq!("2020-04-17 05:00:00", &date_string);
    }

    #[test]
    fn test_naive_date_string_to_time() {
        let date = naive_date_string_to_time("02-10-2020").unwrap();
        let date_string = date.format("%Y-%m-%d %H:%M:%S").to_string();
        assert_eq!("2020-02-10 18:00:00", &date_string);
    }
}
