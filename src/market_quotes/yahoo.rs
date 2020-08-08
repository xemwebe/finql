use super::{MarketQuoteError, MarketQuoteProvider};
use crate::date_time_helper::unix_to_date_time;
use crate::quote::{Quote, Ticker};
use chrono::{DateTime, Utc};
use yahoo_finance_api as yahoo;
use async_trait::async_trait;

pub struct Yahoo {}

#[async_trait]
impl MarketQuoteProvider for Yahoo {
    /// Fetch latest quote
    async fn fetch_latest_quote(&self, ticker: &Ticker) -> Result<Quote, MarketQuoteError> {
        let yahoo = yahoo::YahooConnector::new();
        let response = yahoo
            .get_latest_quotes(&ticker.name, "1m")
            .await
            .map_err(|e| MarketQuoteError::FetchFailed(e.to_string()))?;
        let quote = response
            .last_quote()
            .map_err(|e| MarketQuoteError::FetchFailed(e.to_string()))?;
        Ok(Quote {
            id: None,
            ticker: ticker.id.unwrap(),
            price: quote.close,
            time: unix_to_date_time(quote.timestamp),
            volume: Some(quote.volume as f64),
        })
    }
    /// Fetch historic quotes between start and end date
    async fn fetch_quote_history(
        &self,
        ticker: &Ticker,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Quote>, MarketQuoteError> {
        let yahoo = yahoo::YahooConnector::new();
        let response = yahoo
            .get_quote_history(&ticker.name, start, end)
            .await
            .map_err(|e| MarketQuoteError::FetchFailed(e.to_string()))?;
        let yahoo_quotes = response
            .quotes()
            .map_err(|e| MarketQuoteError::FetchFailed(e.to_string()))?;
        let mut quotes = Vec::new();
        for quote in &yahoo_quotes {
            let volume = Some(quote.volume as f64);
            let time = unix_to_date_time(quote.timestamp);
            quotes.push(Quote {
                id: None,
                ticker: ticker.id.unwrap(),
                price: quote.close,
                time,
                volume,
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
    use tokio_test::block_on;

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
            factor: 1.0,
        };
        let quote = block_on(yahoo.fetch_latest_quote(&ticker)).unwrap();
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
            factor: 1.0,
        };
        let start = Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0);
        let end = Utc.ymd(2020, 1, 31).and_hms_milli(23, 59, 59, 999);
        let quotes = block_on(yahoo.fetch_quote_history(&ticker, start, end)).unwrap();
        assert_eq!(quotes.len(), 21);
        assert!(quotes[0].price != 0.0);
    }
}
