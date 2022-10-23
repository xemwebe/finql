/// A market is either a container to store market data or
/// an adapter to receive and send market data from an external
/// source, e.g a database, files, or REST service.
/// Market data consist of non-static data, like interest rates,
/// asset prices, or foreign exchange rates.
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Local, NaiveDate};
use std::collections::BTreeMap;

use async_trait::async_trait;
use thiserror::Error;

use crate::datatypes::{
    date_time_helper::naive_date_to_date_time, Asset, Currency, CurrencyConverter, CurrencyError,
    CurrencyISOCode, QuoteHandler,
};

use crate::market_quotes::{self, MarketDataSourceError, MarketQuoteProvider};
use cal_calc::Calendar;

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
    CurrencyConversionError,
    #[error("date/time conversion error")]
    DateTimeError(#[from] crate::datatypes::date_time_helper::DateTimeError),
    #[error("Invalid market data source")]
    MarketDataSourceError(#[from] MarketDataSourceError),
    #[error("Invalid currency")]
    InvalidCurrency(#[from] CurrencyError),
    #[error("Cache access failed")]
    CacheFailure,
    #[error("Currency not found")]
    CurrencyNotFound,
}

#[derive(Clone)]
struct TimeRange {
    start: DateTime<Local>,
    end: DateTime<Local>,
}

/// Caching policy for Market
#[derive(Clone)]
enum CachePolicy {
    /// Do not cache any values
    None,
    /// If quote of not yet cached quote exists, fetch quotes for at least the given time range
    PredefinedPeriod(TimeRange),
}

async fn currency_map(db: Arc<dyn QuoteHandler + Sync + Send>) -> BTreeMap<i32, Currency> {
    let mut currency_map = BTreeMap::new();
    if let Ok(currency_vec) = db.get_all_currencies().await {
        for curr in currency_vec {
            currency_map.insert(curr.id.unwrap(), curr);
        }
    }
    currency_map
}

/// Container or adaptor to market data
struct MarketImpl {
    /// Stored calendars
    calendars: BTreeMap<String, Calendar>,
    /// Pre-fetched asset prices
    prices: RwLock<BTreeMap<i32, BTreeMap<DateTime<Local>, (f64, i32)>>>,
    /// collection of market data quotes provider
    providers: RwLock<BTreeMap<String, Arc<dyn MarketQuoteProvider + Sync + Send>>>,
    /// Quotes database
    db: Arc<dyn QuoteHandler + Sync + Send>,
    /// Caching policy
    cache_policy: RwLock<CachePolicy>,
    /// List of currency for fast access
    currencies: RwLock<BTreeMap<i32, Currency>>,
}

#[derive(Clone)]
pub struct Market {
    inner: Arc<MarketImpl>,
}

impl Market {
    pub async fn new(db: Arc<dyn QuoteHandler + Sync + Send>) -> Self {
        Self {
            inner: Arc::new(MarketImpl {
                // Set of default calendars
                calendars: generate_calendars(),
                providers: RwLock::new(BTreeMap::new()),
                prices: RwLock::new(BTreeMap::new()),
                db: db.clone(),
                cache_policy: RwLock::new(CachePolicy::None),
                currencies: RwLock::new(currency_map(db).await),
            }),
        }
    }

    pub async fn new_with_date_range(
        db: Arc<dyn QuoteHandler + Sync + Send>,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Self, MarketError> {
        let cache_policy = CachePolicy::PredefinedPeriod(TimeRange {
            start: naive_date_to_date_time(&start, 0, None)?,
            end: naive_date_to_date_time(&end, 24, None)?,
        });
        Ok(Self {
            inner: Arc::new(MarketImpl {
                // Set of default calendars
                calendars: generate_calendars(),
                providers: RwLock::new(BTreeMap::new()),
                prices: RwLock::new(BTreeMap::new()),
                db: db.clone(),
                cache_policy: RwLock::new(cache_policy),
                currencies: RwLock::new(currency_map(db).await),
            }),
        })
    }

    pub fn db(&self) -> Arc<dyn QuoteHandler + Sync + Send> {
        self.inner.db.clone()
    }

    pub fn set_cache_period(
        &self,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> Result<(), MarketError> {
        if let Ok(mut policy) = self.inner.cache_policy.write() {
            let new_policy = match &*policy {
                CachePolicy::None => CachePolicy::PredefinedPeriod(TimeRange { start, end }),
                CachePolicy::PredefinedPeriod(time_range) => {
                    CachePolicy::PredefinedPeriod(TimeRange {
                        start: start.min(time_range.start),
                        end: end.max(time_range.end),
                    })
                }
            };
            *policy = new_policy;
        } else {
            return Err(MarketError::CacheFailure);
        }
        Ok(())
    }

    /// Get calendar from market
    pub fn get_calendar(&self, name: &str) -> Result<&Calendar, MarketError> {
        if self.inner.calendars.contains_key(name) {
            Ok(&self.inner.calendars[name])
        } else {
            Err(MarketError::CalendarNotFound)
        }
    }

    /// Store currency in cache
    fn store_currency_in_cache(&self, currency: Currency) {
        if let Some(id) = currency.id {
            if let Ok(mut currencies) = self.inner.currencies.write() {
                (*currencies).insert(id, currency.clone());
            }
        }
    }

    /// Get currency from market given the ISO code; if it does not exist, create new currency with default rounding digits
    pub async fn get_currency(&self, iso_code: CurrencyISOCode) -> Result<Currency, MarketError> {
        if let Ok(currencies) = self.inner.currencies.read() {
            for currency in &(*currencies) {
                if currency.1.iso_code == iso_code {
                    return Ok(*currency.1);
                }
            }
        }
        let currency = self.inner.db.get_or_new_currency(iso_code).await?;
        self.store_currency_in_cache(currency);
        Ok(currency)
    }

    /// Get currency given a string
    pub async fn get_currency_from_str(
        &self,
        currency_string: &str,
    ) -> Result<Currency, MarketError> {
        let iso_code = CurrencyISOCode::new(currency_string)?;
        self.get_currency(iso_code).await
    }

    /// Get currency from market
    pub async fn get_currency_by_id(&self, currency_id: i32) -> Result<Currency, MarketError> {
        if let Ok(currencies) = self.inner.currencies.read() {
            if let Some(currency) = (*currencies).get(&currency_id) {
                return Ok(*currency);
            }
        }
        if let Ok(Asset::Currency(currency)) = self.inner.db.get_asset_by_id(currency_id).await {
            self.store_currency_in_cache(currency);
            return Ok(currency);
        }
        Err(MarketError::CurrencyNotFound)
    }

    /// Add market data provider
    pub fn add_provider(&self, name: String, provider: Arc<dyn MarketQuoteProvider + Sync + Send>) {
        if let Ok(mut providers) = self.inner.providers.write() {
            (*providers).insert(name, provider);
        }
    }

    /// Fetch latest quotes for all active ticker
    /// Returns a list of ticker for which the update failed.
    pub async fn update_quotes(&self) -> Result<Vec<i32>, MarketError> {
        let tickers = self.inner.db.get_all_ticker().await?;
        let mut failed_ticker = Vec::new();
        let providers = self
            .inner
            .providers
            .read()
            .map_err(|_| MarketError::CacheFailure)?;
        for ticker in tickers {
            if let Some(provider) = (*providers).get(&ticker.source) {
                if market_quotes::update_ticker((*provider).clone(), &ticker, self.inner.db.clone())
                    .await
                    .is_err()
                {
                    failed_ticker.push(ticker.id.unwrap());
                }
            }
        }
        Ok(failed_ticker)
    }

    /// Update latest quote for a specific ticker id
    pub async fn update_quote_for_ticker(&self, ticker_id: i32) -> Result<(), MarketError> {
        let ticker = self.inner.db.get_ticker_by_id(ticker_id).await?;
        let provider = if let Ok(providers) = self.inner.providers.read() {
            if let Some(provider) = (*providers).get(&ticker.source) {
                Some(provider.clone())
            } else {
                None
            }
        } else {
            None
        };
        if let Some(provider) = provider {
            market_quotes::update_ticker(provider, &ticker, self.inner.db.clone()).await?;
        }
        Ok(())
    }

    /// Fetch latest quotes for all active ticker
    pub async fn update_quote_history(
        &self,
        ticker_id: i32,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> Result<(), MarketError> {
        let ticker = self.inner.db.get_ticker_by_id(ticker_id).await?;
        let provider = if let Ok(providers) = self.inner.providers.read() {
            if let Some(provider) = (*providers).get(&ticker.source) {
                Some(provider.clone())
            } else {
                None
            }
        } else {
            None
        };
        if let Some(provider) = provider {
            market_quotes::update_ticker_history(
                provider,
                &ticker,
                self.inner.db.clone(),
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
        asset_id: i32,
        start: DateTime<Local>,
        end: DateTime<Local>,
    ) -> Result<(), MarketError> {
        let tickers = self.inner.db.get_all_ticker_for_asset(asset_id).await?;
        if let Ok(providers) = self.inner.providers.read() {
            for ticker in tickers {
                if let Some(provider) = providers.get(&ticker.source) {
                    market_quotes::update_ticker_history(
                        provider.clone(),
                        &ticker,
                        self.inner.db.clone(),
                        start,
                        end,
                    )
                    .await?;
                }
            }
        }
        Ok(())
    }

    pub fn try_from_cache(&self, asset_id: i32, time: DateTime<Local>) -> Option<(f64, i32)> {
        if let Ok(prices) = self.inner.prices.read() {
            if let Some(series) = (*prices).get(&asset_id) {
                return series.range(..time).last().map(|entry| *entry.1);
            }
        }
        None
    }

    pub async fn get_asset_price(
        &self,
        asset_id: i32,
        currency: Currency,
        time: DateTime<Local>,
    ) -> Result<f64, MarketError> {
        let (price, quote_currency_id) = if let Some((quote, curr)) =
            self.try_from_cache(asset_id, time)
        {
            (quote, curr)
        } else {
            let cache_policy = if let Ok(cache_policy) = self.inner.cache_policy.read() {
                (*cache_policy).clone()
            } else {
                CachePolicy::None
            };
            match &cache_policy {
                CachePolicy::None => {
                    let (quote, currency) = self
                        .inner
                        .db
                        .get_last_quote_before_by_id(asset_id, time)
                        .await?;
                    (quote.price, currency.id.unwrap())
                }
                CachePolicy::PredefinedPeriod(time_range) => {
                    let date_start = time.date().and_hms(0, 0, 0);
                    let date_end = time.date().and_hms_milli(23, 59, 59, 999);
                    let start = std::cmp::min(time_range.start, date_start);
                    let end = std::cmp::max(time_range.end, date_end);
                    let quotes = self
                        .inner
                        .db
                        .get_quotes_in_range_by_id(asset_id, start, end)
                        .await?;
                    if let Ok(mut prices) = self.inner.prices.write() {
                        // add quotes to cache in this
                        let asset_prices = (*prices).entry(asset_id).or_insert_with(BTreeMap::new);
                        for quote in quotes {
                            asset_prices.insert(quote.0.time, (quote.0.price, quote.1));
                        }
                    }
                    self.try_from_cache(asset_id, time)
                        .ok_or(MarketError::CacheFailure)?
                }
            }
        };
        if currency.id == Some(quote_currency_id) {
            Ok(price)
        } else {
            let quote_currency = self.get_currency_by_id(quote_currency_id).await?;
            let fx_rate = self
                .fx_rate(quote_currency, currency, time)
                .await
                .map_err(|_| MarketError::CurrencyConversionError)?;
            Ok(price * fx_rate)
        }
    }
}

#[async_trait]
impl CurrencyConverter for Market {
    async fn fx_rate(
        &self,
        base_currency: Currency,
        quote_currency: Currency,
        time: DateTime<Local>,
    ) -> Result<f64, CurrencyError> {
        if base_currency == quote_currency {
            return Ok(1.0);
        } else {
            let (fx_quote, quote_curr_id) = if let Some((fx_quote, quote_curr_id)) = self
                .try_from_cache(
                    base_currency.id.ok_or(CurrencyError::ConversionFailed)?,
                    time,
                ) {
                (fx_quote, quote_curr_id)
            } else {
                let fx_quote = self
                    .inner
                    .db
                    .get_last_fx_quote_before(&base_currency.iso_code, time)
                    .await
                    .map_err(|_| CurrencyError::ConversionFailed)?;
                (fx_quote.0.price, fx_quote.1.id.unwrap())
            };
            if quote_currency.id == Some(quote_curr_id) {
                return Ok(fx_quote);
            }
        }
        Err(CurrencyError::ConversionFailed)
    }
}

/// Generate fixed set of some calendars for testing purposes only
pub fn generate_calendars() -> BTreeMap<String, Calendar> {
    use cal_calc::{target_holidays, uk_settlement_holidays};

    let mut calendars = BTreeMap::new();

    let uk_cal = Calendar::calc_calendar(&uk_settlement_holidays(), 1990, 2050);
    calendars.insert("uk".to_string(), uk_cal);

    let target_cal = Calendar::calc_calendar(&target_holidays(), 1990, 2050);
    calendars.insert("TARGET".to_string(), target_cal);

    calendars
}
