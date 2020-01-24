///! Implementation for quote handler with Sqlite3 database as backend

use super::PostgresDB;
use crate::currency::Currency;
use crate::data_handler::{DataError, QuoteHandler};
use crate::quote::{MarketDataSource, Quote, Ticker};
use chrono::{DateTime, Utc};
use std::str::FromStr;
use crate::helpers::{to_time};

/// Sqlite implementation of quote handler
impl QuoteHandler for PostgresDB {
    // insert, get, update and delete for market data sources
    fn insert_md_source(&mut self, source: &MarketDataSource) -> Result<usize, DataError> {
        self.conn
            .execute(
                "INSERT INTO market_data_sources (name) VALUES ($1)",
                &[&source.name],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        let row = self.conn.query_one(
                "SELECT id FROM market_data_sources WHERE name=$1;",
                &[&source.name])
                .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        let id: i32 = row.get::<_,i32>(0);
        Ok(id as usize)
    }

    fn get_md_source_by_id(&mut self, id: usize) -> Result<MarketDataSource, DataError> {
        let row = self
            .conn
            .query_one(
                "SELECT name FROM market_data_sources WHERE id=$1;",
                &[&(id as i32)])
                .map_err(|e| DataError::NotFound(e.to_string()))?;
        let name: String = row.get(0);
        Ok(MarketDataSource {
                        id: Some(id as usize),
                        name: name,
                    })
    }

    fn get_all_md_sources(&mut self) -> Result<Vec<MarketDataSource>, DataError> {
        let mut sources = Vec::new();
        for row in self.conn.query(
            "SELECT id, name FROM market_data_sources", &[])
            .map_err(|e| DataError::NotFound(e.to_string()))? {
                let id: i32 = row.get(0);
                sources.push(MarketDataSource {
                    id: Some(id as usize),
                    name: row.get(1),
                });
        
            }
        Ok(sources)
    }

    fn update_md_source(&mut self, source: &MarketDataSource) -> Result<(), DataError> {
        if source.id.is_none() {
            return Err(DataError::NotFound(
                "not yet stored to database".to_string(),
            ));
        }
        let id = source.id.unwrap() as i32;
        self.conn
            .execute(
                "UPDATE market_data_sources SET name=$2 
                WHERE id=$1;",
                &[&id, &source.name],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }
    fn delete_md_source(&mut self, id: usize) -> Result<(), DataError> {
        self.conn
            .execute(
                "DELETE FROM market_data_sources WHERE id=$1;",
                &[&(id as i32)],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    // insert, get, update and delete for market data sources
    fn insert_ticker(&mut self, ticker: &Ticker) -> Result<usize, DataError> {
        self.conn
            .execute(
                "INSERT INTO ticker (name, source_id, currency) VALUES ($1, $2, $3)",
                &[
                    &ticker.name,
                    &(ticker.source as i32),
                    &(ticker.currency.to_string())
                ],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        let row = self.conn.query_one(
                "SELECT id FROM ticker WHERE name=$1 AND source_id=$2;",
                &[&ticker.name, &(ticker.source as i32)])
                .map_err(|e| DataError::NotFound(e.to_string()))?;
        let id: i32 = row.get(0);
        Ok(id as usize)
    }
    fn get_ticker_by_id(&mut self, id: usize) -> Result<Ticker, DataError> {
        let row = self.conn
            .query_one(
                "SELECT name, source_id, currency FROM ticker WHERE id=$1;",
                &[&(id as i32)])
                .map_err(|e| DataError::NotFound(e.to_string()))?;
        let name: String = row.get(0);
        let source: i32 = row.get(1);
        let currency: String = row.get(2);
        let currency = 
            Currency::from_str(&currency).map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok(Ticker {
            id: Some(id),
            name,
            source: source as usize,
            currency,
        })
    }
    fn get_all_ticker_for_source(&mut self, source: usize) -> Result<Vec<Ticker>, DataError> {
        let mut all_ticker = Vec::new();
        for row in self.conn.query(
            "SELECT id, name, currency FROM ticker WHERE source_id=$1;", &[&(source as i32)])
            .map_err(|e| DataError::NotFound(e.to_string()))?
        {
                let id: i32 = row.get(0);
                let currency: String = row.get(2);
                let currency =
                    Currency::from_str(&currency).map_err(|e| DataError::NotFound(e.to_string()))?;
                all_ticker.push(Ticker {
                    id: Some(id as usize),
                    name: row.get(1),
                    source,
                    currency,
                });
        }
        Ok(all_ticker)
    }

    fn update_ticker(&mut self, ticker: &Ticker) -> Result<(), DataError> {
        if ticker.id.is_none() {
            return Err(DataError::NotFound(
                "not yet stored to database".to_string(),
            ));
        }
        let id = ticker.id.unwrap() as i32;
        self.conn
            .execute(
                "UPDATE ticker SET name=$2, source_id=$3, currency=$4
                WHERE id=$1",
                &[
                    &id,
                    &ticker.name,
                    &(ticker.source as i32),
                    &ticker.currency.to_string()
                ],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    fn delete_ticker(&mut self, id: usize) -> Result<(), DataError> {
        self.conn
            .execute("DELETE FROM ticker WHERE id=$1;", &[&(id as i32)])
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    // insert, get, update and delete for market data sources
    fn insert_quote(&mut self, quote: &Quote) -> Result<usize, DataError> {
        let row = self.conn.query_one(
            "INSERT INTO quotes (ticker_id, price, time, volume) 
                VALUES ($1, $2, $3, $4) RETURNING ticker_id",
                &[
                    &(quote.ticker as i32),
                    &quote.price,
                    &quote.time.to_rfc3339(),
                    &quote.volume
                ])
                .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        let id: i32 = row.get(0);
        Ok(id as usize)
    }

   fn get_last_quote_before(
        &mut self,
        ticker: usize,
        time: DateTime<Utc>,
    ) -> Result<(Quote, Currency), DataError> {
        let time = time.to_rfc3339();
        let row = self.conn.query_one(
                "SELECT q.id, q.price, q.time, q.volume, t.currency 
                FROM quotes q, ticker t 
                WHERE t.id=$1 AND t.id=q.ticker_id AND q.time<=$1;",
                &[&(ticker as i32), &(time)])
                .map_err(|e| DataError::NotFound(e.to_string()))?;


        let id: i32 = row.get(0);
        let price: f64 = row.get(1);
        let time: String = row.get(2);
        let volume: Option<f64> = row.get(3);
        let currency: String = row.get(4);
        let currency =
            Currency::from_str(&currency).map_err(|e| DataError::NotFound(e.to_string()))?;
        let time = to_time(&time).map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok((
            Quote {
                id: Some(id as usize),
                ticker,
                price,
                time,
                volume,
            },
            currency,
        ))
    }

    fn get_all_quotes_for_ticker(&mut self, ticker_id: usize) -> Result<Vec<Quote>, DataError> {
        let mut quotes = Vec::new();
        for row in self.conn.query(
                "SELECT id, price, time, volume FROM quotes 
                WHERE ticker_id=$1 ORDER BY time ASC;", &[]
            )
            .map_err(|e| DataError::NotFound(e.to_string()))? {
                let id: i32 = row.get(0);
                let time: String = row.get(2);
                quotes.push(Quote {
                    id: Some(id as usize),
                    ticker: ticker_id,
                    price: row.get(1),
                    time: to_time(&time)?,
                    volume: row.get(3),
                });
            }
        Ok(quotes)
    }

    fn update_quote(&mut self, quote: &Quote) -> Result<(), DataError> {
        if quote.id.is_none() {
            return Err(DataError::NotFound(
                "not yet stored to database".to_string(),
            ));
        }
        let id = quote.id.unwrap() as i32;
        self.conn
            .execute(
                "UPDATE quotes SET ticker_id=$2, price=$2, time=$4, volume=$5
                WHERE id=$1",
                &[
                    &id,
                    &(quote.ticker as i32),
                    &quote.price,
                    &quote.time.to_rfc3339(),
                    &quote.volume
                ],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    fn delete_quote(&mut self, id: usize) -> Result<(), DataError> {
        self.conn
            .execute("DELETE FROM quotes WHERE id=$1;", &[&(id as i32)])
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }
}
