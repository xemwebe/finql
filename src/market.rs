/// A market is either a container to store market data or
/// an adapter to receive and send market data from an external
/// source, e.g a database, files, or REST service.
/// Market data consist of non-static data, like interest rates,
/// asset prices, or foreign exchange rates.
use std::ops::Deref;
use std::sync::Arc;

use chrono::{DateTime, NaiveDate, Local, Weekday};
use std::collections::BTreeMap;

use async_trait::async_trait;
use thiserror::Error;

use crate::datatypes::{Currency, CurrencyConverter, CurrencyError, QuoteHandler, date_time_helper::naive_date_to_date_time};
use crate::time_period::TimePeriod;

use crate::calendar::{Calendar, Holiday, NthWeek};
use crate::market_quotes;
use crate::market_quotes::{MarketQuoteProvider, MarketDataSourceError};

/// Error related to market data object
#[derive(Error, Debug)]
pub enum MarketError {
    #[error("Unknown calendar")]
    CalendarNotFound,
    #[error("Market quote error")]
    MarketQuoteError(#[from] market_quotes::MarketQuoteError),
    #[error("Database error")]
    DBError(#[from] crate::datatypes::DataError),
    #[error("Missing market data provider token")]
    MissingProviderToken,
    #[error("Currency conversion failure")]
    CurrencyError,
    #[error("date/time conversion error")]
    DateTimeError(#[from] crate::datatypes::date_time_helper::DateTimeError),
    #[error("Invalid market data source")]
    MarketDataSourceError(#[from] MarketDataSourceError),
}

/// Container or adaptor to market data
#[derive(Clone)]
pub struct Market {
    calendars: BTreeMap<String, Calendar>,
    /// collection of market data quotes provider
    provider: BTreeMap<String, Arc<dyn MarketQuoteProvider+Sync+Send>>,
    /// Quotes database
    db: Arc<dyn QuoteHandler+Sync+Send>,
}

impl Market {
    pub fn new(db: Arc<dyn QuoteHandler+Sync+Send>) -> Market {
        Market {
            // Set of default calendars
            calendars: generate_calendars(),
            provider: BTreeMap::new(),
            db,
        }
    }

    pub fn db(&self) -> Arc<dyn QuoteHandler+Sync+Send> {
        self.db.clone()
    }

    /// Get calendar from market
    pub fn get_calendar(&self, name: &str) -> Result<&Calendar, MarketError> {
        if self.calendars.contains_key(name) {
            Ok(&self.calendars[name])
        } else {
            Err(MarketError::CalendarNotFound)
        }
    }

    /// Add market data provider
    pub fn add_provider(&mut self, name: String, provider: Arc<dyn MarketQuoteProvider+Sync+Send>) {
        self.provider.insert(name, provider);
    }

    /// Fetch latest quotes for all active ticker
    /// Returns a list of ticker for which the update failed.
    pub async fn update_quotes(&self) -> Result<Vec<usize>, MarketError> {
        let tickers = self.db.get_all_ticker().await?;
        let mut failed_ticker = Vec::new();
        for ticker in tickers {
            let provider = self.provider.get(&ticker.source);
            if provider.is_some()
                && market_quotes::update_ticker(
                    provider.unwrap().deref(),
                    &ticker,
                    self.db.clone(),
                )
                .await
                .is_err()
            {
                failed_ticker.push(ticker.id.unwrap());
            }
        }
        Ok(failed_ticker)
    }

    /// Fetch latest quotes for all active ticker
    pub async fn update_quote_history(
        &self,
        ticker_id: usize,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> Result<(), MarketError> {
        let ticker = self.db.get_ticker_by_id(ticker_id).await?;
        let provider = self.provider.get(&ticker.source);
        if provider.is_some() {
            market_quotes::update_ticker_history(
                provider.unwrap().deref(),
                &ticker,
                self.db.clone(),
                start,
                end,
            )
            .await?;
        }
        Ok(())
    }

    /// Update quote history using all tickers of given asset
    pub async fn update_quote_history_for_asset(
        &self,
        asset_id: usize,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> Result<(), MarketError> {
        let tickers = self.db.get_all_ticker_for_asset(asset_id).await?;
        for ticker in tickers {
            let provider = self.provider.get(&ticker.source);
            if provider.is_some() {
                market_quotes::update_ticker_history(
                    provider.unwrap().deref(),
                    &ticker,
                    self.db.clone(),
                    start,
                    end,
                )
                .await?;
            }
        }
        Ok(())
    }

    pub async fn get_asset_price(&self, asset_id: usize, currency: Currency, date: NaiveDate) -> Result<f64, MarketError> {
        let quote_curr = self.db.get_last_quote_before_by_id(asset_id, naive_date_to_date_time(&date, 18, None)?).await;
        let (price, quote_currency) = if let Ok((quote, currency)) = quote_curr {
            (quote.price, currency)            
        } else {
            // if no valid quote could be found in database, try to fetch quotes for previous week and try again
            let one_week_before = "-7D".parse::<TimePeriod>().unwrap();
            let date_one_week_before = one_week_before.add_to(date, None);
            self.update_quote_history_for_asset(asset_id, naive_date_to_date_time(&date_one_week_before, 0, None)?, 
                naive_date_to_date_time(&date, 20, None)?).await?;
            let (quote, currency) = self.db.get_last_quote_before_by_id(asset_id, naive_date_to_date_time(&date, 20, None)?).await?;
            (quote.price, currency)
        };
        if currency == quote_currency {
            Ok(price)
        } else  {
            let fx_rate = self.fx_rate(currency, quote_currency, naive_date_to_date_time(&date, 20, None)?).await
                .map_err(|_| MarketError::CurrencyError)?;
            Ok(price*fx_rate)
        }

    }
}

#[async_trait]
impl CurrencyConverter for Market {
    async fn fx_rate(
        &self,
        foreign: Currency,
        base: Currency,
        time: DateTime<Local>,
    ) -> Result<f64, CurrencyError> {
        if foreign == base {
            return Ok(1.0);
        } else {
            let (fx_quote, quote_currency) = self.db.deref().deref()
                .get_last_quote_before(&foreign.to_string(), time)
                .await
                .map_err(|_| CurrencyError::ConversionFailed)?;
            if quote_currency == base {
                return Ok(fx_quote.price);
            }
        }
        Err(CurrencyError::ConversionFailed)
    }
}

/// Generate fixed set of some calendars for testing purposes only
pub fn generate_calendars() -> BTreeMap<String, Calendar> {
    let mut calendars = BTreeMap::new();
    let uk_settlement_holidays = vec![
        // Saturdays
        Holiday::WeekDay(Weekday::Sat),
        // Sundays
        Holiday::WeekDay(Weekday::Sun),
        // New Year's day
        Holiday::MovableYearlyDay {
            month: 1,
            day: 1,
            first: None,
            last: None,
        },
        // Good Friday
        Holiday::EasterOffset {
            offset: -2,
            first: None,
            last: None,
        },
        // Easter Monday
        Holiday::EasterOffset {
            offset: 1,
            first: None,
            last: None,
        },
        // first Monday of May, moved two times in history to 8th of May
        Holiday::MonthWeekday {
            month: 5,
            weekday: Weekday::Mon,
            nth: NthWeek::First,
            first: None,
            last: Some(1994),
        },
        Holiday::SingularDay(NaiveDate::from_ymd(1995, 5, 8)),
        Holiday::MonthWeekday {
            month: 5,
            weekday: Weekday::Mon,
            nth: NthWeek::First,
            first: Some(1996),
            last: Some(2019),
        },
        Holiday::SingularDay(NaiveDate::from_ymd(2020, 5, 8)),
        Holiday::MonthWeekday {
            month: 5,
            weekday: Weekday::Mon,
            nth: NthWeek::First,
            first: Some(2021),
            last: None,
        },
        // last Monday of May (Spring Bank Holiday), has been skipped two times
        Holiday::MonthWeekday {
            month: 5,
            weekday: Weekday::Mon,
            nth: NthWeek::Last,
            first: None,
            last: Some(2001),
        },
        Holiday::MonthWeekday {
            month: 5,
            weekday: Weekday::Mon,
            nth: NthWeek::Last,
            first: Some(2003),
            last: Some(2011),
        },
        Holiday::MonthWeekday {
            month: 5,
            weekday: Weekday::Mon,
            nth: NthWeek::Last,
            first: Some(2013),
            last: None,
        },
        // last Monday of August (Summer Bank Holiday)
        Holiday::MonthWeekday {
            month: 8,
            weekday: Weekday::Mon,
            nth: NthWeek::Last,
            first: None,
            last: None,
        },
        // Christmas
        Holiday::MovableYearlyDay {
            month: 12,
            day: 25,
            first: None,
            last: None,
        },
        // Boxing Day
        Holiday::MovableYearlyDay {
            month: 12,
            day: 26,
            first: None,
            last: None,
        },
        // Golden Jubilee
        Holiday::SingularDay(NaiveDate::from_ymd(2002, 6, 3)),
        // Special Spring Holiday
        Holiday::SingularDay(NaiveDate::from_ymd(2002, 6, 4)),
        // Royal Wedding
        Holiday::SingularDay(NaiveDate::from_ymd(2011, 4, 29)),
        // Diamond Jubilee
        Holiday::SingularDay(NaiveDate::from_ymd(2012, 6, 4)),
        // Special Spring Holiday
        Holiday::SingularDay(NaiveDate::from_ymd(2012, 6, 5)),
        // Introduction of EUR
        Holiday::SingularDay(NaiveDate::from_ymd(1999, 12, 31)),
    ];
    let uk_cal = Calendar::calc_calendar(&uk_settlement_holidays, 1990, 2050);
    calendars.insert("uk".to_string(), uk_cal);

    let target_holidays = vec![
        // Saturdays
        Holiday::WeekDay(Weekday::Sat),
        // Sundays
        Holiday::WeekDay(Weekday::Sun),
        // New Year's day
        Holiday::YearlyDay {
            month: 1,
            day: 1,
            first: None,
            last: None,
        },
        // Good Friday
        Holiday::EasterOffset {
            offset: -2,
            first: Some(2000),
            last: None,
        },
        // Easter Monday
        Holiday::EasterOffset {
            offset: 1,
            first: Some(2000),
            last: None,
        },
        // Labour Day
        Holiday::YearlyDay {
            month: 5,
            day: 1,
            first: Some(2000),
            last: None,
        },
        // Labour Day
        Holiday::YearlyDay {
            month: 5,
            day: 1,
            first: Some(2000),
            last: None,
        },
        // Labour Day
        Holiday::YearlyDay {
            month: 5,
            day: 1,
            first: Some(2000),
            last: None,
        },
        Holiday::SingularDay(NaiveDate::from_ymd(1995, 5, 8)),
    ];
    let target_cal = Calendar::calc_calendar(&target_holidays, 1990, 2050);
    calendars.insert("TARGET".to_string(), target_cal);

    calendars
}
