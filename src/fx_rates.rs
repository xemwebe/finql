///! Calculation of fx rates based on currency quotes
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use async_trait::async_trait;
use chrono::{DateTime, Local};

use crate::datatypes::{
    Asset, Currency, CurrencyConverter, CurrencyError, DataError, DataItem, Quote, QuoteHandler,
    Ticker,
};

/// Insert fx rate quote in database including the inverse quote
pub async fn insert_fx_quote(
    fx_rate: f64,
    foreign: Currency,
    base: Currency,
    time: DateTime<Local>,
    quotes: Arc<dyn QuoteHandler + Send + Sync>,
) -> Result<(), DataError> {
    let foreign_id = if let Ok(id) = foreign.get_id() {
        id
    } else {
        quotes.insert_asset(&Asset::Currency(foreign)).await?
    };
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
            tz: None,
            cal: None,
        })
        .await?;
    quotes
        .insert_quote(&Quote {
            id: None,
            ticker: ticker_id,
            price: fx_rate,
            time,
            volume: None,
        })
        .await?;
    // Insert inverse fx quote
    let base_id = if let Ok(id) = base.get_id() {
        id
    } else {
        quotes.insert_asset(&Asset::Currency(base)).await?
    };
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
            tz: None,
            cal: None,
        })
        .await?;
    quotes
        .insert_quote(&Quote {
            id: None,
            ticker: ticker_id,
            price: 1.0 / fx_rate,
            time,
            volume: None,
        })
        .await?;
    Ok(())
}

/// Currency converter based of stored list of exchange rates, ignoring dates
pub struct SimpleCurrencyConverter {
    fx_rates: RwLock<HashMap<String, f64>>,
}

#[async_trait]
impl CurrencyConverter for SimpleCurrencyConverter {
    async fn fx_rate(
        &self,
        foreign_currency: Currency,
        domestic_currency: Currency,
        _time: DateTime<Local>,
    ) -> Result<f64, CurrencyError> {
        let currency_string = format!(
            "{}/{}",
            &foreign_currency.to_string(),
            &domestic_currency.to_string()
        );
        if let Ok(fx_store) = self.fx_rates.read() {
            if fx_store.contains_key(&currency_string) {
                Ok(fx_store[&currency_string])
            } else {
                Err(CurrencyError::ConversionFailed)
            }
        } else {
            Err(CurrencyError::ConversionFailed)
        }
    }
}

impl SimpleCurrencyConverter {
    /// Create new container
    pub fn new() -> SimpleCurrencyConverter {
        SimpleCurrencyConverter {
            fx_rates: RwLock::new(HashMap::new()),
        }
    }

    /// Insert or update the price of 1 unit of foreign currency in terms of domestic currency and its inverse rate
    pub fn insert_fx_rate(
        &mut self,
        foreign_currency: Currency,
        domestic_currency: Currency,
        fx_rate: f64,
    ) {
        let for_key = foreign_currency.to_string();
        let dom_key = domestic_currency.to_string();
        if let Ok(mut fx_store) = self.fx_rates.write() {
            fx_store.insert(format!("{}/{}", for_key, dom_key), fx_rate);
            fx_store.insert(format!("{}/{}", dom_key, for_key), 1. / fx_rate);
        }
    }
}

impl Default for SimpleCurrencyConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use chrono::offset::TimeZone;
    use chrono::Local;

    use crate::datatypes::CurrencyISOCode;
    use crate::market::Market;
    use crate::postgres::PostgresDB;

    async fn prepare_db(db: Arc<dyn QuoteHandler + Send + Sync>) {
        let time = Local.ymd(1970, 1, 1).and_hms_milli(0, 0, 1, 444);
        let eur = db
            .get_or_new_currency(CurrencyISOCode::new("EUR").unwrap())
            .await
            .unwrap();
        let usd = db
            .get_or_new_currency(CurrencyISOCode::new("USD").unwrap())
            .await
            .unwrap();
        insert_fx_quote(0.9, usd, eur, time, db).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_fx_rate() {
        let db_url = std::env::var("FINQL_TEST_DATABASE_URL");
        assert!(
            db_url.is_ok(),
            "environment variable $FINQL_TEST_DATABASE_URL is not set"
        );
        let db = PostgresDB::new(&db_url.unwrap()).await.unwrap();
        db.clean().await.unwrap();

        let qh: Arc<dyn QuoteHandler + Send + Sync> = Arc::new(db);
        prepare_db(qh.clone()).await;
        let tol = 1.0e-6_f64;
        let market = Market::new(qh);
        let eur = market.get_currency("EUR").await.unwrap();
        let usd = market.get_currency("USD").await.unwrap();
        let time = Local::now();
        let fx = market.fx_rate(usd, eur, time).await.unwrap();
        assert_fuzzy_eq!(fx, 0.9, tol);
    }
}
