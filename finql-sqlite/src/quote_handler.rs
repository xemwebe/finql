///! Implementation for quote handler with Sqlite3 database as backend

use std::str::FromStr;

use chrono::{DateTime, Local};
use async_trait::async_trait;
use std::sync::Arc;

use finql_data::{DataError, QuoteHandler, AssetHandler,
    Currency, Quote, Ticker, date_time_helper::to_time};
use super::SqliteDB;

/// Sqlite implementation of quote handler
#[async_trait]
impl QuoteHandler for SqliteDB {
    fn into_arc_dispatch(self: Arc<Self>) -> Arc<dyn AssetHandler + Send + Sync> {
        self
    }

    // insert, get, update and delete for market data sources
    async fn insert_ticker(&self, ticker: &Ticker) -> Result<usize, DataError> {
        let asset_id = ticker.asset as i64;
        let curr = ticker.currency.to_string();
        sqlx::query!(
            "INSERT INTO ticker (name, asset_id, source, priority, currency, factor) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            ticker.name,
            asset_id,
            ticker.source,
            ticker.priority,
            curr,
            ticker.factor,
        ).execute(&self.pool).await
        .map_err(|e| DataError::InsertFailed(e.to_string()))?;

        let row = sqlx::query!(
            "SELECT id FROM ticker WHERE name=? AND source=?",
                ticker.name, ticker.source,
            ).fetch_one(&self.pool).await
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok(row.id as usize)
    }

    async fn get_ticker_id(&self, ticker: &str) -> Option<usize> {
        let row = sqlx::query!("SELECT id FROM ticker WHERE name=?", ticker)
            .fetch_one(&self.pool).await;
        match row {
            Ok(row) => {
                let id = row.id as usize;
                Some(id)
            }
            _ => None,
        }
    }

    async fn insert_if_new_ticker(&self, ticker: &Ticker) -> Result<usize, DataError> {
         match self.get_ticker_id(&ticker.name).await {
             Some(id) => Ok(id),
             None => self.insert_ticker(ticker).await,
         }
    }

    async fn get_ticker_by_id(&self, id: usize) -> Result<Ticker, DataError> {
        let id_param = id as i32;
        let row = sqlx::query!(
                "SELECT name, asset_id, source, priority, currency, factor FROM ticker WHERE id=?",
                id_param,
            ).fetch_one(&self.pool).await
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let name = row.name;
        let asset = row.asset_id;
        let source = row.source;
        let currency = row.currency;
        let currency =
            Currency::from_str(&currency).map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok(Ticker {
            id: Some(id),
            name,
            asset: asset as usize,
            source,
            priority: row.priority as i32,
            currency,
            factor: row.factor.into(),
        })
    }

    async fn get_all_ticker(&self) -> Result<Vec<Ticker>, DataError> {
        let mut all_ticker = Vec::new();
        for row in sqlx::query!(
                "SELECT id, name, asset_id, priority, source, currency, factor FROM ticker",
            ).fetch_all(&self.pool).await
            .map_err(|e| DataError::NotFound(e.to_string()))?
        {
            let id = row.id;
            let asset = row.asset_id;
            let source = row.source;
            let currency = row.currency;
            let currency =
                Currency::from_str(&currency).map_err(|e| DataError::NotFound(e.to_string()))?;
            let factor = row.factor;
            all_ticker.push(Ticker {
                id: Some(id as usize),
                name: row.name,
                asset: asset as usize,
                source,
                priority: row.priority as i32,
                currency,
                factor: factor as f64,
            });
        }
        Ok(all_ticker)
    }

    async fn get_all_ticker_for_source(
        &self,
        source: &str,
    ) -> Result<Vec<Ticker>, DataError> {
        let mut all_ticker = Vec::new();
        for row in sqlx::query!(
                "SELECT id, name, asset_id, priority, currency, factor FROM ticker WHERE source=?",
                source,
            ).fetch_all(&self.pool).await
            .map_err(|e| DataError::NotFound(e.to_string()))?
        {
            let id = row.id;
            let asset = row.asset_id;
            let currency = row.currency;
            let currency =
                Currency::from_str(&currency).map_err(|e| DataError::NotFound(e.to_string()))?;
            let factor = row.factor;
            all_ticker.push(Ticker {
                id: Some(id as usize),
                name: row.name,
                asset: asset as usize,
                source: source.to_string(),
                priority: row.priority as i32,
                currency,
                factor: factor.into(),
            });
        }
        Ok(all_ticker)
    }

    async fn get_all_ticker_for_asset(
        &self,
        asset_id: usize,
    ) -> Result<Vec<Ticker>, DataError> {
        let mut all_ticker = Vec::new();
        let a_id = asset_id as i64;
        for row in sqlx::query!(
                "SELECT id, name, source, priority, currency, factor FROM ticker WHERE asset_id=?",
                a_id,
            ).fetch_all(&self.pool).await
            .map_err(|e| DataError::NotFound(e.to_string()))?
        {
            let id = row.id;
            let source = row.source;
            let currency = row.currency;
            let currency =
                Currency::from_str(&currency).map_err(|e| DataError::NotFound(e.to_string()))?;
            all_ticker.push(Ticker {
                id: Some(id as usize),
                name: row.name,
                asset: asset_id,
                source,
                priority: row.priority as i32,
                currency,
                factor: row.factor.into(),
            });
        }
        Ok(all_ticker)
    }

    async fn update_ticker(&self, ticker: &Ticker) -> Result<(), DataError> {
        if ticker.id.is_none() {
            return Err(DataError::NotFound(
                "not yet stored to database".to_string(),
            ));
        }
        let id = ticker.id.unwrap() as i32;
        let curr = ticker.currency.to_string();
        let asset_id = ticker.asset as i32;
        sqlx::query!(
                "UPDATE ticker SET name=?2, asset_id=?3, source=?4, priority=?5, currency=?6, factor=?7
                WHERE id=?1",
                id,
                ticker.name,
                asset_id,
                ticker.source,
                ticker.priority,
                curr,
                ticker.factor,
            )
            .execute(&self.pool).await
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    async fn delete_ticker(&self, id: usize) -> Result<(), DataError> {
        let id = id as i64;
        sqlx::query!("DELETE FROM ticker WHERE id=?", id)
            .execute(&self.pool).await
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    // insert, get, update and delete for market data sources
    async fn insert_quote(&self, quote: &Quote) -> Result<usize, DataError> {
        let ticker_id = quote.ticker as i64;
        sqlx::query!(
                "INSERT INTO quotes (ticker_id, price, time, volume) 
                VALUES (?, ?, ?, ?)",
                ticker_id,
                quote.price,
                quote.time,
                quote.volume,
            ).execute(&self.pool).await
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        let row = sqlx::query!(
                "SELECT id FROM quotes WHERE ticker_id=? and time=?",
                ticker_id, quote.time,
            ).fetch_one(&self.pool).await
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok(row.id as usize)
    }

    async fn get_last_quote_before(
        &self,
        asset_name: &str,
        time: DateTime<Local>,
    ) -> Result<(Quote, Currency), DataError> {
        let row = sqlx::query!(
                "SELECT q.id, q.ticker_id, q.price, q.time, q.volume, t.currency, t.priority
                FROM quotes q, ticker t, assets a 
                WHERE a.name=? AND t.asset_id=a.id AND t.id=q.ticker_id AND q.time<=?
                ORDER BY q.time DESC, t.priority ASC LIMIT 1",
                asset_name, time,
            ).fetch_one(&self.pool).await
            .map_err(|e| DataError::NotFound(e.to_string()))?;

        let id = row.id;
        let ticker = row.ticker_id;
        let price = row.price;
        let time = row.time;
        let volume = row.volume;
        let currency = row.currency;
        let currency =
            Currency::from_str(&currency).map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok((
            Quote {
                id: Some(id as usize),
                ticker: ticker as usize,
                price: price.into(),
                time: to_time(&time, 0).map_err(|e| DataError::NotFound(e.to_string()))?,
                volume: volume.map(|x| x as f64),
            },
            currency,
        ))
    }

    async fn get_last_quote_before_by_id(
        &self,
        asset_id: usize,
        time: DateTime<Local>,
    ) -> Result<(Quote, Currency), DataError> {
        let asset_id = asset_id as i32;
        let row = sqlx::query!(
                "SELECT q.id, q.ticker_id, q.price, q.time, q.volume, t.currency, t.priority
                FROM quotes q, ticker t
                WHERE t.asset_id=?1 AND t.id=q.ticker_id AND q.time<= ?2
                ORDER BY q.time DESC, t.priority ASC LIMIT 1",
                asset_id, time,
            ).fetch_one(&self.pool).await
            .map_err(|e| DataError::NotFound(e.to_string()))?;

        let id = row.id;
        let ticker = row.ticker_id;
        let price = row.price;
        let time = row.time;
        let volume = row.volume;
        let currency = row.currency;
        let currency =
            Currency::from_str(&currency).map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok((
            Quote {
                id: Some(id as usize),
                ticker: ticker as usize,
                price: price.into(),
                time: to_time(&time, 0).map_err(|e| DataError::NotFound(e.to_string()))?,
                volume: volume.map(|x| x as f64),
            },
            currency,
        ))
    }

    async fn get_all_quotes_for_ticker(&self, ticker_id: usize) -> Result<Vec<Quote>, DataError> {
        let mut quotes = Vec::new();
        let t_id = ticker_id as i32;
        for row in sqlx::query!(
                "SELECT id, price, time, volume FROM quotes 
                WHERE ticker_id=?1 ORDER BY time ASC",
                t_id,
            ).fetch_all(&self.pool).await
            .map_err(|e| DataError::NotFound(e.to_string()))?
        {
            let id = row.id;
            let time = row.time;
            quotes.push(Quote {
                id: Some(id as usize),
                ticker: ticker_id,
                price: row.price.into(),
                time: to_time(&time, 0).map_err(|e| DataError::NotFound(e.to_string()))?,
                volume: row.volume.map(|x| x as f64),
            });
        }
        Ok(quotes)
    }

    async fn update_quote(&self, quote: &Quote) -> Result<(), DataError> {
        if quote.id.is_none() {
            return Err(DataError::NotFound(
                "not yet stored to database".to_string(),
            ));
        }
        let id = quote.id.unwrap() as i32;
        let ticker_id = quote.ticker as i32;
        sqlx::query!(
                "UPDATE quotes SET ticker_id=?2, price=?3, time=?4, volume=?5
                WHERE id=?1",
                id,
                ticker_id,
                quote.price,
                quote.time,
                quote.volume,
            )
            .execute(&self.pool).await
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    async fn delete_quote(&self, id: usize) -> Result<(), DataError> {
        let id = id as i64;
        sqlx::query!("DELETE FROM quotes WHERE id=?1;", id)
            .execute(&self.pool).await
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }


    async fn remove_duplicates(&self) -> Result<(), DataError> {
        sqlx::query!("
            delete from quotes 
            where id in
            (select q2.id
            from 
                quotes q1,
                quotes q2
            where 
                q1.id < q2.id
            and q1.ticker_id = q2.ticker_id 
            and q1.time = q2.time
            and q1.price = q2.price) 
            ")
            .execute(&self.pool).await
            .map_err(|e| DataError::DeleteFailed(e.to_string()))?;
        Ok(())
    }

    async fn get_rounding_digits(&self, currency: Currency) -> i32 {
        let curr = currency.to_string();
        let rows = sqlx::query!(
            "SELECT digits FROM rounding_digits WHERE currency=?1;",
            curr,
        ).fetch_all(&self.pool).await;
        match rows {
            Ok(row_vec) => {
                if !row_vec.is_empty() {
                    row_vec[0].digits as i32
                } else {
                    2
                }
            }
            Err(_) => 2,
        }
    }

    async fn set_rounding_digits(&self, currency: Currency, digits: i32) -> Result<(), DataError> {
        let curr = currency.to_string();
        let _row = sqlx::query!(
                "INSERT INTO rounding_digits (currency, digits) VALUES (?1, ?2)",
                curr, digits,
            ).execute(&self.pool).await
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }
}
