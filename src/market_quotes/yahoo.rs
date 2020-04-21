use super::{MarketQuoteError, MarketQuoteProvider};
use crate::quote::{Quote, Ticker};
use chrono::{DateTime, Utc};
use yahoo_finance;

pub struct Yahoo {}

impl MarketQuoteProvider for Yahoo {
    /// Fetch latest quote
    fn fetch_latest_quote(&self, ticker: &Ticker) -> Result<Quote, MarketQuoteError> {
        let data =
            yahoo_finance::history::retrieve_interval(&ticker.name, yahoo_finance::Interval::_1d)
                .map_err(|e| MarketQuoteError::FetchFailed(e.to_string()))?;
        let bar = data.last().ok_or(MarketQuoteError::FetchFailed(
            "received empty response from yahoo".to_string(),
        ))?;
        Ok(Quote {
            id: None,
            ticker: ticker.id.unwrap(),
            price: bar.close,
            time: bar.timestamp,
            volume: Some(bar.volume as f64),
        })
    }
    /// Fetch historic quotes between start and end date
    fn fetch_quote_history(
        &self,
        ticker: &Ticker,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Quote>, MarketQuoteError> {
        let data = yahoo_finance::history::retrieve_range(&ticker.name, start, Some(end))
            .map_err(|e| MarketQuoteError::FetchFailed(e.to_string()))?;
        let mut quotes = Vec::new();
        for bar in &data {
            quotes.push(Quote {
                id: None,
                ticker: ticker.id.unwrap(),
                price: bar.close,
                time: bar.timestamp,
                volume: Some(bar.volume as f64),
            })
        }
        Ok(quotes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::Currency;
    use crate::quote::MarketDataSource;
    use chrono::offset::TimeZone;
    use std::str::FromStr;

    #[test]
    fn test_yahoo_fetch_quote() {
        let yahoo = Yahoo {};
        let ticker = Ticker {
            id: Some(1),
            asset: 1,
            name: "AAPL".to_string(),
            currency: Currency::from_str("EUR").unwrap(),
            source: MarketDataSource::Yahoo,
            priority: 1,
        };
        let quote = yahoo.fetch_latest_quote(&ticker).unwrap();
        assert!(quote.price != 0.0);
    }

    #[test]
    fn test_yahoo_fetch_history() {
        let yahoo = Yahoo {};
        let ticker = Ticker {
            id: Some(1),
            asset: 1,
            name: "AAPL".to_string(),
            currency: Currency::from_str("EUR").unwrap(),
            source: MarketDataSource::Yahoo,
            priority: 1,
        };
        let start = Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0);
        let end = Utc.ymd(2020, 1, 31).and_hms_milli(23, 59, 59, 999);
        let quotes = yahoo.fetch_quote_history(&ticker, start, end).unwrap();
        assert_eq!(quotes.len(), 21);
        assert!(quotes[0].price != 0.0);
    }
}
