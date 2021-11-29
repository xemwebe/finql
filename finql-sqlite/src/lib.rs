///! Implementation of sqlite3 data handler

use std::path::Path;
use deadpool_sqlite::{Config, Runtime, Pool, Connection};
use thiserror::Error;
use finql_data::currency::CurrencyError;

pub mod asset_handler;
//pub mod quote_handler;
//pub mod transaction_handler;
pub mod object_handler;

#[derive(Error, Debug)]
pub enum SQLiteError {
    #[error("Failed to create pool")]
    CreatePoolFailed(#[from] deadpool_sqlite::CreatePoolError),
    #[error("Failed to execute SQL statement")]
    QueryError(#[from] deadpool_sqlite::rusqlite::Error),
    #[error("Failed to interact with connetion pool")]
    PoolError(#[from] deadpool_sqlite::InteractError),
    #[error("Failed to get connection")]
    DeadPoolError,
    #[error("Query returned invalid result")]
    InvalidQueryResult,
    #[error("Malformed currency")]
    MalformedCurrency(#[from] CurrencyError),
}


/// Pool of connections to sqlite3 databases
pub struct SqliteDBPool {
    /// pool is made public to allow extending this struct outside of the library
    pool: Pool,
}

/// Struct to handle connections to sqlite3 databases
pub struct SqliteDB {
    /// pool is made public to allow extending this struct outside of the library
    conn: Connection,
}

impl SqliteDBPool {
    /// Create a new in memory database
    pub async fn in_memory() -> Result<Self, SQLiteError> {
        let cfg = Config::new(":memory:");
        Ok(Self {
            pool: cfg.create_pool(Runtime::Tokio1)?,
        })
    }

    /// Open a connection to a file based database
    pub async fn open(path: &Path) -> Result<Self, SQLiteError> {
        let cfg = Config::new(path);
        Ok(Self {
            pool: cfg.create_pool(Runtime::Tokio1)?,
        })
    }

    /// Get connection to Sqlite pool
    pub async fn get_conection(&self) -> Result<SqliteDB, SQLiteError> {
        Ok(SqliteDB{
            conn: self.pool.get().await.map_err(|_| SQLiteError::DeadPoolError)?
        })
    }
}

impl SqliteDB {
    /// Clean database by dropping all tables and than run init
    pub async fn clean(&self) -> Result<(), SQLiteError> {
        let _ = self.conn.interact(|conn| -> Result<(), SQLiteError> {
            let mut stmt = conn.prepare("DROP TABLE IF EXISTS transactions")?;
            stmt.execute([])?;
            stmt = conn.prepare("DROP TABLE IF EXISTS quotes")?;
            stmt.execute([])?;
            stmt = conn.prepare("DROP TABLE IF EXISTS ticker")?;
            stmt.execute([])?;
            stmt = conn.prepare("DROP TABLE IF EXISTS assets")?;
            stmt.execute([])?;
            stmt = conn.prepare("DROP TABLE IF EXISTS rounding_digits")?;
            stmt.execute([])?;
            Ok(())
        }).await?;
        self.init().await
    }

    /// Initialize new database by creating table, fill
    pub async fn init(&self) -> Result<(), SQLiteError> {
        let _ = self.conn.interact(|conn| -> Result<(), SQLiteError> {
            conn.execute(
            "CREATE TABLE IF NOT EXISTS assets (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                wkn TEXT UNIQUE,
                isin TEXT UNIQUE,
                note TEXT)", [])?;
            conn.execute(
                    "CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER PRIMARY KEY,
                trans_type TEXT NOT NULL,
                asset_id INTEGER,
                cash_amount REAL NOT NULL,
                cash_currency TEXT NOT NULL,
                cash_date TEXT NOT NULL,
                related_trans INTEGER,
                position REAL,
                note TEXT,
                time_stamp INTEGER NOT NULL,
                FOREIGN KEY(asset_id) REFERENCES assets(id),
                FOREIGN KEY(related_trans) REFERENCES transactions(id)
            )", [])?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS ticker (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                asset_id INTEGER NOT NULL,
                source TEXT NOT NULL,
                priority INTEGER NOT NULL,
                currency TEXT NOT NULL,
                factor REAL NOT NULL DEFAULT 1.0,
                tz TEXT,
                cal TEXT,
                FOREIGN KEY(asset_id) REFERENCES assets(id) 
            )", [])?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS quotes (
                id INTEGER PRIMARY KEY,
                ticker_id INTEGER NOT NULL,
                price REAL NOT NULL,
                time TEXT NOT NULL,
                volume REAL,
                FOREIGN KEY(ticker_id) REFERENCES ticker(id) 
            )", [])?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS rounding_digits (
                id INTEGER PRIMARY KEY,
                currency TEXT NOT NULL UNIQUE,
                digits INT NOT NULL
            )", [])?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS objects (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                type TEXT NOT NULL,
                object TEXT NOT NULL
            )", [])?;
            Ok(())
        }).await?;
        Ok(())
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn file_create_insert_query() {
        let db_pool = Arc::new(SqliteDBPool::in_memory().await.unwrap());
        let db = db_pool.get_conection().await.unwrap();
        assert!(db.clean().await.is_ok());
    }
}
