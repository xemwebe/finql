use super::{MarketQuoteError, MarketQuoteProvider};
use crate::datatypes::{
    date_time_helper::{to_time_offset_date_time, unix_to_offset_date_time},
    CashFlow, Quote, Ticker,
};
use async_trait::async_trait;
use std::convert::TryInto;
use time::OffsetDateTime;
use yahoo_finance_api as yahoo;

pub struct Yahoo {}

#[async_trait]
impl MarketQuoteProvider for Yahoo {
    /// Fetch latest quote
    async fn fetch_latest_quote(&self, ticker: &Ticker) -> Result<Quote, MarketQuoteError> {
        let yahoo = yahoo::YahooConnector::new()?;
        let response = yahoo.get_latest_quotes(&ticker.name, "1d").await?;
        let quote = response.last_quote()?;
        Ok(Quote {
            id: None,
            ticker: ticker.id.unwrap(),
            price: quote.close,
            time: unix_to_offset_date_time(quote.timestamp.try_into().unwrap()),
            volume: Some(quote.volume as f64),
        })
    }
    /// Fetch historic quotes between start and end date
    async fn fetch_quote_history(
        &self,
        ticker: &Ticker,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Vec<Quote>, MarketQuoteError> {
        let yahoo = yahoo::YahooConnector::new()?;
        let response = yahoo
            .get_quote_history(
                &ticker.name,
                to_time_offset_date_time(start),
                to_time_offset_date_time(end),
            )
            .await?;
        let yahoo_quotes = response.quotes()?;
        let mut quotes = Vec::new();
        for quote in &yahoo_quotes {
            let volume = Some(quote.volume as f64);
            let time = unix_to_offset_date_time(quote.timestamp.try_into().unwrap());
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
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Vec<CashFlow>, MarketQuoteError> {
        let yahoo = yahoo::YahooConnector::new()?;
        let response = yahoo
            .get_quote_history(
                &ticker.name,
                to_time_offset_date_time(start),
                to_time_offset_date_time(end),
            )
            .await?;
        let yahoo_dividends = response.dividends()?;
        let mut dividends = Vec::new();
        for dividend in &yahoo_dividends {
            let amount = dividend.amount;
            let time = unix_to_offset_date_time(dividend.date.try_into().unwrap());
            dividends.push(CashFlow::new(amount, ticker.currency, time.date()));
        }
        Ok(dividends)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use time::{macros::offset, OffsetDateTime};

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
        let start = OffsetDateTime::new_in_offset(
            time::Date::from_calendar_date(2020, time::Month::January, 1).unwrap(),
            time::Time::from_hms_milli(0, 0, 0, 0).unwrap(),
            offset!(-05:00),
        );
        let end = OffsetDateTime::new_in_offset(
            time::Date::from_calendar_date(2020, time::Month::January, 31).unwrap(),
            time::Time::from_hms_milli(23, 59, 59, 999).unwrap(),
            offset!(-05:00),
        );
        let quotes = yahoo
            .fetch_quote_history(&ticker, start, end)
            .await
            .unwrap();
        assert_eq!(quotes.len(), 21);
        assert!(quotes[0].price != 0.0);
    }
}
