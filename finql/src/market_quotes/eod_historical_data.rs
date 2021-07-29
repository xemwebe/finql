use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use eodhistoricaldata_api as eod_api;
use finql_data::{CashFlow, Currency, Quote, Ticker, 
        date_time_helper::{
            date_time_from_str_standard, 
            date_from_str, 
            unix_to_date_time,
            naive_date_to_date_time,
        }
    };

use super::{MarketQuoteError, MarketQuoteProvider};

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
            let volume = quote.volume.map(|vol| vol as f64);
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

    /// Fetch historic dividend payments between start and end date
    async fn fetch_dividend_history(
        &self,
        ticker: &Ticker,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<CashFlow>, MarketQuoteError> {
        let dividends_since_start = self
            .connector
            .get_dividend_history(
                &ticker.name,
                start.naive_utc().date()
            )
            .await
            .map_err(|e| MarketQuoteError::FetchFailed(e.to_string()))?;
        let mut div_cash_flows = Vec::new();
        for div in dividends_since_start {
            let pay_date = date_from_str(&div.payment_date,"%Y-%m-%d")?;
            if naive_date_to_date_time(&pay_date, 18) <= end {
                let currency = Currency::from_str(&div.currency)
                    .map_err(|e| MarketQuoteError::FetchFailed(e.to_string()))?;
                div_cash_flows.push(CashFlow::new(div.value, currency, pay_date));
            }
        }
        Ok(div_cash_flows)        
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::TimeZone;

    use finql_data::Currency;

    use super::*;
    use crate::market_quotes::MarketDataSource;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_eod_fetch_quote() {
        let token = "OeAFFmMliFG5orCUuwAKQ8l4WWFQ67YX".to_string();
        let eod = EODHistData::new(token);
        let ticker = Ticker {
            id: Some(1),
            asset: 1,
            name: "AAPL".to_string(),
            currency: Currency::from_str("USD").unwrap(),
            source: MarketDataSource::EodHistData.to_string(),
            priority: 1,
            factor: 1.0,
        };
        let quote = eod.fetch_latest_quote(&ticker).await.unwrap();
        assert!(quote.price != 0.0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_eod_fetch_history() {
        let token = "OeAFFmMliFG5orCUuwAKQ8l4WWFQ67YX".to_string();
        let eod = EODHistData::new(token.to_string());
        let ticker = Ticker {
            id: Some(1),
            asset: 1,
            name: "AAPL".to_string(),
            currency: Currency::from_str("USD").unwrap(),
            source: MarketDataSource::EodHistData.to_string(),
            priority: 1,
            factor: 1.0,
        };
        let start = Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0);
        let end = Utc.ymd(2020, 1, 31).and_hms_milli(23, 59, 59, 999);
        let quotes = eod.fetch_quote_history(&ticker, start, end).await.unwrap();
        assert_eq!(quotes.len(), 21);
        assert!(quotes[0].price != 0.0);
    }
}
