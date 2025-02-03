use async_trait::async_trait;
use chrono::{DateTime, Local};
use reqwest;

use alpha_vantage as alpha;

use crate::datatypes::{date_time_helper::date_time_from_str_standard, CashFlow, Quote, Ticker};

use super::{MarketQuoteError, MarketQuoteProvider};

pub struct AlphaVantage {
    token: String,
}

impl AlphaVantage {
    pub fn new(token: String) -> AlphaVantage {
        AlphaVantage { token }
    }
}

#[async_trait]
impl MarketQuoteProvider for AlphaVantage {
    /// Fetch latest quote
    async fn fetch_latest_quote(&self, ticker: &Ticker) -> Result<Quote, MarketQuoteError> {
        let api_key = alpha::set_api(&self.token, reqwest::Client::new());
        let alpha_quote = api_key.quote(&ticker.name).json().await.unwrap();
        let time = date_time_from_str_standard(alpha_quote.last_trading(), 0, ticker.tz.clone())?;
        Ok(Quote {
            id: None,
            ticker: ticker.id.unwrap(),
            price: alpha_quote.price(),
            time,
            volume: Some(alpha_quote.volume() as f64),
        })
    }
    /// Fetch historic quotes between start and end date
    async fn fetch_quote_history(
        &self,
        ticker: &Ticker,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> Result<Vec<Quote>, MarketQuoteError> {
        let api_key = alpha::set_api(&self.token, reqwest::Client::new());
        let alpha_quotes = api_key
            .stock_time(alpha::stock_time::StockFunction::Daily, &ticker.name)
            .json()
            .await?;

        let mut quotes = Vec::new();
        for quote in alpha_quotes.data().iter() {
            let time = date_time_from_str_standard(quote.time(), 18, ticker.tz.clone())?;
            if time >= start && time <= end {
                quotes.push(Quote {
                    id: None,
                    ticker: ticker.id.unwrap(),
                    price: quote.close(),
                    time,
                    volume: Some(quote.volume() as f64),
                })
            }
        }
        Ok(quotes)
    }

    /// Fetch historic dividend payments between start and end date
    async fn fetch_dividend_history(
        &self,
        _ticker: &Ticker,
        _start: DateTime<Local>,
        _end: DateTime<Local>,
    ) -> Result<Vec<CashFlow>, MarketQuoteError> {
        Err(MarketQuoteError::UnexpectedError(
            "The Alpha Vantage API does not support fetching dividends".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use chrono::offset::TimeZone;
    use std::str::FromStr;

    use crate::datatypes::Currency;

    use super::*;
    use crate::market_quotes::MarketDataSource;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_alpha_fetch_quote() {
        let token = "demo".to_string();
        let alpha = AlphaVantage::new(token);
        let ticker = Ticker {
            id: Some(1),
            asset: 1,
            name: "IBM".to_string(),
            currency: Currency::from_str("USD").unwrap(),
            source: "alphavantage".to_string(),
            priority: 1,
            factor: 1.0,
            tz: None,
            cal: None,
        };
        let quote = alpha.fetch_latest_quote(&ticker).await.unwrap();
        assert!(quote.price != 0.0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_alpha_fetch_history() {
        let token = "demo".to_string();
        let alpha = AlphaVantage::new(token);
        let ticker = Ticker {
            id: Some(1),
            asset: 1,
            name: "IBM".to_string(),
            currency: Currency::from_str("USD").unwrap(),
            source: MarketDataSource::AlphaVantage.to_string(),
            priority: 1,
            factor: 1.0,
            tz: None,
            cal: None,
        };
        let start = Local.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0);
        let end = Local.ymd(3000, 1, 31).and_hms_milli(23, 59, 59, 999);
        let quotes = alpha
            .fetch_quote_history(&ticker, start, end)
            .await
            .unwrap();
        assert!(quotes.len() != 0);
        assert!(quotes[0].price != 0.0);
    }
}
