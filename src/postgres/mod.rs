///! Implementation of PostgreSQL data handler
use sqlx::postgres::{PgPoolOptions, Postgres};

pub mod asset_handler;
pub mod quote_handler;
pub mod transaction_handler;
pub mod object_handler;

/// Struct to handle connections to postgres databases
pub struct PostgresDB {
    /// pool is made public to allow extending this struct outside of the library
    pub pool: sqlx::Pool<Postgres>,
}

impl PostgresDB {
    pub async fn new(connection_string: &str) -> Result<PostgresDB, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(connection_string)
            .await?;
        Ok(PostgresDB { pool })
    }

    /// Clean database by dropping all tables and than run init
    pub async fn clean(&self) -> Result<(), sqlx::Error> {
        sqlx::query!("DROP TABLE IF EXISTS transactions")
            .execute(&self.pool)
            .await?;
        sqlx::query!("DROP TABLE IF EXISTS quotes")
            .execute(&self.pool)
            .await?;
        sqlx::query!("DROP TABLE IF EXISTS ticker")
            .execute(&self.pool)
            .await?;
        sqlx::query!("DROP TYPE IF EXISTS market_data_source")
            .execute(&self.pool)
            .await?;
        sqlx::query!("DROP TABLE IF EXISTS currencies")
            .execute(&self.pool)
            .await?;
        sqlx::query!("DROP TABLE IF EXISTS stocks")
            .execute(&self.pool)
            .await?;
        sqlx::query!("DROP TABLE IF EXISTS assets")
            .execute(&self.pool)
            .await?;
        self.init().await
    }

    /// Initialize new database by creating table
    pub async fn init(&self) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "CREATE TABLE IF NOT EXISTS assets (
                id SERIAL PRIMARY KEY,
                asset_class VARCHAR(20) NOT NULL
            )"
        )
        .execute(&self.pool)
        .await?;
        sqlx::query!(
                "CREATE TABLE IF NOT EXISTS currencies (
                    id INTEGER PRIMARY KEY,
                    iso_code CHAR(3) NOT NULL UNIQUE,
                    rounding_digits INT NOT NULL,
                    FOREIGN KEY(id) REFERENCES assets(id)
                )"
        )
            .execute(&self.pool)
            .await?;
        sqlx::query!(
                "CREATE TABLE IF NOT EXISTS stocks (
                  id INTEGER PRIMARY KEY,
                  name TEXT NOT NULL UNIQUE,
                  wkn CHAR(6) UNIQUE,
                  isin CHAR(12) UNIQUE,
                  note TEXT,
                  FOREIGN KEY(id) REFERENCES assets(id)
                )"
        )
            .execute(&self.pool)
            .await?;
        sqlx::query!(
            "CREATE TABLE IF NOT EXISTS transactions (
                id SERIAL PRIMARY KEY,
                trans_type TEXT NOT NULL,
                asset_id INTEGER,
                cash_amount FLOAT8 NOT NULL,
                cash_currency_id INT NOT NULL,
                cash_date DATE NOT NULL,
                related_trans INTEGER,
                position FLOAT8,
                note TEXT,
                FOREIGN KEY(asset_id) REFERENCES assets(id),
                FOREIGN KEY(cash_currency_id) REFERENCES currencies(id),
                FOREIGN KEY(related_trans) REFERENCES transactions(id)
            )"
        )
        .execute(&self.pool)
        .await?;
        sqlx::query!(
            "CREATE TABLE IF NOT EXISTS ticker (
                id SERIAL PRIMARY KEY,
                name TEXT NOT NULL,
                asset_id INTEGER NOT NULL,
                source TEXT NOT NULL,
                priority INTEGER NOT NULL,
                currency_id INT NOT NULL,
                factor FLOAT8 NOT NULL DEFAULT 1.0,
                tz TEXT,
                cal TEXT,
                FOREIGN KEY(asset_id) REFERENCES assets(id),
                FOREIGN KEY(currency_id) REFERENCES currencies(id)
            )"
        )
        .execute(&self.pool)
        .await?;
        sqlx::query!(
            "CREATE TABLE IF NOT EXISTS quotes (
                id SERIAL PRIMARY KEY,
                ticker_id INTEGER NOT NULL,
                price FLOAT8 NOT NULL,
                time TIMESTAMP WITH TIME ZONE NOT NULL,
                volume FLOAT8,
                FOREIGN KEY(ticker_id) REFERENCES ticker(id) 
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!(
            "CREATE TABLE IF NOT EXISTS objects (
            id TEXT PRIMARY KEY,
            object JSON NOT NULL)"
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
