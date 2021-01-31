use super::{MarketQuoteError, MarketQuoteProvider};
use crate::date_time_helper::{date_time_from_str_standard, unix_to_date_time};
use finql_data::{Quote, Ticker};
use chrono::{DateTime, Utc};
use eodhistoricaldata_api as eod_api;
use async_trait::async_trait;

pub struct EODHistData {
    connector: eod_api::EodHistConnector,
}

impl EODHistData {
    pub fn new(token: String) -> EODHistData {
        EODHistData {
            connector: eod_api::EodHistConnector::new(token),
        }
    }
}

#[async_trait]
impl MarketQuoteProvider for EODHistData {
    /// Fetch latest quote
    async fn fetch_latest_quote(&self, ticker: &Ticker) -> Result<Quote, MarketQuoteError> {
        let eod_quote = self
            .connector
            .get_latest_quote(&ticker.name)
            .await
            .map_err(|e| MarketQuoteError::FetchFailed(e.to_string()))?;

        let time = unix_to_date_time(eod_quote.timestamp as u64);
        Ok(Quote {
            id: None,
            ticker: ticker.id.unwrap(),
            price: eod_quote.close,
            time,
            volume: Some(eod_quote.volume as f64),
        })
    }

    /// Fetch historic quotes between start and end date
    async fn fetch_quote_history(
        &self,
        ticker: &Ticker,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Quote>, MarketQuoteError> {
        let eod_quotes = self
            .connector
            .get_quote_history(
                &ticker.name,
                start.naive_utc().date(),
                end.naive_utc().date(),
            )
            .await
            .map_err(|e| MarketQuoteError::FetchFailed(e.to_string()))?;

        let mut quotes = Vec::new();
        for quote in &eod_quotes {
            let time = date_time_from_str_standard(&quote.date, 18)?;
            let volume = match quote.volume {
                Some(vol) => Some(vol as f64),
                None => None,
            };
            if let Some(price) = quote.close {
                quotes.push(Quote {
                    id: None,
                    ticker: ticker.id.unwrap(),
                    price,
                    time,
                    volume,
                })
            }
        }
        Ok(quotes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use finql_data::Currency;
    use chrono::TimeZone;
    use std::str::FromStr;
    use tokio_test::block_on;

    #[test]
    fn test_eod_fetch_quote() {
        let token = "OeAFFmMliFG5orCUuwAKQ8l4WWFQ67YX".to_string();
        let eod = EODHistData::new(token);
        let ticker = Ticker {
            id: Some(1),
            asset: 1,
            name: "AAPL".to_string(),
            currency: Currency::from_str("USD").unwrap(),
            source: MarketDataSource::EodHistData,
            priority: 1,
            factor: 1.0,
        };
        let quote = block_on(eod.fetch_latest_quote(&ticker)).unwrap();
        assert!(quote.price != 0.0);
    }

    #[test]
    fn test_eod_fetch_history() {
        let token = "OeAFFmMliFG5orCUuwAKQ8l4WWFQ67YX".to_string();
        let eod = EODHistData::new(token.to_string());
        let ticker = Ticker {
            id: Some(1),
            asset: 1,
            name: "AAPL".to_string(),
            currency: Currency::from_str("USD").unwrap(),
            source: MarketDataSource::EodHistData,
            priority: 1,
            factor: 1.0,
        };
        let start = Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0);
        let end = Utc.ymd(2020, 1, 31).and_hms_milli(23, 59, 59, 999);
        let quotes = block_on(eod.fetch_quote_history(&ticker, start, end)).unwrap();
        assert_eq!(quotes.len(), 21);
        assert!(quotes[0].price != 0.0);
    }
}
