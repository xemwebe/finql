///! Calculation of fx rates based on currency quotes

use crate::currency::Currency;
use crate::data_handler::{QuoteHandler, DataError};
use chrono::{DateTime,Utc};
use crate::quote::Quote;

/// Calculate foreign exchange rates by reading data from quotes table
/// If no direct quote is available, try to calculate cross rates using a simple heuristic.
pub fn  get_fx_rate(foreign: Currency, base: Currency, time: DateTime<Utc>, quotes: &mut dyn QuoteHandler) -> Result<(Quote, Currency), DataError> {
    if foreign == base {
        let dummy_quote = Quote{
            id: Some(0),
            ticker: 0,
            price: 1.0,
            time: Utc::now(), 
            volume: None,
        };
        Ok((dummy_quote, base))
    } else {
        let fx_quote = quotes.get_last_quote_before(&format!("{}",foreign), time)?;
        Ok(fx_quote)
        }
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
        let fx = get_fx_rate(eur, eur, time, &mut db).unwrap().0.price;
        assert_fuzzy_eq!(
            fx, 1.0, tol  );
        let fx = get_fx_rate(usd, eur, time, &mut db).unwrap().0.price;
        assert_fuzzy_eq!(
            fx, 0.9, tol  );
    
    }

}