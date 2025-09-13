use crate::datatypes::{
    date_time_helper::{
        date_from_str, date_to_offset_date_time, offset_date_time_from_str_standard,
        unix_to_offset_date_time,
    },
    CashFlow, Currency, Quote, Ticker,
};
use async_trait::async_trait;
use eodhistoricaldata_api as eod_api;
use std::{convert::TryFrom, str::FromStr};
use time::{Date, Month, OffsetDateTime};

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
        let eod_quote = self.connector.get_latest_quote(&ticker.name).await?;

        let time = unix_to_offset_date_time(eod_quote.timestamp as u64);
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
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Vec<Quote>, MarketQuoteError> {
        let eod_quotes = self
            .connector
            .get_quote_history(
                &ticker.name,
                Date::from_calendar_date(
                    start.date().year(),
                    Month::try_from(start.date().month() as u8)?,
                    start.date().day() as u8,
                )
                .unwrap(),
                Date::from_calendar_date(
                    end.date().year(),
                    Month::try_from(end.date().month() as u8)?,
                    end.date().day() as u8,
                )
                .unwrap(),
            )
            .await?;

        let mut quotes = Vec::new();
        for quote in &eod_quotes {
            let time = offset_date_time_from_str_standard(&quote.date, 18, ticker.tz.clone())?;
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
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Vec<CashFlow>, MarketQuoteError> {
        let dividends_since_start = self
            .connector
            .get_dividend_history(
                &ticker.name,
                Date::from_calendar_date(
                    start.date().year(),
                    Month::try_from(start.date().month() as u8)?,
                    start.date().day() as u8,
                )
                .unwrap(),
            )
            .await?;
        let mut div_cash_flows = Vec::new();
        for div in dividends_since_start {
            let pay_date = date_from_str(&div.payment_date, "%Y-%m-%d")?;
            if date_to_offset_date_time(&pay_date, 18, ticker.tz.clone())? <= end {
                let currency = Currency::from_str(&div.currency)?;
                div_cash_flows.push(CashFlow::new(div.value, currency, pay_date));
            }
        }
        Ok(div_cash_flows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datatypes::{date_time_helper::make_offset_time, Currency};
    use crate::market_quotes::MarketDataSource;
    use std::str::FromStr;

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
            tz: None,
            cal: None,
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
            tz: None,
            cal: None,
        };
        let start = make_offset_time(2020, 1, 1, 0, 0, 0).unwrap();
        let end = make_offset_time(2020, 1, 31, 23, 59, 59).unwrap();
        let quotes = eod.fetch_quote_history(&ticker, start, end).await.unwrap();
        assert_eq!(quotes.len(), 21);
        assert!(quotes[0].price != 0.0);
    }
}
