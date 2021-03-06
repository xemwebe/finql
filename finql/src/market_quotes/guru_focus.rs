use super::{MarketQuoteError, MarketQuoteProvider};
use finql_data::{Quote, Ticker, CashFlow, date_time_helper::{date_time_from_str_american, unix_to_date_time}};
use chrono::{DateTime, Utc};
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
            .map_err(MarketQuoteError::FetchFailed)?;

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
            .map_err(MarketQuoteError::FetchFailed)?;

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

    /// Fetch historic dividend payments between start and end date
    async fn fetch_dividend_history(
        &self,
        ticker: &Ticker,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<CashFlow>, MarketQuoteError> {
        Err(MarketQuoteError::FetchFailed("Guru Focus interface does not support fetching dividends".to_string()))
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_gf_fetch_quote() {
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
        let quote = gf.fetch_latest_quote(&ticker).await.unwrap();
        assert!(quote.price != 0.0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_gf_fetch_history() {
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
        let quotes = gf.fetch_quote_history(&ticker, start, end).await.unwrap();
        assert!(quotes.len() > 15);
        assert!(quotes[0].price != 0.0);
    }
}
