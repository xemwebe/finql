///! Implementation of sqlite3 data handler

use sqlx::sqlite::SqlitePool;

pub mod asset_handler;
pub mod quote_handler;
pub mod transaction_handler;

/// Struct to handle connections to sqlite3 databases
pub struct SqliteDB {
    /// pool is made public to allow extending this struct outside of the library
    pub pool: SqlitePool,
}

impl SqliteDB {
    pub async fn new(connection_string: &str) -> Result<SqliteDB, sqlx::Error> {
        let pool = SqlitePool::connect(connection_string).await?;
        Ok(SqliteDB{pool})
    }

    /// Clean database by dropping all tables and than run init
    pub async fn clean(&mut self) -> Result<(), sqlx::Error> {
        sqlx::query!("DROP TABLE IF EXISTS transactions").execute(&self.pool).await?;
        sqlx::query!("DROP TABLE IF EXISTS quotes").execute(&self.pool).await?;
        sqlx::query!("DROP TABLE IF EXISTS ticker").execute(&self.pool).await?;
        sqlx::query!("DROP TABLE IF EXISTS assets").execute(&self.pool).await?;
        sqlx::query!("DROP TABLE IF EXISTS rounding_digits").execute(&self.pool).await?;
        self.init().await
    }

    /// Initialize new database by creating table, fill
    pub async fn init(&mut self) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "CREATE TABLE IF NOT EXISTS assets (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                wkn TEXT UNIQUE,
                isin TEXT UNIQUE,
                note TEXT
            )").execute(&self.pool).await?;
            
        sqlx::query!(
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
                FOREIGN KEY(asset_id) REFERENCES assets(id),
                FOREIGN KEY(related_trans) REFERENCES transactions(id)
            )").execute(&self.pool).await?;

        sqlx::query!(
            "CREATE TABLE IF NOT EXISTS ticker (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                asset_id INTEGER NOT NULL,
                source TEXT NOT NULL,
                priority INTEGER NOT NULL,
                currency TEXT NOT NULL,
                factor REAL NOT NULL DEFAULT 1.0,
                FOREIGN KEY(asset_id) REFERENCES assets(id) 
            )").execute(&self.pool).await?;
            
        sqlx::query!(
            "CREATE TABLE IF NOT EXISTS quotes (
                id INTEGER PRIMARY KEY,
                ticker_id INTEGER NOT NULL,
                price REAL NOT NULL,
                time TEXT NOT NULL,
                volume REAL,
                FOREIGN KEY(ticker_id) REFERENCES ticker(id) 
            )").execute(&self.pool).await?;

        sqlx::query!(
            "CREATE TABLE IF NOT EXISTS rounding_digits (
                id INTEGER PRIMARY KEY,
                currency TEXT NOT NULL UNIQUE,
                digits INT NOT NULL
            )").execute(&self.pool).await?;

        Ok(())
    }
}
