use super::{naive_date_string_to_time, unix_to_date_time, MarketQuoteError, MarketQuoteProvider};
use crate::quote::{Quote, Ticker};
use chrono::{DateTime, Utc};
use eodhistoricaldata_api as eod_api;

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

impl MarketQuoteProvider for EODHistData {
    /// Fetch latest quote
    fn fetch_latest_quote(&self, ticker: &Ticker) -> Result<Quote, MarketQuoteError> {
        let eod_quote = self
            .connector
            .get_latest_quote(&ticker.name)
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
    fn fetch_quote_history(
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
            .map_err(|e| MarketQuoteError::FetchFailed(e.to_string()))?;

        let mut quotes = Vec::new();
        for quote in &eod_quotes {
            let time = naive_date_string_to_time(&quote.date)?;
            quotes.push(Quote {
                id: None,
                ticker: ticker.id.unwrap(),
                price: quote.close,
                time,
                volume: Some(quote.volume as f64),
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
    use std::str::FromStr;

    #[test]
    fn test_eod_fetch_quote() {
        let token = "OeAFFmMliFG5orCUuwAKQ8l4WWFQ67YX".to_string();
        let eod = EODHistData::new(token);
        let ticker = Ticker {
            id: Some(1),
            asset: 1,
            name: "AAPL".to_string(),
            currency: Currency::from_str("EUR").unwrap(),
            source: 1,
            priority: 1,
        };
        let quote = eod.fetch_latest_quote(&ticker).unwrap();
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
            currency: Currency::from_str("EUR").unwrap(),
            source: 1,
            priority: 1,
        };
        let start = Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0);
        let end = Utc.ymd(2020, 1, 31).and_hms_milli(23, 59, 59, 999);
        let quotes = eod.fetch_quote_history(&ticker, start, end).unwrap();
        assert_eq!(quotes.len(), 21);
        assert!(quotes[0].price != 0.0);
    }
}
