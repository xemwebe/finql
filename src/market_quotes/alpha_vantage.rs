use chrono::{DateTime, Utc, Duration};
use async_trait::async_trait;
use tokio_compat_02::FutureExt;

use alpha_vantage as alpha;

use finql_data::{Quote, Ticker};

use super::{MarketQuoteError, MarketQuoteProvider};
use crate::date_time_helper::date_time_from_str_standard;

pub struct AlphaVantage {
    connector: alpha::user::APIKey,
}

impl AlphaVantage {
    pub fn new(token: String) -> AlphaVantage {
        AlphaVantage {
            connector: alpha::set_api(&token),
        }
    }
}

#[async_trait]
impl MarketQuoteProvider for AlphaVantage {
    /// Fetch latest quote
    async fn fetch_latest_quote(&self, ticker: &Ticker) -> Result<Quote, MarketQuoteError> {
        let alpha_quote = self
            .connector
            .quote(&ticker.name)
            .compat()
            .await
            .map_err(|e| MarketQuoteError::FetchFailed(e.to_string()))?;
        let time = date_time_from_str_standard(alpha_quote.last_trading(), 0)?;
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
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Quote>, MarketQuoteError> {
        let now = Utc::now();
        // This estimate is conservative, since we expect less business days than calendar
        // days, but to be on the conservative side, we use calendar days
        let output_size = if now.signed_duration_since(start) > Duration::days(100) {
            alpha::util::OutputSize::Full
        } else {
            alpha::util::OutputSize::Compact
        };
        let alpha_quotes = self
            .connector
            .stock_time(
                alpha::util::StockFunction::Daily,
                &ticker.name,
                alpha::util::TimeSeriesInterval::None,
                output_size,
            )
            .compat()
            .await
            .map_err(|e| MarketQuoteError::FetchFailed(e.to_string()))?;

        let mut quotes = Vec::new();
        for quote in alpha_quotes.entry() {
            let time = date_time_from_str_standard(quote.time(), 18)?;
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
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use chrono::offset::TimeZone;
    use tokio_test::block_on;

    use finql_data::Currency;

    use super::*;
    use crate::market_quotes::MarketDataSource;

    #[test]
    fn test_alpha_fetch_quote() {
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
        };
        let quote = block_on(alpha.fetch_latest_quote(&ticker)).unwrap();
        assert!(quote.price != 0.0);
    }

    #[test]
    fn test_alpha_fetch_history() {
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
        };
        let start = Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0);
        let end = Utc.ymd(2020, 1, 31).and_hms_milli(23, 59, 59, 999);
        let quotes = block_on(alpha.fetch_quote_history(&ticker, start, end)).unwrap();
        assert_eq!(quotes.len(), 21);
        assert!(quotes[0].price != 0.0);
    }
}
