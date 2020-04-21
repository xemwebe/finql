///! Implemenation of sqlite3 data handler
use rusqlite::{Connection, OpenFlags, NO_PARAMS};

mod asset_handler;
mod quote_handler;
mod raw_transaction;
mod transaction_handler;

/// Struct to handle connections to sqlite3 databases
pub struct SqliteDB {
    /// conn is made public to allow extending this struct outside of the library
    pub conn: Connection,
}

impl SqliteDB {
    pub fn connect(file_path: &str) -> rusqlite::Result<SqliteDB> {
        let conn = Connection::open_with_flags(file_path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        Ok(SqliteDB { conn })
    }

    pub fn create(file_path: &str) -> rusqlite::Result<SqliteDB> {
        let conn = Connection::open(file_path)?;
        let db = SqliteDB { conn };
        db.init()?;
        Ok(db)
    }

    /// Initialize new database by creating table, fill
    fn init(&self) -> rusqlite::Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS assets (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                wkn TEXT UNIQUE,
                isin TEXT UNIQUE,
                note TEXT
            )",
            NO_PARAMS,
        )?;
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER PRIMARY KEY,
                trans_type TEXT NOT NULL,
                asset_id INTEGER,
                cash_amount REAL NOT NULL,
                cash_currency TXT NOT NULL,
                cash_date TEXT NOT NULL,
                related_trans KEY,
                position REAL,
                note TEXT,
                FOREIGN KEY(asset_id) REFERENCES assets(id),
                FOREIGN KEY(related_trans) REFERENCES transactions(id)
            );",
            NO_PARAMS,
        )?;
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS ticker (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                asset_id INTEGER NOT NULL,
                source TEXT NOT NULL,
                priority INTEGER NOT NULL,
                currency TEXT NOT NULL,
                FOREIGN KEY(asset_id) REFERENCES assets(id) 
            );",
            NO_PARAMS,
        )?;
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS quotes (
                id INTEGER PRIMARY KEY,
                ticker_id INTEGER NOT NULL,
                price REAL NOT NULL,
                time TEXT NOT NULL,
                volume REAL,
                FOREIGN KEY(ticker_id) REFERENCES ticker(id) );",
            NO_PARAMS,
        )?;
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS rounding_digits (
                id INTEGER PRIMARY KEY,
                currency TEXT NOT NULL UNIQUE,
                digits INTEGER NOT NULL);",
            NO_PARAMS,
        )?;
        Ok(())
    }
}
