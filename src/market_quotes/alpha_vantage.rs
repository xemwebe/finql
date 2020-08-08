use super::{MarketQuoteError, MarketQuoteProvider};
use crate::date_time_helper::date_time_from_str_standard;
use crate::quote::{Quote, Ticker};
use alpha_vantage as alpha;
use chrono::{DateTime, Utc};
use async_trait::async_trait;

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
        let alpha_quotes = self
            .connector
            .stock_time(
                alpha::util::StockFunction::Daily,
                &ticker.name,
                alpha::util::TimeSeriesInterval::None,
                alpha::util::OutputSize::None,
            )
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
    use super::*;
    use crate::currency::Currency;
    use crate::quote::MarketDataSource;
    use chrono::offset::TimeZone;
    use std::str::FromStr;

    #[test]
    fn test_alpha_fetch_quote() {
        let token = "demo".to_string();
        let alpha = AlphaVantage::new(token);
        let ticker = Ticker {
            id: Some(1),
            asset: 1,
            name: "IBM".to_string(),
            currency: Currency::from_str("USD").unwrap(),
            source: MarketDataSource::AlphaVantage,
            priority: 1,
            factor: 1.0,
        };
        let quote = alpha.fetch_latest_quote(&ticker).unwrap();
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
            source: MarketDataSource::AlphaVantage,
            priority: 1,
            factor: 1.0,
        };
        let start = Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0);
        let end = Utc.ymd(2020, 1, 31).and_hms_milli(23, 59, 59, 999);
        let quotes = alpha.fetch_quote_history(&ticker, start, end).unwrap();
        assert_eq!(quotes.len(), 21);
        assert!(quotes[0].price != 0.0);
    }
}
