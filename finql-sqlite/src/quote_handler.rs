///! Implementation for quote handler with Sqlite3 database as backend

use std::str::FromStr;

use chrono::{DateTime, Local};
use async_trait::async_trait;
use std::sync::Arc;

use finql_data::{DataError, QuoteHandler, AssetHandler, Currency, Quote, Ticker, CurrencyISOCode};

use super::{SqliteDB, SQLiteError};
use deadpool_sqlite::rusqlite::params;
use deadpool_sqlite::rusqlite;
use futures::TryFutureExt;


/// Sqlite implementation of quote handler
#[async_trait]
impl QuoteHandler for SqliteDB {
    fn into_arc_dispatch(self: Arc<Self>) -> Arc<dyn AssetHandler + Send + Sync> {
        self
    }

    // insert, get, update and delete for market data sources
    async fn insert_ticker(&self, ticker: &Ticker) -> Result<usize, DataError> {
        let ticker = ticker.to_owned();
        let ticker_name = ticker.name.clone();
        let ticker_source = ticker.source.clone();
        let curr = ticker.currency;

        let _ = self.conn.interact(move |conn| -> Result<(), SQLiteError> {
            conn.execute(
                "INSERT INTO ticker (name, asset_id, source, priority, currency_id, factor, tz, cal)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![            
                    &ticker.name,
                    &ticker.asset,
                    &ticker.source,
                    &ticker.priority,
                    &curr.id.unwrap(),
                    &ticker.factor,
                    &ticker.tz,
                    &ticker.cal,
                ])?;
            Ok(())
        }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))?;

        self.conn.interact(move |conn| -> Result<usize, SQLiteError> {
            Ok(conn.query_row(
                "SELECT id FROM ticker WHERE name=? AND source=?",
                params![&ticker_name, &ticker_source],
                |row| row.get(0) )?)
        }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))
    }

    async fn get_ticker_id(&self, ticker: &str) -> Option<usize> {
        let ticker = ticker.to_owned();
        self.conn.interact(move |conn| -> Option<usize> {
            conn.query_row(
                "SELECT id FROM ticker WHERE name=?",
                params![&ticker],
                |row| row.get(0) ).ok()
        }).await.ok().flatten()
    }

    async fn insert_if_new_ticker(&self, ticker: &Ticker) -> Result<usize, DataError> {
         match self.get_ticker_id(&ticker.name).await {
             Some(id) => Ok(id),
             None => self.insert_ticker(ticker).await,
         }
    }

    async fn get_ticker_by_id(&self, id: usize) -> Result<Ticker, DataError> {
        self.conn.interact(move |conn| -> Result<Ticker, SQLiteError> {
            Ok(conn.query_row(
                "SELECT t.name, t.asset_id, t.source, t.priority, t.factor, t.tz, t.cal,
                     c.id AS currency_id, c.iso_code AS currency_iso_code, c.rounding_digits AS currency_rounding_digits
                 FROM ticker t
                 JOIN currencies c ON c.id = t.currency_id
                 WHERE t.id = ?",
                params![&id],
                |row| {
                    let cio:String = row.get(8)?;

                    Ok(Ticker {
                    id: Some(id),
                    name: row.get(0)?,
                    asset: row.get(1)?,
                    source: row.get(2)?,
                    priority: row.get(3)?,
                    currency: Currency{
                        id: row.get(7)?,
                        iso_code: CurrencyISOCode::from_str(&cio).expect("unknown currency asset referenced in db"),
                        rounding_digits: row.get(9)?,
                    },
                    factor: row.get(4)?,
                    tz: row.get(5)?,
                    cal: row.get(6)?
                })
            })?)
        }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))
    }

    async fn get_all_ticker(&self) -> Result<Vec<Ticker>, DataError> {
        self.conn.interact(|conn| -> Result<Vec<Ticker>, SQLiteError> {
            let mut stmt = conn.prepare(
                "SELECT t.id, t.name, t.asset_id, t.priority, t.source, t.factor, t.tz, t.cal,
                     c.id AS currency_id, c.iso_code AS currency_iso_code, c.rounding_digits AS currency_rounding_digits
                 FROM ticker t
                 JOIN currencies c ON c.id = t.currency_id"
            )?;

            let ticker: Vec<Ticker> = stmt.query_map([], |row| {
                let cio:String = row.get(9)?;

                Ok(Ticker {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    asset: row.get(2)?,
                    priority: row.get(3)?,
                    source: row.get(4)?,
                    currency: Currency{
                        id: row.get(8)?,
                        iso_code: CurrencyISOCode::from_str(&cio).expect("unknown currency asset referenced in db"),
                        rounding_digits: row.get(10)?,
                    },
                    factor: row.get(5)?,
                    tz: row.get(6)?,
                    cal: row.get(7)?
                })
            })?.filter_map(|e| e.ok()).collect();
            Ok(ticker)
        })
        .await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))
    }

    async fn get_all_ticker_for_source(
        &self,
        source: &str,
    ) -> Result<Vec<Ticker>, DataError> {
        let source = source.to_owned();
        self.conn.interact(move |conn| -> Result<Vec<Ticker>, SQLiteError> {
            let mut stmt = conn.prepare(
                "SELECT t.id, t.name, t.asset_id, t.priority, t.source, t.factor, t.tz, t.cal,
                     c.id AS currency_id, c.iso_code AS currency_iso_code, c.rounding_digits AS currency_rounding_digits
                 FROM ticker t
                 JOIN currencies c ON c.id = t.currency_id
                 WHERE t.source = ?"
            )?;

            let ticker: Vec<Ticker> = stmt.query_map(params![source], |row| {
                let cio:String = row.get(9)?;

                Ok(Ticker {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    asset: row.get(2)?,
                    priority: row.get(3)?,
                    source: row.get(4)?,
                    currency: Currency{
                        id: row.get(8)?,
                        iso_code: CurrencyISOCode::from_str(&cio).expect("unknown currency asset referenced in db"),
                        rounding_digits: row.get(10)?,
                    },
                    factor: row.get(5)?,
                    tz: row.get(6)?,
                    cal: row.get(7)?
                })
            })?.filter_map(|e| e.ok() ).collect();
            Ok(ticker)
        })
        .await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))
    }

    async fn get_all_ticker_for_asset(
        &self,
        asset_id: usize,
    ) -> Result<Vec<Ticker>, DataError> {
        self.conn.interact(move |conn| -> Result<Vec<Ticker>, SQLiteError> {
            let mut stmt = conn.prepare(
                "SELECT t.id, t.name, t.asset_id, t.priority, t.source, t.factor, t.tz, t.cal,
                     c.id AS currency_id, c.iso_code AS currency_iso_code, c.rounding_digits AS currency_rounding_digits
                 FROM ticker t
                 JOIN currencies c ON c.id = t.currency_id
                 WHERE t.asset_id = ?"
            )?;

            let ticker: Vec<Ticker> = stmt.query_map(params![&asset_id], |row| {
                let cio:String = row.get(9)?;

                Ok(Ticker {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    asset: row.get(2)?,
                    priority: row.get(3)?,
                    source: row.get(4)?,
                    currency: Currency{
                        id: row.get(8)?,
                        iso_code: CurrencyISOCode::from_str(&cio).expect("unknown currency asset referenced in db"),
                        rounding_digits: row.get(10)?,
                    },
                    factor: row.get(5)?,
                    tz: row.get(6)?,
                    cal: row.get(7)?
                })
            })?.filter_map(|e| e.ok() ).collect();
            Ok(ticker)
        })
        .await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))
    }

    async fn update_ticker(&self, ticker: &Ticker) -> Result<(), DataError> {
        if ticker.id.is_none() {
            return Err(DataError::NotFound(
                "not yet stored to database".to_string(),
            ));
        }
        let ticker = ticker.to_owned();
        let cid = ticker.currency.id.expect("currency asset_id required");
        self.conn.interact(move |conn| -> Result<(), SQLiteError> {
            conn.execute(
                "UPDATE ticker SET name = ?2, asset_id = ?3, source = ?4, priority = ?5,
                currency_id = ?6, factor = ?7, tz = ?8, cal = ?9
                WHERE id = ?1",
                params![
                    &ticker.id,                
                    &ticker.name,
                    &ticker.asset,
                    &ticker.source,
                    &ticker.priority,
                    &cid,
                    &ticker.factor,
                    &ticker.tz,
                    &ticker.cal
                ]
            )?;
            Ok(())
        }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))
    }

    async fn delete_ticker(&self, id: usize) -> Result<(), DataError> {
        self.conn.interact(move |conn| -> Result<(), SQLiteError> {
            conn.execute("DELETE FROM ticker WHERE id=?", params![&id])?;
            Ok(())
        }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))
    }

    // insert, get, update and delete for market data sources
    async fn insert_quote(&self, quote: &Quote) -> Result<usize, DataError> {
        let quote = quote.to_owned();
        let quote_ticker = quote.ticker.clone();
        let quote_time = quote.time.clone();
        let _ = self.conn.interact(move |conn| -> Result<(), SQLiteError> {
            conn.execute(
                "INSERT INTO quotes (ticker_id, price, time, volume) \
                VALUES (?, ?, ?, ?)",
                params![&quote.ticker, quote.price, quote.time, quote.volume])?;
            Ok(())
        }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()));

        self.conn.interact(move |conn| -> Result<usize, SQLiteError> {
            Ok(conn.query_row(
                "SELECT id FROM quotes WHERE ticker_id=? and time=?",
                params![&quote_ticker, &quote_time],
                |row| row.get(0) )?)
        }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))
    }

    async fn get_last_quote_before(
        &self,
        asset_name: &str,
        time: DateTime<Local>,
    ) -> Result<(Quote, Currency), DataError> {
        let asset = asset_name.to_owned();
        let r = self.conn.interact(move |conn| -> Result<(Quote, usize), SQLiteError> {
            Ok(conn.query_row(
                "SELECT q.id, q.ticker_id, q.price, q.time, q.volume, t.currency_id, t.priority
                FROM quotes q
                JOIN ticker t ON t.id = q.ticker_id
                JOIN assets a ON  a.id = t.asset_id
                WHERE
                    a.name = ?
                    AND q.time <= ?
                ORDER BY q.time DESC, t.priority ASC
                LIMIT 1",
                params![&asset, &time],
                |row| {
                    Ok((Quote {
                        id: row.get(0)?,
                        ticker: row.get(1)?,
                        price: row.get(2)?,
                        time: row.get(3)?,
                        volume: row.get(4)?,
                    }, row.get(5)?))
                }
            )?)
        }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))?;

        let (q, cid) = r.map_err(|e| DataError::DataAccessFailure(e.to_string()))?;

        let ca = self.get_asset_by_id(cid).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))
    .map_err(|e| DataError::DataAccessFailure(e.to_string()))?;
        let c = ca.currency().expect("asset should refer to a currency");
        Ok((q, c))
}

/// Get the latest quote before a given time for a specific asset id
    async fn get_last_quote_before_by_id(
        &self,
        asset_id: usize,
        time: DateTime<Local>,
    ) -> Result<(Quote, Currency), DataError> {
        let (q, cid) = self.conn.interact(move |conn| -> Result<(Quote, usize), SQLiteError> {
            Ok(conn.query_row(
                "SELECT q.id, q.ticker_id, q.price, q.time, q.volume, t.currency_id, t.priority
                FROM quotes q
                JOIN ticker t ON t.id = q.ticker_id
                WHERE t.asset_id = ?1 AND q.time <= ?2
                ORDER BY q.time DESC, t.priority ASC
                LIMIT 1",
                params![&asset_id, &time],
                |row| { 
                    Ok((Quote {
                    id: row.get(0)?,
                    ticker: row.get(1)?,
                    price: row.get(2)?,
                    time: row.get(3)?,
                    volume: row.get(4)?,
                }, row.get(5)?))
            })?)
        }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))?;

        let ca = self.get_asset_by_id(cid).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))
            .map_err(|e| DataError::DataAccessFailure(e.to_string()))?;
        let c = ca.currency().expect("asset should refer to a currency");
        Ok((q, c))
    }

    async fn get_all_quotes_for_ticker(&self, ticker_id: usize) -> Result<Vec<Quote>, DataError> {
        self.conn.interact(move |conn| -> Result<Vec<Quote>, SQLiteError> {
            let mut stmt = conn.prepare("SELECT id, price, time, volume FROM quotes \
            WHERE ticker_id=?1 ORDER BY time ASC")?;
            let quotes: Vec<Quote> = stmt.query_map([&ticker_id], |row| {
                Ok(Quote {
                    id: row.get(0)?,
                    ticker: ticker_id,
                    price: row.get(1)?,
                    time: row.get(2)?,
                    volume: row.get(3)?,
                })
            })?.filter_map(|quote| quote.ok() ).collect();
            Ok(quotes)
        })
        .await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))
    }

    async fn update_quote(&self, quote: &Quote) -> Result<(), DataError> {
        if let Some(id) = quote.id {
            let quote = quote.to_owned();
            self.conn.interact(move |conn| -> Result<(), SQLiteError> {
                conn.execute(
                    "UPDATE quotes SET ticker_id=?2, price=?3, time=?4, volume=?5 \
                    WHERE id=?1",
                    params![&id, &quote.ticker, &quote.price, &quote.time, &quote.volume])?;
                Ok(())
            }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
            .map_err(|e| DataError::DataAccessFailure(e.to_string()))
        } else {
            Err(DataError::NotFound(
                "not yet stored to database".to_string(),
            ))
        }
    }

    async fn delete_quote(&self, id: usize) -> Result<(), DataError> {
        self.conn.interact(move |conn| -> Result<(), SQLiteError> {
            conn.execute("DELETE FROM quotes WHERE id=?1", params![&id])?;
            Ok(())
        }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))
    }

    async fn remove_duplicates(&self) -> Result<(), DataError> {
        self.conn.interact(|conn| -> Result<(), SQLiteError> {
            let _ = conn.execute("DELETE FROM quotes \
            WHERE id IN \
            (SELECT q2.id \
            FROM \
                quotes q1, \
                quotes q2 \
            WHERE \
                q1.id < q2.id \
            AND q1.ticker_id = q2.ticker_id \
            AND q1.time = q2.time \
            AND q1.price = q2.price)", []);
            Ok(())
        }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::Arc;
    use super::super::SqliteDBPool;
    use finql_data::{Asset, CurrencyISOCode, DataItem, date_time_helper::make_time};

    #[tokio::test]
    async fn sqlite_ticker_test() {
        let sqlite_pool = Arc::new(SqliteDBPool::in_memory().await.unwrap());
        let db = sqlite_pool.get_conection().await.unwrap();
        assert!(db.clean().await.is_ok());

        let mut asset = Asset::new_stock(
            None,
            "asset A".to_string(),
            Some("Just a simple asset for testing".to_string()),
            "123456789012".to_string(),
            Some("A1B2C3".to_string()),
        );

        let id = db.insert_asset(&mut asset).await.unwrap();
        asset.set_id(id);
        assert_eq!(asset.id, Some(1));

        let mut eur_asset = Asset::new_currency(None, "euro".to_string(), None, CurrencyISOCode::from_str("EUR").unwrap(), 0);
        let eur_id = db.insert_asset(&eur_asset).await.unwrap();
        eur_asset.set_id(eur_id);
        let eur = eur_asset.currency().unwrap();

        let s1_ticker = Ticker{
            id: None,
            asset: asset.id.unwrap(),
            name: "A".to_string(),
            currency: eur,
            source: "s1".to_string(),
            priority: 2,
            factor: 3.0,
            tz: None,
            cal: None,
        };
        let s1_id = db.insert_ticker(&s1_ticker).await.unwrap();
        assert_eq!(s1_id, 1);
        assert_eq!(db.insert_if_new_ticker(&s1_ticker).await.unwrap(), 1);

        let mut s2_ticker = Ticker{
            id: None,
            asset: asset.id.unwrap(),
            name: "B".to_string(),
            currency: eur,
            source: "s2".to_string(),
            priority: 4,
            factor: 5.0,
            tz: Some("Europe".to_string()),
            cal: Some("TARGET".to_string()),
        };
        let s2_id = db.insert_if_new_ticker(&s2_ticker).await.unwrap();
        assert_eq!(s2_id, 2);

        let s2_id2 = db.get_ticker_id("B").await.unwrap();
        assert_eq!(s2_id2, s2_id);
        
        s2_ticker.id = Some(s2_id);

        let new_s2 = db.get_ticker_by_id(2).await.unwrap();
        assert_eq!(new_s2.name, s2_ticker.name);

        let all_tickers = db.get_all_ticker().await.unwrap();
        assert_eq!(all_tickers.len(), 2);

        let all_tickers = db.get_all_ticker_for_source("s2").await.unwrap();
        assert_eq!(all_tickers.len(), 1);

        let all_tickers = db.get_all_ticker_for_asset(1).await.unwrap();
        assert_eq!(all_tickers.len(), 2);

        s2_ticker.tz = Some("UK".to_string());
        assert!(db.update_ticker(&s2_ticker).await.is_ok());
        assert!(db.delete_ticker(s2_id).await.is_ok());   
    }

    #[tokio::test]
    async fn sqlite_quotes_test() {
        let sqlite_pool = Arc::new(SqliteDBPool::in_memory().await.unwrap());
        let db = sqlite_pool.get_conection().await.unwrap();
        assert!(db.clean().await.is_ok());

        let mut asset = Asset::new_stock(
            None,
            "asset A".to_string(),
            Some("Just a simple asset for testing".to_string()),
            "123456789012".to_string(),
            Some("A1B2C3".to_string()),
        );


        let id = db.insert_asset(&mut asset).await.unwrap();
        asset.set_id(id);
        assert_eq!(asset.id, Some(1));

        let mut eur_asset = Asset::new_currency(None, "euro".to_string(), None, CurrencyISOCode::from_str("EUR").unwrap(), 0);
        let eur_id = db.insert_asset(&eur_asset).await.unwrap();
        eur_asset.set_id(eur_id);
        let eur = eur_asset.currency().unwrap();

        let ticker = Ticker{
            id: None,
            asset: asset.id.unwrap(),
            name: "A".to_string(),
            currency: eur,
            source: "s1".to_string(),
            priority: 2,
            factor: 3.0,
            tz: None,
            cal: None,
        };
        let ticker = db.insert_ticker(&ticker).await.unwrap();
        assert_eq!(ticker, 1);

        let time_now = Local::now();
        let quote1 = Quote{
            id: None,
            ticker,
            price: 2.0,
            time: time_now,
            volume: None,
        };
        let qid1 = db.insert_quote(&quote1).await.unwrap();
        assert_eq!(qid1, 1);

        let time2 = make_time(2021,12,6,19,0,0).unwrap();
        let mut quote2 = Quote{
            id: None,
            ticker,
            price: 1.5,
            time: time2,
            volume: None,
        };
        let qid2 = db.insert_quote(&quote2).await.unwrap();
        assert_eq!(qid2, 2);

        let time3 = make_time(2021,12,6,19,1,0).unwrap();
        let quote_before = db.get_last_quote_before("asset A", time3).await.unwrap();
        assert_eq!(quote_before.0.price, 1.5);
        assert_eq!(quote_before.1, eur);

        let quote_before = db.get_last_quote_before_by_id(asset.id.unwrap(), time3).await.unwrap();
        assert_eq!(quote_before.0.price, 1.5);
        
        quote2.id = Some(qid2);
        quote2.price = 1.7;
        assert!(db.update_quote(&quote2).await.is_ok());
        assert!(db.delete_quote(qid2).await.is_ok());

        let _ = db.insert_quote(&quote1).await.unwrap();
        let quotes = db.get_all_quotes_for_ticker(ticker).await.unwrap();
        assert_eq!(quotes.len(), 2);
        assert!(db.remove_duplicates().await.is_ok());
        let quotes = db.get_all_quotes_for_ticker(ticker).await.unwrap();
        assert_eq!(quotes.len(), 1);
    }
}