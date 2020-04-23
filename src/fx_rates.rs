use crate::asset::Asset;
///! Calculation of fx rates based on currency quotes
use crate::currency::Currency;
use crate::data_handler::{DataError, QuoteHandler};
use crate::quote::{MarketDataSource, Quote, Ticker};
use chrono::{DateTime, Utc};

/// Calculate foreign exchange rates by reading data from quotes table
pub fn get_fx_rate(
    foreign: Currency,
    base: Currency,
    time: DateTime<Utc>,
    quotes: &mut dyn QuoteHandler,
) -> Result<f64, DataError> {
    if foreign == base {
        return Ok(1.0);
    } else {
        let (fx_quote, quote_currency) =
            quotes.get_last_quote_before(&format!("{}", foreign), time)?;
        if quote_currency == base {
            return Ok(fx_quote.price);
        }
    }
    Err(DataError::NotFound(format!("{}/{}", foreign, base)))
}

/// Insert fx rate quote in database including the inverse quote
pub fn insert_fx_quote(
    fx_rate: f64,
    foreign: Currency,
    base: Currency,
    time: DateTime<Utc>,
    quotes: &mut dyn QuoteHandler,
) -> Result<(), DataError> {
    let foreign_id = quotes
        .insert_asset(&Asset {
            id: None,
            name: foreign.to_string(),
            wkn: None,
            isin: None,
            note: None,
        })
        .unwrap();
    let market_data_source = MarketDataSource::Manual;
    let currency_pair = format!("{}/{}", foreign, base);
    let ticker_id = quotes
        .insert_ticker(&Ticker {
            id: None,
            name: currency_pair,
            asset: foreign_id,
            source: market_data_source,
            priority: 10,
            currency: base,
            factor: 1.0,
        })
        .unwrap();
    let _ = quotes.insert_quote(&Quote {
        id: None,
        ticker: ticker_id,
        price: fx_rate,
        time,
        volume: None,
    });
    // Insert inverse fx quote
    let base_id = quotes
        .insert_asset(&Asset {
            id: None,
            name: base.to_string(),
            wkn: None,
            isin: None,
            note: None,
        })
        .unwrap();
    let currency_pair = format!("{}/{}", base, foreign);
    let ticker_id = quotes
        .insert_ticker(&Ticker {
            id: None,
            name: currency_pair,
            asset: base_id,
            source: market_data_source,
            priority: 10,
            currency: foreign,
            factor: 1.0,
        })
        .unwrap();
    let _ = quotes.insert_quote(&Quote {
        id: None,
        ticker: ticker_id,
        price: 1.0 / fx_rate,
        time,
        volume: None,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_handler::QuoteHandler;
    use crate::sqlite_handler::SqliteDB;
    use chrono::offset::TimeZone;
    use chrono::Utc;
    use std::str::FromStr;

    fn prepare_db(db: &mut dyn QuoteHandler) {
        let time = Utc.ymd(1970, 1, 1).and_hms_milli(0, 0, 1, 444);
        let eur = Currency::from_str("EUR").unwrap();
        let usd = Currency::from_str("USD").unwrap();
        let _ = insert_fx_quote(0.9, usd, eur, time, db);
    }

    #[test]
    fn test_get_fx_rate() {
        let mut db = SqliteDB::create(":memory:").unwrap();
        prepare_db(&mut db);
        let tol = 1.0e-8;
        let eur = Currency::from_str("EUR").unwrap();
        let usd = Currency::from_str("USD").unwrap();
        let time = Utc::now();
        let fx = get_fx_rate(eur, eur, time, &mut db).unwrap();
        assert_fuzzy_eq!(fx, 1.0, tol);
        let fx = get_fx_rate(usd, eur, time, &mut db).unwrap();
        assert_fuzzy_eq!(fx, 0.9, tol);
    }
}
