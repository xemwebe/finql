///! Calculation of fx rates based on currency quotes

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use async_trait::async_trait;

use finql_data::{Asset, Currency, CurrencyConverter, CurrencyError, DataError, QuoteHandler, Quote, Ticker};


/// Insert fx rate quote in database including the inverse quote
pub async fn insert_fx_quote(
    fx_rate: f64,
    foreign: Currency,
    base: Currency,
    time: DateTime<Utc>,
    quotes: &dyn QuoteHandler,
) -> Result<(), DataError> {
    let foreign_id = quotes
        .insert_asset(&Asset {
            id: None,
            name: foreign.to_string(),
            wkn: None,
            isin: None,
            note: None,
        })
        .await.unwrap();
    let currency_pair = format!("{}/{}", foreign, base);
    let ticker_id = quotes
        .insert_ticker(&Ticker {
            id: None,
            name: currency_pair,
            asset: foreign_id,
            source: "manual".to_string(),
            priority: 10,
            currency: base,
            factor: 1.0,
        })
        .await.unwrap();
    quotes.insert_quote(&Quote {
        id: None,
        ticker: ticker_id,
        price: fx_rate,
        time,
        volume: None,
    }).await.unwrap();
    // Insert inverse fx quote
    let base_id = quotes
        .insert_asset(&Asset {
            id: None,
            name: base.to_string(),
            wkn: None,
            isin: None,
            note: None,
        })
        .await.unwrap();
    let currency_pair = format!("{}/{}", base, foreign);
    let ticker_id = quotes
        .insert_ticker(&Ticker {
            id: None,
            name: currency_pair,
            asset: base_id,
            source: "manual".to_string(),
            priority: 10,
            currency: foreign,
            factor: 1.0,
        })
        .await.unwrap();
    quotes.insert_quote(&Quote {
        id: None,
        ticker: ticker_id,
        price: 1.0 / fx_rate,
        time,
        volume: None,
    }).await.unwrap();
    Ok(())
}


/// Currency converter based of stored list of exchange rates, ignoring dates
pub struct SimpleCurrencyConverter {
    fx_rates: HashMap<String,HashMap<String,f64>>,
}

#[async_trait]
impl CurrencyConverter for SimpleCurrencyConverter {
    async fn fx_rate(&self, foreign_currency: Currency, domestic_currency: Currency, _time: DateTime<Utc>) -> Result<f64, CurrencyError> {
        Ok(self.fx_rates[&foreign_currency.to_string()][&domestic_currency.to_string()])
    }
}

impl SimpleCurrencyConverter {
    /// Create new container
    pub fn new() -> SimpleCurrencyConverter {
        SimpleCurrencyConverter{ fx_rates: HashMap::new() }
    }

    /// Insert or update the price of 1 unit of foreign currency in terms of domestic currency
    fn insert_fx_rate_one(&mut self, foreign_currency: Currency, domestic_currency: Currency, fx_rate: f64) {
        let for_key = foreign_currency.to_string();
        let dom_key = domestic_currency.to_string();
        match self.fx_rates.get_mut(&for_key) {
            Some(fx) => { fx.insert(dom_key, fx_rate); },
            None => {
                let mut new_map = HashMap::new();
                new_map.insert(dom_key, fx_rate);
                self.fx_rates.insert(for_key, new_map);
            }
        }
    }

    /// Insert or update the price of 1 unit of foreign currency in terms of domestic currency and its inverse rate
    pub fn insert_fx_rate(&mut self, foreign_currency: Currency, domestic_currency: Currency, fx_rate: f64) {
        self.insert_fx_rate_one(foreign_currency, domestic_currency, fx_rate);
        self.insert_fx_rate_one(domestic_currency, foreign_currency, 1./fx_rate);
    }
}

impl Default for SimpleCurrencyConverter{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::str::FromStr;

    use chrono::offset::TimeZone;
    use chrono::Utc;

    use finql_sqlite::SqliteDB;
    use crate::market::Market;

    async fn prepare_db(db: &dyn QuoteHandler) {
        let time = Utc.ymd(1970, 1, 1).and_hms_milli(0, 0, 1, 444);
        let eur = Currency::from_str("EUR").unwrap();
        let usd = Currency::from_str("USD").unwrap();
        insert_fx_quote(0.9, usd, eur, time, db).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_fx_rate() {
        let mut fx_db = SqliteDB::new("sqlite::memory:").await.unwrap();
        fx_db.init().await.unwrap();
        prepare_db(&fx_db).await;
        let tol = 1.0e-6_f64;
        let eur = Currency::from_str("EUR").unwrap();
        let usd = Currency::from_str("USD").unwrap();
        let time = Utc::now();
        let qh: Arc<dyn QuoteHandler+Send+Sync> = Arc::new(fx_db);
        let market = Market::new(qh);
        let fx = market.fx_rate(usd, eur, time).await.unwrap();
        assert_fuzzy_eq!(fx, 0.9, tol);
    }
}