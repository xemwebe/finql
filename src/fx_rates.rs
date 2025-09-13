/// Calculation of fx rates based on currency quotes
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use async_trait::async_trait;
use time::OffsetDateTime;

use crate::datatypes::{
    Asset, Currency, CurrencyConverter, CurrencyError, DataError, DataItem, Quote, QuoteHandler,
    Ticker,
};

/// Insert fx rate quote in database including the inverse quote
/// fx_rate is the price of one unit of base currency in terms of the quote currency.
pub async fn insert_fx_quote(
    fx_rate: f64,
    base_currency: Currency,
    quote_currency: Currency,
    time: OffsetDateTime,
    quotes: Arc<dyn QuoteHandler + Send + Sync>,
) -> Result<(), DataError> {
    let base_id = if let Ok(id) = base_currency.get_id() {
        id
    } else {
        quotes.insert_asset(&Asset::Currency(base_currency)).await?
    };
    let currency_pair = format!("{base_currency}/{quote_currency}");
    let ticker_id = quotes
        .insert_ticker(&Ticker {
            id: None,
            name: currency_pair,
            asset: base_id,
            source: "manual".to_string(),
            priority: 10,
            currency: quote_currency,
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
    let quote_id = if let Ok(id) = quote_currency.get_id() {
        id
    } else {
        quotes
            .insert_asset(&Asset::Currency(quote_currency))
            .await?
    };
    let currency_pair = format!("{quote_currency}/{base_currency}");
    let ticker_id = quotes
        .insert_ticker(&Ticker {
            id: None,
            name: currency_pair,
            asset: quote_id,
            source: "manual".to_string(),
            priority: 10,
            currency: base_currency,
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
        base_currency: Currency,
        quote_currency: Currency,
        _time: OffsetDateTime,
    ) -> Result<f64, CurrencyError> {
        let currency_string = format!(
            "{}/{}",
            &base_currency.to_string(),
            &quote_currency.to_string()
        );
        let fx_store = self
            .fx_rates
            .read()
            .map_err(|e| CurrencyError::InternalError(e.to_string()))?;
        if fx_store.contains_key(&currency_string) {
            Ok(fx_store[&currency_string])
        } else {
            Err(CurrencyError::CurrencyNotFound(currency_string.clone()))
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
        base_currency: Currency,
        quote_currency: Currency,
        fx_rate: f64,
    ) {
        let base_key = base_currency.to_string();
        let quote_key = quote_currency.to_string();
        if let Ok(mut fx_store) = self.fx_rates.write() {
            fx_store.insert(format!("{base_key}/{quote_key}"), fx_rate);
            fx_store.insert(format!("{quote_key}/{base_key}"), 1. / fx_rate);
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
    use crate::datatypes::CurrencyISOCode;
    use crate::market::{CachePolicy, Market};
    use crate::postgres::PostgresDB;
    use std::sync::Arc;

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
        let market = Market::new(qh).await;
        let eur = market.get_currency_from_str("EUR").await.unwrap();
        let usd = market.get_currency_from_str("USD").await.unwrap();
        let time = Local::now();
        let fx = market.fx_rate(usd, eur, time).await.unwrap();
        assert_fuzzy_eq!(fx, 0.9, tol);
    }
}
