use super::{MarketQuoteError, MarketQuoteProvider};
use crate::date_time_helper::{date_time_from_str_american, unix_to_date_time};
use finql_data::{Quote, Ticker};
use chrono::{DateTime, Utc};
use gurufocus_api;
use async_trait::async_trait;

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

#[async_trait]
impl MarketQuoteProvider for GuruFocus {
    /// Fetch latest quote
    async fn fetch_latest_quote(&self, ticker: &Ticker) -> Result<Quote, MarketQuoteError> {
        let prices = self
            .connector
            .get_quotes(&[&ticker.name])
            .await
            .map_err(|e| MarketQuoteError::FetchFailed(e.to_string()))?;

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
    async fn fetch_quote_history(
        &self,
        ticker: &Ticker,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Quote>, MarketQuoteError> {
        let gf_quotes = self
            .connector
            .get_price_hist(&ticker.name)
            .await
            .map_err(|e| MarketQuoteError::FetchFailed(e.to_string()))?;

        let gf_quotes: Vec<(String, f64)> = serde_json::from_value(gf_quotes)
            .map_err(|e| MarketQuoteError::FetchFailed(e.to_string()))?;

        let mut quotes = Vec::new();
        for (timestamp, price) in &gf_quotes {
            let time = date_time_from_str_american(timestamp, 18)?;
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
    use finql_data::Currency;
    use chrono::offset::TimeZone;
    use crate::market_quotes::MarketDataSource;
    use std::env;
    use std::str::FromStr;
    use tokio_test::block_on;

    #[test]
    fn test_gf_fetch_quote() {
        let token = env::var("GURUFOCUS_TOKEN").unwrap();
        let gf = GuruFocus::new(token);
        let ticker = Ticker {
            id: Some(1),
            asset: 1,
            name: "AAPL".to_string(),
            currency: Currency::from_str("USD").unwrap(),
            source: MarketDataSource::GuruFocus.to_string(),
            priority: 1,
            factor: 1.0,
        };
        let quote = block_on(gf.fetch_latest_quote(&ticker)).unwrap();
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
            currency: Currency::from_str("USD").unwrap(),
            source: MarketDataSource::GuruFocus.to_string(),
            priority: 1,
            factor: 1.0,
        };
        let start = Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0);
        let end = Utc.ymd(2020, 1, 31).and_hms_milli(23, 59, 59, 999);
        let quotes = block_on(gf.fetch_quote_history(&ticker, start, end)).unwrap();
        assert_eq!(quotes.len(), 23);
        assert!(quotes[0].price != 0.0);
    }
}
