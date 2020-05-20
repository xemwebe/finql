///! Implementation for quote handler with Sqlite3 database as backend
use super::PostgresDB;
use crate::currency::Currency;
use crate::data_handler::{DataError, QuoteHandler};
use crate::quote::{MarketDataSource, Quote, Ticker};
use chrono::{DateTime, Utc};
use std::str::FromStr;

/// Sqlite implementation of quote handler
impl QuoteHandler for PostgresDB<'_> {
    // insert, get, update and delete for market data sources
    fn insert_ticker(&mut self, ticker: &Ticker) -> Result<usize, DataError> {
        let row = self
            .conn
            .query_one(
                "INSERT INTO ticker (name, asset_id, source, priority, currency, factor) 
                VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
                &[
                    &ticker.name,
                    &(ticker.asset as i32),
                    &(ticker.source.to_string()),
                    &ticker.priority,
                    &(ticker.currency.to_string()),
                    &ticker.factor,
                ],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        let id: i32 = row.get(0);
        Ok(id as usize)
    }

    fn get_ticker_id(&mut self, ticker: &str) -> Option<usize> {
        let row = self
            .conn
            .query_one("SELECT id FROM ticker WHERE name=$1", &[&ticker]);
        match row {
            Ok(row) => {
                let id: i32 = row.get(0);
                Some(id as usize)
            }
            _ => None,
        }
    }

    fn get_ticker_by_id(&mut self, id: usize) -> Result<Ticker, DataError> {
        let row = self
            .conn
            .query_one(
                "SELECT name, asset_id, source, priority, currency, factor FROM ticker WHERE id=$1;",
                &[&(id as i32)],
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let name: String = row.get(0);
        let asset: i32 = row.get(1);
        let source: String = row.get(2);
        let source =
            MarketDataSource::from_str(&source).map_err(|e| DataError::NotFound(e.to_string()))?;
        let currency: String = row.get(4);
        let currency =
            Currency::from_str(&currency).map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok(Ticker {
            id: Some(id),
            name,
            asset: asset as usize,
            source,
            priority: row.get(3),
            currency,
            factor: row.get(5),
        })
    }
    fn get_all_ticker(&mut self) -> Result<Vec<Ticker>, DataError> {
        let mut all_ticker = Vec::new();
        for row in self
            .conn
            .query(
                "SELECT id, name, asset_id, priority, source, currency, factor FROM ticker",
                &[],
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?
        {
            let id: i32 = row.get(0);
            let asset: i32 = row.get(2);
            let source: String = row.get(4);
            let source = MarketDataSource::from_str(&source)
                .map_err(|e| DataError::NotFound(e.to_string()))?;
            let currency: String = row.get(5);
            let currency =
                Currency::from_str(&currency).map_err(|e| DataError::NotFound(e.to_string()))?;
            let factor: f64 = row.get(6);
            all_ticker.push(Ticker {
                id: Some(id as usize),
                name: row.get(1),
                asset: asset as usize,
                source,
                priority: row.get(3),
                currency,
                factor,
            });
        }
        Ok(all_ticker)
    }

    fn get_all_ticker_for_source(
        &mut self,
        source: MarketDataSource,
    ) -> Result<Vec<Ticker>, DataError> {
        let mut all_ticker = Vec::new();
        for row in self
            .conn
            .query(
                "SELECT id, name, asset_id, priority, currency, factor FROM ticker WHERE source=$1;",
                &[&(source.to_string())],
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?
        {
            let id: i32 = row.get(0);
            let asset: i32 = row.get(2);
            let currency: String = row.get(4);
            let currency =
                Currency::from_str(&currency).map_err(|e| DataError::NotFound(e.to_string()))?;
            let factor: f64 = row.get(5);
            all_ticker.push(Ticker {
                id: Some(id as usize),
                name: row.get(1),
                asset: asset as usize,
                source,
                priority: row.get(3),
                currency,
                factor,
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
                "UPDATE ticker SET name=$2, asset_id=$3, source=$4, priority=$5, currency=$6, factor=$7
                WHERE id=$1",
                &[
                    &id,
                    &ticker.name,
                    &(ticker.asset as i32),
                    &ticker.source.to_string(),
                    &ticker.priority,
                    &ticker.currency.to_string(),
                    &ticker.factor,
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
        let row = self
            .conn
            .query_one(
                "INSERT INTO quotes (ticker_id, price, time, volume) 
                VALUES ($1, $2, $3, $4) RETURNING id",
                &[
                    &(quote.ticker as i32),
                    &quote.price,
                    &quote.time,
                    &quote.volume,
                ],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        let id: i32 = row.get(0);
        Ok(id as usize)
    }

    fn get_last_price_before(
        &mut self,
        asset_name: &str,
        time: DateTime<Utc>,
    ) -> Result<(Quote, Currency), DataError> {
        let row = self
            .conn
            .query_one(
                "SELECT q.id, q.ticker_id, q.price, q.time, q.volume, t.currency, t.priority
                FROM quotes q, ticker t, assets a 
                WHERE a.name=$1 AND t.asset_id=a.id AND t.id=q.ticker_id AND q.time<= $2
                ORDER BY q.time DESC, t.priority ASC LIMIT 1",
                &[&asset_name, &time],
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;

        let id: i32 = row.get(0);
        let ticker: i32 = row.get(1);
        let price: f64 = row.get(2);
        let time: DateTime<Utc> = row.get(3);
        let volume: Option<f64> = row.get(4);
        let currency: String = row.get(5);
        let currency =
            Currency::from_str(&currency).map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok((
            Quote {
                id: Some(id as usize),
                ticker: ticker as usize,
                price,
                time,
                volume,
            },
            currency,
        ))
    }

    fn get_last_price_before_by_id(
        &mut self,
        asset_id: usize,
        time: DateTime<Utc>,
    ) -> Result<(Quote, Currency), DataError> {
        let row = self
            .conn
            .query_one(
                "SELECT q.id, q.ticker_id, q.price, q.time, q.volume, t.currency, t.priority
                FROM quotes q, ticker t
                WHERE t.asset_id=$1 AND t.id=q.ticker_id AND q.time<= $2
                ORDER BY q.time DESC, t.priority ASC LIMIT 1",
                &[&(asset_id as i32), &time],
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;

        let id: i32 = row.get(0);
        let ticker: i32 = row.get(1);
        let price: f64 = row.get(2);
        let time: DateTime<Utc> = row.get(3);
        let volume: Option<f64> = row.get(4);
        let currency: String = row.get(5);
        let currency =
            Currency::from_str(&currency).map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok((
            Quote {
                id: Some(id as usize),
                ticker: ticker as usize,
                price,
                time,
                volume,
            },
            currency,
        ))
    }

    fn get_all_quotes_for_ticker(&mut self, ticker_id: usize) -> Result<Vec<Quote>, DataError> {
        let mut quotes = Vec::new();
        for row in self
            .conn
            .query(
                "SELECT id, price, time, volume FROM quotes 
                WHERE ticker_id=$1 ORDER BY time ASC;",
                &[&(ticker_id as i32)],
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?
        {
            let id: i32 = row.get(0);
            let time: DateTime<Utc> = row.get(2);
            quotes.push(Quote {
                id: Some(id as usize),
                ticker: ticker_id,
                price: row.get(1),
                time,
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
                "UPDATE quotes SET ticker_id=$2, price=$3, time=$4, volume=$5
                WHERE id=$1",
                &[
                    &id,
                    &(quote.ticker as i32),
                    &quote.price,
                    &quote.time,
                    &quote.volume,
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

    fn get_rounding_digits(&mut self, currency: Currency) -> i32 {
        let rows = self.conn.query(
            "SELECT digits FROM rounding_digits WHERE currency=$1;",
            &[&currency.to_string()],
        );
        match rows {
            Ok(row_vec) => {
                if row_vec.len() > 0 {
                    let digits: i32 = row_vec[0].get(0);
                    digits
                } else {
                    2
                }
            }
            Err(_) => 2,
        }
    }

    fn set_rounding_digits(&mut self, currency: Currency, digits: i32) -> Result<(), DataError> {
        let _row = self
            .conn
            .execute(
                "INSERT INTO rounding_digits (currency, digits) VALUES ($1, $2)",
                &[&currency.to_string(), &digits],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }
}
