use async_trait::async_trait;
use gurufocus_api as gfapi;
use std::str::FromStr;
use time::OffsetDateTime;

use super::{MarketQuoteError, MarketQuoteProvider};

use crate::datatypes::{
    date_time_helper::{
        date_from_str, date_to_offset_date_time, offset_date_time_from_str_american,
        unix_to_offset_date_time,
    },
    CashFlow, Currency, Quote, Ticker,
};

type DividendHistory = Vec<gfapi::Dividend>;

pub struct GuruFocus {
    connector: gfapi::GuruFocusConnector,
}

impl GuruFocus {
    pub fn new(token: String) -> GuruFocus {
        GuruFocus {
            connector: gfapi::GuruFocusConnector::new(token),
        }
    }
}

#[async_trait]
impl MarketQuoteProvider for GuruFocus {
    /// Fetch latest quote
    async fn fetch_latest_quote(&self, ticker: &Ticker) -> Result<Quote, MarketQuoteError> {
        let prices = self.connector.get_quotes(&[&ticker.name]).await?;

        let quote: gfapi::Quote = serde_json::from_value(prices)?;

        let time = unix_to_offset_date_time(quote.timestamp as u64);
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
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Vec<Quote>, MarketQuoteError> {
        let gf_quotes = self.connector.get_price_hist(&ticker.name).await?;

        let gf_quotes: Vec<(String, f64)> = serde_json::from_value(gf_quotes)?;

        let mut quotes = Vec::new();
        for (timestamp, price) in &gf_quotes {
            let time = offset_date_time_from_str_american(timestamp, 18, ticker.tz.clone())?;
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
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Vec<CashFlow>, MarketQuoteError> {
        let gf_dividends = self.connector.get_dividend_history(&ticker.name).await?;
        let dividends: DividendHistory = serde_json::from_value(gf_dividends)?;
        let mut div_cash_flows = Vec::new();
        for div in dividends {
            let pay_date = date_from_str(&div.pay_date, "%Y-%m-%d")?;
            let pay_date_time = date_to_offset_date_time(&pay_date, 18, ticker.tz.clone())?;
            if pay_date_time >= start && pay_date_time <= end {
                let currency = Currency::from_str(&div.currency)?;
                div_cash_flows.push(CashFlow::new(div.amount.into(), currency, pay_date));
            }
        }
        Ok(div_cash_flows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datatypes::Currency;
    use crate::market_quotes::MarketDataSource;
    use std::env;
    use std::str::FromStr;
    use time::{macros::offset, OffsetDateTime};

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
            tz: None,
            cal: None,
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
            tz: None,
            cal: None,
        };
        let start = OffsetDateTime::new_in_offset(
            time::Date::from_calendar_date(2020, time::Month::January, 1).unwrap(),
            time::Time::from_hms_milli(0, 0, 0, 0).unwrap(),
            offset!(UTC),
        );
        let end = OffsetDateTime::new_in_offset(
            time::Date::from_calendar_date(2020, time::Month::January, 31).unwrap(),
            time::Time::from_hms_milli(23, 59, 59, 999).unwrap(),
            offset!(UTC),
        );
        let quotes = gf.fetch_quote_history(&ticker, start, end).await.unwrap();
        assert!(quotes.len() > 15);
        assert!(quotes[0].price != 0.0);
    }
}
