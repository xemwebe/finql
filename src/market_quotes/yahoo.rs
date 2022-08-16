use super::{MarketQuoteError, MarketQuoteProvider};
use crate::datatypes::{date_time_helper::unix_to_date_time, CashFlow, Quote, Ticker};
use async_trait::async_trait;
use chrono::{DateTime, Local};
use yahoo_finance_api as yahoo;

pub struct Yahoo {}

#[async_trait]
impl MarketQuoteProvider for Yahoo {
    /// Fetch latest quote
    async fn fetch_latest_quote(&self, ticker: &Ticker) -> Result<Quote, MarketQuoteError> {
        let yahoo = yahoo::YahooConnector::new();
        let response = yahoo.get_latest_quotes(&ticker.name, "1d").await?;
        let quote = response.last_quote()?;
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
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> Result<Vec<Quote>, MarketQuoteError> {
        let yahoo = yahoo::YahooConnector::new();
        let response = yahoo
            .get_quote_history(&ticker.name, start.into(), end.into())
            .await?;
        let yahoo_quotes = response.quotes()?;
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

    /// Fetch historic dividend payments between start and end date
    async fn fetch_dividend_history(
        &self,
        ticker: &Ticker,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> Result<Vec<CashFlow>, MarketQuoteError> {
        let yahoo = yahoo::YahooConnector::new();
        let response = yahoo
            .get_quote_history(&ticker.name, start.into(), end.into())
            .await?;
        let yahoo_dividends = response.dividends()?;
        let mut dividends = Vec::new();
        for dividend in &yahoo_dividends {
            let amount = dividend.amount;
            let time = unix_to_date_time(dividend.date);
            dividends.push(CashFlow::new(
                amount,
                ticker.currency,
                time.naive_local().date(),
            ));
        }
        Ok(dividends)
    }
}

#[cfg(test)]
mod tests {
    use chrono::offset::TimeZone;
    use chrono_tz::America::New_York;
    use std::str::FromStr;

    use crate::datatypes::Currency;

    use super::*;
    use crate::market_quotes::MarketDataSource;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_yahoo_fetch_quote() {
        let yahoo = Yahoo {};
        let ticker = Ticker {
            id: Some(1),
            asset: 1,
            name: "AAPL".to_string(),
            currency: Currency::from_str("EUR").unwrap(),
            source: MarketDataSource::Yahoo.to_string(),
            priority: 1,
            factor: 1.0,
            tz: None,
            cal: None,
        };
        let quote = yahoo.fetch_latest_quote(&ticker).await.unwrap();
        assert!(quote.price != 0.0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_yahoo_fetch_history() {
        let yahoo = Yahoo {};
        let ticker = Ticker {
            id: Some(1),
            asset: 1,
            name: "AAPL".to_string(),
            currency: Currency::from_str("EUR").unwrap(),
            source: MarketDataSource::Yahoo.to_string(),
            priority: 1,
            factor: 1.0,
            tz: None,
            cal: None,
        };
        let start = New_York
            .ymd(2020, 1, 1)
            .and_hms_milli(0, 0, 0, 0)
            .with_timezone(&Local);
        let end = New_York
            .ymd(2020, 1, 31)
            .and_hms_milli(23, 59, 59, 999)
            .with_timezone(&Local);
        let quotes = yahoo
            .fetch_quote_history(&ticker, start, end)
            .await
            .unwrap();
        assert_eq!(quotes.len(), 21);
        assert!(quotes[0].price != 0.0);
    }
}
