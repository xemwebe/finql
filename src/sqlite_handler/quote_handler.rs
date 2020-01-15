///! Implementation for quote handler with Sqlite3 database as backend
use super::SqliteDB;
use crate::data_handler::{QuoteHandler, DataError};
use crate::quote::{MarketDataSource, Ticker, Quote};
use chrono::{DateTime, Utc};
use rusqlite::{params, NO_PARAMS};
use super::helpers::{usize_to_int,int_to_usize};

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
                "INSERT INTO ticker (name, source_id) VALUES (?, ?)",
                params![ticker.name, ticker.source as i64],
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
    fn get_all_ticker_for_asset(&self, asset_id: usize) -> Result<Vec<Ticker>, DataError> {
        // let mut stmt = self
        //     .conn
        //     .prepare("SELECT id, name FROM market_data_sources;")
        //     .map_err(|e| DataError::NotFound(e.to_string()))?;
        // let sources_map = stmt
        //     .query_map(NO_PARAMS, |row| {
        //         let id: i64 = row.get(0)?;
        //         let id = Some(id as usize);
        //         Ok(MarketDataSource {
        //             id,
        //             name: row.get(1)?,
        //         })
        //     })
        //     .map_err(|e| DataError::NotFound(e.to_string()))?;
        // let mut sources = Vec::new();
        // for source in sources_map {
        //     sources.push(source.unwrap());
        // }
        // Ok(sources)
        Ok(Vec::new())
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
            "UPDATE ticker SET name=?2, source_id=?3
                WHERE id=?1",
            params![id, ticker.name, ticker.source as i64],
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
                params![quote.ticker as i64, quote.price.amount, 
                quote.price.currency.to_string(), 
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
    fn get_last_quote_before(&self, ticker: usize, time: DateTime<Utc>) -> Result<Quote, DataError> {
        Err(DataError::NotFound("test".to_string()))
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
            "UPDATE quotes SET ticker_id=?2, price=?2, currency=?4, time=?5, volume=?6
                WHERE id=?1",
            params![id, quote.ticker as i64, quote.price.amount, 
            quote.price.currency.to_string(), 
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