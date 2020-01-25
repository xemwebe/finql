///! Implemenation of PostgreSQL data handler
use postgres::{Client, NoTls};
use tokio_postgres::error::Error;

pub mod quote_handler;
pub mod transaction_handler;

/// Struct to handle connections to sqlite3 databases
pub struct PostgresDB {
    conn: Client,
}

impl PostgresDB {
    pub fn connect(conn_str: &str) -> Result<PostgresDB, Error> {
        let conn = Client::connect(conn_str, NoTls)?;
        Ok(PostgresDB { conn })
    }

    /// Clean database by dropping all tables and than run init
    pub fn clean(&mut self) -> Result<(), Error> {
        self.conn
            .execute("DROP TABLE IF EXISTS transactions", &[])?;
        self.conn.execute("DROP TABLE IF EXISTS assets", &[])?;
        self.conn.execute("DROP TABLE IF EXISTS quotes", &[])?;
        self.conn.execute("DROP TABLE IF EXISTS ticker", &[])?;
        self.conn
            .execute("DROP TABLE IF EXISTS market_data_sources", &[])?;
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
            "CREATE TABLE IF NOT EXISTS market_data_sources (
                id SERIAL PRIMARY KEY,
                name TEXT NOT NULL UNIQUE );",
            &[],
        )?;
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS ticker (
                id SERIAL PRIMARY KEY,
                name TEXT NOT NULL,
                source_id INTEGER NOT NULL,
                currency TEXT NOT NULL,
                FOREIGN KEY(source_id) REFERENCES market_data_sources(id) );",
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

        Ok(())
    }
}
