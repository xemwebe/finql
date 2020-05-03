///! Implemenation of PostgreSQL data handler
use postgres::Client;
use tokio_postgres::error::Error;
mod asset_handler;
mod quote_handler;
mod transaction_handler;

pub use transaction_handler::RawTransaction;

/// Struct to handle connections to sqlite3 databases
pub struct PostgresDB<'a> {
    /// conn is made public to allow extending this struct outside of the library
    pub conn: &'a mut Client,
}

impl PostgresDB<'_> {
    /// Clean database by dropping all tables and than run init
    pub fn clean(&mut self) -> Result<(), Error> {
        self.conn
            .execute("DROP TABLE IF EXISTS transactions", &[])?;
        self.conn.execute("DROP TABLE IF EXISTS quotes", &[])?;
        self.conn.execute("DROP TABLE IF EXISTS ticker", &[])?;
        self.conn
            .execute("DROP TYPE IF EXISTS market_data_source", &[])?;
        self.conn.execute("DROP TABLE IF EXISTS assets", &[])?;
        self.conn
            .execute("DROP TABLE IF EXISTS rounding_digits", &[])?;
        self.init()
    }

    /// Initialize new database by creating table
    pub fn init(&mut self) -> Result<(), Error> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS assets (
                id SERIAL PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                wkn TEXT UNIQUE,
                isin TEXT UNIQUE,
                note TEXT
            )",
            &[],
        )?;
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
                id SERIAL PRIMARY KEY,
                trans_type TEXT NOT NULL,
                asset_id INTEGER,
                cash_amount FLOAT8 NOT NULL,
                cash_currency TEXT NOT NULL,
                cash_date DATE NOT NULL,
                related_trans INTEGER,
                position FLOAT8,
                note TEXT,
                FOREIGN KEY(asset_id) REFERENCES assets(id),
                FOREIGN KEY(related_trans) REFERENCES transactions(id)
            );",
            &[],
        )?;
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS ticker (
                id SERIAL PRIMARY KEY,
                name TEXT NOT NULL,
                asset_id INTEGER NOT NULL,
                source TEXT NOT NULL,
                priority INTEGER NOT NULL,
                currency TEXT NOT NULL,
                factor FLOAT8 NOT NULL DEFAULT 1.0,
                FOREIGN KEY(asset_id) REFERENCES assets(id) 
            );",
            &[],
        )?;
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS quotes (
                id SERIAL PRIMARY KEY,
                ticker_id INTEGER NOT NULL,
                price FLOAT8 NOT NULL,
                time TIMESTAMP WITH TIME ZONE NOT NULL,
                volume FLOAT8,
                FOREIGN KEY(ticker_id) REFERENCES ticker(id) );",
            &[],
        )?;
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS rounding_digits (
                id SERIAL PRIMARY KEY,
                currency TEXT NOT NULL UNIQUE,
                digits INT NOT NULL);",
            &[],
        )?;

        Ok(())
    }
}
