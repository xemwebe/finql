///! Calculation of fx rates based on currency quotes

use crate::currency::Currency;
use crate::data_handler::{QuoteHandler, DataError};
use chrono::{DateTime,Utc};

/// Calculate foreign exchange rates by reading data from quotes table
pub fn  get_fx_rate(foreign: Currency, base: Currency, time: DateTime<Utc>, quotes: &mut dyn QuoteHandler) -> Result<f64, DataError> {
    if foreign == base {
        return Ok(1.0);
    } else {
        let (fx_quote, quote_currency) = quotes.get_last_quote_before(&format!("{}",foreign), time)?;
        if quote_currency == base {
            return Ok(fx_quote.price);
        }
    }
    Err(DataError::NotFound(format!("{}/{}", foreign, base)))
}
             
#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::Asset;
    use crate::quote::{Quote,Ticker,MarketDataSource};
    use crate::memory_handler::InMemoryDB;
    use crate::data_handler::{AssetHandler, QuoteHandler};
    use chrono::Utc;
    use chrono::offset::TimeZone;
    use std::str::FromStr;

    fn prepare_db() -> InMemoryDB {
        let mut db = InMemoryDB::new();
        let usd_id = db.insert_asset(&Asset{
            id: None,
            name: "USD".to_string(),
            wkn: None,
            isin: None,
            note: None,
        }).unwrap();
        let market_id = db.insert_md_source(
            &MarketDataSource{
                id: None,
                name: "manual".to_string(),
            }
        ).unwrap();
        let eur_usd_id = db.insert_ticker(
            &Ticker{
                id: None,
                name: "USD/EUR".to_string(),
                asset: usd_id,
                source: market_id,
                priority: 10,
                currency: Currency::from_str("EUR").unwrap(),
            }
        ).unwrap();
        let time = Utc.ymd(1970, 1, 1).and_hms_milli(0, 0, 1, 444);
        let _ = db.insert_quote(
            &Quote{
                id: None,
                ticker: eur_usd_id,
                price: 0.9,
                time,
                volume: None,
            }
        );

        db
        }

    

    #[test]
    fn test_get_fx_rate() {
        let mut db = prepare_db();
        let tol = 1.0e-8;
        let eur = Currency::from_str("EUR").unwrap();
        let usd = Currency::from_str("USD").unwrap();
        let time = Utc::now();
        let fx = get_fx_rate(eur, eur, time, &mut db).unwrap();
        assert_fuzzy_eq!(
            fx, 1.0, tol  );
        let fx = get_fx_rate(usd, eur, time, &mut db).unwrap();
        assert_fuzzy_eq!(
            fx, 0.9, tol  );
    
    }

}