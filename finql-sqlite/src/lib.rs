///! Implementation of sqlite3 data handler

use std::path::Path;
use deadpool_sqlite::{Config, Runtime, Pool};
use thiserror::Error;

//pub mod asset_handler;
//pub mod quote_handler;
//pub mod transaction_handler;
//pub mod object_handler;

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
}


/// Struct to handle connections to sqlite3 databases
pub struct SqliteDB {
    /// pool is made public to allow extending this struct outside of the library
    pool: Pool,
}

impl SqliteDB {
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

    /// Clean database by dropping all tables and than run init
    pub async fn clean(&self) -> Result<(), SQLiteError> {
        let conn = self.pool.get().await.map_err(|_| SQLiteError::DeadPoolError)?;
        let _ = conn.interact(|conn| -> Result<(), SQLiteError> {
            let mut stmt = conn.prepare(
                "DROP TABLE IF EXISTS transactions;
                DROP TABLE IF EXISTS quotes;
                DROP TABLE IF EXISTS ticker;
                DROP TABLE IF EXISTS assets;
                DROP TABLE IF EXISTS rounding_digits;
                ")?;
                stmt.execute([])?;
                Ok(())
        }).await?;
        self.init().await
    }

    /// Initialize new database by creating table, fill
    pub async fn init(&self) -> Result<(), SQLiteError> {
        let conn = self.pool.get().await.map_err(|_| SQLiteError::DeadPoolError)?;
        let _ = conn.interact(|conn| -> Result<(), SQLiteError> {
            let mut stmt = conn.prepare(
            "CREATE TABLE IF NOT EXISTS assets (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                wkn TEXT UNIQUE,
                isin TEXT UNIQUE,
                note TEXT
            );
            CREATE TABLE IF NOT EXISTS transactions (
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
            );
            CREATE TABLE IF NOT EXISTS ticker (
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
            );
            CREATE TABLE IF NOT EXISTS quotes (
                id INTEGER PRIMARY KEY,
                ticker_id INTEGER NOT NULL,
                price REAL NOT NULL,
                time TEXT NOT NULL,
                volume REAL,
                FOREIGN KEY(ticker_id) REFERENCES ticker(id) 
            );
            CREATE TABLE IF NOT EXISTS rounding_digits (
                id INTEGER PRIMARY KEY,
                currency TEXT NOT NULL UNIQUE,
                digits INT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS objects (
                id TEXT PRIMARY KEY,
                object TEXT NOT NULL
            );")?;
            stmt.execute([])?;
            Ok(())
        }).await?;
        Ok(())
    }

    // pub async fn insert(&self, name: String, age: i32) -> Result<usize, SQLiteError> {
    //     let conn = self.pool.get().await.map_err(|_| SQLiteError::DeadPoolError)?;
    //     let age_string = age.to_string();
    //     conn.interact(move |conn| {
    //         let mut stmt = conn.prepare(
    //             "INSERT INTO person (name, age)
    //             VALUES(?,?)")?;
    //         Ok(stmt.execute([&name, &age_string])?)
    //     }).await?
    // }

    // pub async fn count(&self) -> Result<usize, SQLiteError> {
    //     let conn = self.pool.get().await.map_err(|_| SQLiteError::DeadPoolError)?;
    //     conn.interact(move |conn| {
    //         let mut stmt = conn.prepare(
    //             "SELECT COUNT(id) FROM person")?;
    //         let mut rows = stmt.query([])?;
    //         if let Some(row) = rows.next()? {
    //             Ok(row.get(0)?)
    //         } else {
    //             Ok(0)
    //         }
    //     }).await?
    // }
}


#[cfg(test)]
mod test {
use super::*;
//use tempfile::tempdir;
use std::sync::Arc;

    #[tokio::test]
    async fn file_create_insert_query() {
        //let tmp_dir = tempdir().unwrap();
        //let db_path = tmp_dir.path().join("test.db");
        let db_path = Path::new("data/test.db");

        let db = Arc::new(SqliteDB::open(&db_path).await.unwrap());
        assert!(db.clean().await.is_ok());
    }

    // #[tokio::test]
    // async fn file_create_insert_query() {
    //     let tmp_dir = tempdir().unwrap();
    //     let db_path = tmp_dir.path().join("test.db");

    //     let db = Arc::new(SqliteDB::open(&db_path).await.unwrap());
    //     assert_eq!(db.create().await.unwrap(), 0);

    //     // spawn tasks that run in parallel
    //     let tasks = vec![
    //         db.insert("Karl".to_string(), 21),
    //         db.insert("Franz".to_string(), 20),
    //         db.insert("Mathilda".to_string(), 25),
    //         db.insert("Karl".to_string(), 27),
    //         db.insert("Juliane".to_string(), 22)
    //     ];

    //     // now await them to get the resolve's to complete
    //     let mut results = Vec::new();
    //     for task in tasks {
    //         results.push(task.await.unwrap());
    //     }
    //     assert_eq!(results, vec![1,1,1,1,1]);
    //     assert_eq!(db.count().await.unwrap(), 5);
    // }

    // #[tokio::test]
    // async fn memory_create_insert_query() {
    //     let db = Arc::new(SqliteDB::in_memory().await.unwrap());
    //     assert_eq!(db.create().await.unwrap(), 0);

    //     // spawn tasks that run in parallel
    //     let tasks = vec![
    //         db.insert("Karl".to_string(), 21),
    //         db.insert("Franz".to_string(), 20),
    //         db.insert("Mathilda".to_string(), 25),
    //         db.insert("Karl".to_string(), 27),
    //         db.insert("Juliane".to_string(), 22)
    //     ];

    //     // now await them to get the resolve's to complete
    //     let mut results = Vec::new();
    //     for task in tasks {
    //         results.push(task.await.unwrap());
    //     }
    //     assert_eq!(results, vec![1,1,1,1,1]);
    //     assert_eq!(db.count().await.unwrap(), 5);
    // }
}
