///! Implementation for quote handler with Sqlite3 database as backend
use super::SqliteDB;
use super::helpers::to_time;
use crate::data_handler::{QuoteHandler, DataError};
use crate::quote::{MarketDataSource, Ticker, Quote};
use chrono::{DateTime, Utc};
use rusqlite::{params, NO_PARAMS};
use crate::currency::Currency;
use std::str::FromStr;

/// Sqlite implementation of quote handler
impl QuoteHandler for SqliteDB {
    // insert, get, update and delete for market data sources
    fn insert_md_source(&self, source: &MarketDataSource) -> Result<usize, DataError> {
        self.conn
            .execute(
                "INSERT INTO market_data_sources (name) VALUES (?1)",
                params![source.name],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        let id = self
            .conn
            .query_row(
                "SELECT id FROM market_data_sources
        WHERE name=?;",
                params![source.name],
                |row| {
                    let id: i64 = row.get(0)?;
                    Ok(id as usize)
                },
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok(id)
    }
    fn get_md_source_by_id(&self, id: usize) -> Result<MarketDataSource, DataError> {
        let ticker = self
            .conn
            .query_row(
                "SELECT name FROM market_data_sources WHERE id=?;",
                params![id as i64],
                |row| {
                    Ok(MarketDataSource {
                        id: Some(id),
                        name: row.get(0)?,
                    })
                },
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok(ticker)
    }
    fn get_all_md_sources(&self) -> Result<Vec<MarketDataSource>, DataError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name FROM market_data_sources;")
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let sources_map = stmt
            .query_map(NO_PARAMS, |row| {
                let id: i64 = row.get(0)?;
                let id = Some(id as usize);
                Ok(MarketDataSource {
                    id,
                    name: row.get(1)?,
                })
            })
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let mut sources = Vec::new();
        for source in sources_map {
            sources.push(source.unwrap());
        }
        Ok(sources)
    }
    fn update_md_source(&self, source: &MarketDataSource) -> Result<(), DataError> {
        if source.id.is_none() {
            return Err(DataError::NotFound(
                "not yet stored to database".to_string(),
            ));
        }
        let id = source.id.unwrap() as i64;
        self.conn
            .execute(
                "UPDATE market_data_sources SET name=?2 
                WHERE id=?1;",
                params![id, source.name],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }
    fn delete_md_source(&self, id: usize) -> Result<(), DataError> {
        self.conn
            .execute("DELETE FROM market_data_sources WHERE id=?1;", params![id as i64])
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    // insert, get, update and delete for market data sources
    fn insert_ticker(&self, ticker: &Ticker) -> Result<usize, DataError> {
        self.conn
            .execute(
                "INSERT INTO ticker (name, source_id, currency) VALUES (?, ?, ?)",
                params![ticker.name, ticker.source as i64,
                    ticker.currency.to_string()],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        let id = self
            .conn
            .query_row(
                "SELECT id FROM ticker
        WHERE name=? AND source_id=?;",
                params![ticker.name, ticker.source as i64],
                |row| {
                    let id: i64 = row.get(0)?;
                    Ok(id as usize)
                },
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok(id)
    }
    fn get_ticker_by_id(&self, id: usize) -> Result<Ticker, DataError> {
        let ticker = self
            .conn
            .query_row(
                "SELECT name, source_id, currency FROM ticker WHERE id=?;",
                params![id as i64],
                |row| {
                    let name: String = row.get(0)?;
                    let source: i64 = row.get(1)?;
                    let currency: String = row.get(2)?;
                    let raw_ticker = (name, source, currency);
                    Ok(raw_ticker)
                },
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let currency = Currency::from_str(&ticker.2).map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok(Ticker{id: Some(id), name: ticker.0, source: ticker.1 as usize, currency})
    }
    fn get_all_ticker_for_source(&self, source_id: usize) -> Result<Vec<Ticker>, DataError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, currency FROM ticker WHERE source_id=?;")
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let ticker_map = stmt
            .query_map(params![source_id as i64], |row| {
                let id: i64 = row.get(0)?;
                let name: String = row.get(1)?;
                let currency: String = row.get(2)?;
                let raw_ticker = (id, name, currency);
                Ok(raw_ticker)
            })
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let mut all_ticker = Vec::new();
        for ticker in ticker_map {
            let ticker = ticker.unwrap();
            let currency = Currency::from_str(&ticker.2).map_err(|e| DataError::NotFound(e.to_string()))?;
            all_ticker.push(Ticker{ id: Some(ticker.0 as usize), name: ticker.1, source: source_id, currency});
        }
        Ok(all_ticker)
    }
    fn update_ticker(&self, ticker: &Ticker) -> Result<(), DataError> {
        if ticker.id.is_none() {
            return Err(DataError::NotFound(
                "not yet stored to database".to_string(),
            ));
        }
        let id = ticker.id.unwrap() as i64;
        self.conn
        .execute(
            "UPDATE ticker SET name=?2, source_id=?3, currency=?4
                WHERE id=?1",
            params![id, ticker.name, 
            ticker.source as i64,
            ticker.currency.to_string()],
        )
        .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }
    fn delete_ticker(&self, id: usize) -> Result<(), DataError> {
        self.conn
            .execute("DELETE FROM ticker WHERE id=?1;", params![id as i64])
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }
    
    // insert, get, update and delete for market data sources
    fn insert_quote(&self, quote: &Quote) -> Result<usize, DataError> {
        self.conn
            .execute(
                "INSERT INTO quotes (ticker_id, price, time, volume) VALUES (?, ?, ?, ?, ?)",
                params![quote.ticker as i64, quote.price, 
                quote.time.to_rfc3339(), 
                quote.volume],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        let id = self
            .conn
            .query_row(
                "SELECT last_insert_rowid();", NO_PARAMS,
                |row| {
                    let id: i64 = row.get(0)?;
                    Ok(id as usize)
                },
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok(id)
    }
    fn get_last_quote_before(&self, ticker: usize, time: DateTime<Utc>) -> Result<(Quote,Currency), DataError> {
        let time = time.to_rfc3339();
        let row = self
            .conn
            .query_row(
                "SELECT q.id, q.price, q.time, q.volume, t.currency 
                FROM quotes q, ticker t 
                WHERE t.id=? AND t.id=q.ticker_id AND q.time<=?;",
                params![ticker as i64, time],
                |row| {
                    let id: i64 = row.get(0)?;
                    let price: f64 = row.get(1)?;
                    let time: String = row.get(2)?;
                    let volume: Option<f64> = row.get(3)?;
                    let currency: String = row.get(4)?;
                    Ok((id, price, time, volume, currency))
                },
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let (id, price, time, volume, currency) = row;
        let currency = Currency::from_str(&currency).map_err(|e| DataError::NotFound(e.to_string()))?;
        let time = to_time(&time).map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok((Quote{id: Some(id as usize), ticker, price, time, volume}, currency))
    }
    fn get_all_quotes_for_ticker(&self, ticker_id: usize) -> Result<Vec<Quote>, DataError> {
        let mut stmt = self
        .conn
        .prepare("SELECT id, price, time, volume FROM quotes 
            WHERE ticker_id=? ORDER BY time ASC;")
        .map_err(|e| DataError::NotFound(e.to_string()))?;
    let quotes_map = stmt
        .query_map(params![ticker_id as i64], |row| {
            let id: i64 = row.get(0)?;
            let price: f64 = row.get(1)?;
            let time: String = row.get(2)?;
            let volume: Option<f64> = row.get(3)?;
            let raw_quote = (id, price, time, volume);
            Ok(raw_quote)
        })
        .map_err(|e| DataError::NotFound(e.to_string()))?;
    let mut quotes = Vec::new();
    for quote in quotes_map {
        let (id, price, time, volume) = quote.unwrap();
        let time = to_time(&time)?;
        quotes.push(Quote {
            id: Some(id as usize),
            ticker: ticker_id,
            price,
            time,
            volume,
        });
    }
    Ok(quotes)
    }

    fn update_quote(&self, quote: &Quote) -> Result<(), DataError> {
        if quote.id.is_none() {
            return Err(DataError::NotFound(
                "not yet stored to database".to_string(),
            ));
        }
        let id = quote.id.unwrap() as i64;
        self.conn
        .execute(
            "UPDATE quotes SET ticker_id=?2, price=?2, time=?4, volume=?5
                WHERE id=?1",
            params![id, quote.ticker as i64, quote.price, 
            quote.time.to_rfc3339(),
            quote.volume],
        )
        .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }
    fn delete_quote(&self, id: usize) -> Result<(), DataError> {
        self.conn
            .execute("DELETE FROM quotes WHERE id=?1;", params![id as i64])
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }
}