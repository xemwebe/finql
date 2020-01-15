///! Implemenation of sqlite3 data handler
use crate::asset::Asset;
use crate::data_handler::{DataError, DataHandler};
use crate::transaction::{Transaction, TransactionType};
use rusqlite::{params, Connection, OpenFlags, NO_PARAMS};
use std::convert::TryFrom;

mod helpers;

/// Struct to handle connections to sqlite3 databases
pub struct SqliteDB {
    conn: Connection,
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
                trans_type INTEGER NOT NULL,
                asset_id INTEGER NOT NULL,
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
        Ok(())
    }
}

struct RawTransaction {
    id: Option<i64>,
    trans_type: u8,
    asset: Option<i64>,
    cash_amount: f64,
    cash_currency: String,
    cash_date: String,
    related_trans: Option<i64>,
    position: Option<f64>,
    note: Option<String>,
}

impl RawTransaction {
    fn to_transaction(&self) -> Result<Transaction, DataError> {
        Ok(Transaction {
            id: helpers::int_to_usize(self.id),
            trans_type: TransactionType::try_from(self.trans_type)
                .map_err(|e| DataError::NotFound(e.to_string()))?,
            asset: helpers::int_to_usize(self.asset),
            cash_flow: helpers::raw_to_cash_flow(
                self.cash_amount,
                &self.cash_currency,
                &self.cash_date,
            )?,
            related_trans: helpers::int_to_usize(self.related_trans),
            position: self.position,
            note: self.note.clone(),
        })
    }

    fn from_transaction(transaction: &Transaction) -> RawTransaction {
        RawTransaction {
            id: helpers::usize_to_int(transaction.id),
            trans_type: transaction.trans_type as u8,
            asset: helpers::usize_to_int(transaction.asset),
            cash_amount: transaction.cash_flow.amount.amount,
            cash_currency: transaction.cash_flow.amount.currency.to_string(),
            cash_date: transaction.cash_flow.date.format("%Y-%m-%d").to_string(),
            related_trans: helpers::usize_to_int(transaction.related_trans),
            position: transaction.position,
            note: transaction.note.clone(),
        }
    }
}

/// Handler for globally available data
impl DataHandler for SqliteDB {
    fn insert_asset(&self, asset: &Asset) -> Result<usize, DataError> {
        self.conn
            .execute(
                "INSERT INTO assets (name, wkn, isin, note) VALUES (?1, ?2, ?3, ?4)",
                params![asset.name, asset.wkn, asset.isin, asset.note],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        let id = self
            .conn
            .query_row(
                "SELECT id FROM assets
        WHERE name=?;",
                params![asset.name],
                |row| {
                    let id: i64 = row.get(0)?;
                    Ok(id as usize)
                },
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok(id)
    }

    fn get_asset_by_id(&self, id: usize) -> Result<Asset, DataError> {
        let asset = self
            .conn
            .query_row(
                "SELECT id, name, wkn, isin, note FROM assets
        WHERE id=?;",
                &[id as i64],
                |row| {
                    let id: i64 = row.get(0)?;
                    let id = Some(id as usize);
                    Ok(Asset {
                        id,
                        name: row.get(1)?,
                        wkn: row.get(2)?,
                        isin: row.get(3)?,
                        note: row.get(4)?,
                    })
                },
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok(asset)
    }

    fn get_all_assets(&self) -> Result<Vec<Asset>, DataError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, wkn, isin, note FROM assets;")
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let asset_map = stmt
            .query_map(NO_PARAMS, |row| {
                let id: i64 = row.get(0)?;
                let id = Some(id as usize);
                Ok(Asset {
                    id,
                    name: row.get(1)?,
                    wkn: row.get(2)?,
                    isin: row.get(3)?,
                    note: row.get(4)?,
                })
            })
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let mut assets = Vec::new();
        for asset in asset_map {
            assets.push(asset.unwrap());
        }
        Ok(assets)
    }

    fn update_asset(&self, asset: &Asset) -> Result<(), DataError> {
        if asset.id.is_none() {
            return Err(DataError::NotFound(
                "not yet stored to database".to_string(),
            ));
        }
        let id = asset.id.unwrap() as i64;
        self.conn
            .execute(
                "UPDATE assets SET name=?2, wkn=?3, isin=?4, note=?5 
                WHERE id=?1;",
                params![id, asset.name, asset.wkn, asset.isin, asset.note],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    fn delete_asset(&self, id: usize) -> Result<(), DataError> {
        self.conn
            .execute("DELETE FROM assets WHERE id=?1;", params![id as i64])
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    // insert, get, update and delete for transactions
    fn insert_transaction(&self, transaction: &Transaction) -> Result<(), DataError> {
        let transaction = RawTransaction::from_transaction(transaction);
        self.conn
            .execute(
                "INSERT INTO transactions (trans_type, asset_id, cash_amount, 
                cash_currency, cash_date, related_trans, position,
                note) 
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);",
                params![
                    transaction.trans_type,
                    transaction.asset,
                    transaction.cash_amount,
                    transaction.cash_currency,
                    transaction.cash_date,
                    transaction.related_trans,
                    transaction.position,
                    transaction.note
                ],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    fn get_transaction_by_id(&self, id: usize) -> Result<Transaction, DataError> {
        let transaction = self
            .conn
            .query_row(
                "SELECT id, trans_type, asset_id, 
        cash_amount, cash_currency, cash_date, related_trans, position, note 
        FROM transactions
        WHERE id=?;",
                params![id as i64],
                |row| {
                    Ok(RawTransaction {
                        id: row.get(0)?,
                        trans_type: row.get(1)?,
                        asset: row.get(2)?,
                        cash_amount: row.get(3)?,
                        cash_currency: row.get(4)?,
                        cash_date: row.get(5)?,
                        related_trans: row.get(6)?,
                        position: row.get(7)?,
                        note: row.get(8)?,
                    })
                },
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let transaction = transaction.to_transaction()?;
        Ok(transaction)
    }

    fn get_all_transactions(&self) -> Result<Vec<Transaction>, DataError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, trans_type, asset_id, 
        cash_amount, cash_currency, cash_date, related_trans, position, note 
        FROM transactions;",
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let transaction_map = stmt
            .query_map(NO_PARAMS, |row| {
                Ok(RawTransaction {
                    id: row.get(0)?,
                    trans_type: row.get(1)?,
                    asset: row.get(2)?,
                    cash_amount: row.get(3)?,
                    cash_currency: row.get(4)?,
                    cash_date: row.get(5)?,
                    related_trans: row.get(6)?,
                    position: row.get(7)?,
                    note: row.get(8)?,
                })
            })
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let mut transactions = Vec::new();
        for transaction in transaction_map {
            transactions.push(transaction.unwrap().to_transaction()?);
        }
        Ok(transactions)
    }

    fn update_transaction(&self, transaction: &Transaction) -> Result<(), DataError> {
        if transaction.id.is_none() {
            return Err(DataError::NotFound(
                "not yet stored to database".to_string(),
            ));
        }
        let id = transaction.id.unwrap() as i64;
        let transaction = RawTransaction::from_transaction(transaction);
        self.conn
            .execute(
                "UPDATE transactions SET 
                trans_type=?2, 
                asset_id=?3, 
                cash_value=?4, 
                cash_currency=?5,
                cash_date=?6,
                related_trans=?7,
                position=?8,
                note=?9
            WHERE id=?1;",
                params![
                    id,
                    transaction.trans_type,
                    transaction.asset,
                    transaction.cash_amount,
                    transaction.cash_currency,
                    transaction.cash_date,
                    transaction.related_trans,
                    transaction.position,
                    transaction.note
                ],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }
    fn delete_transaction(&self, id: usize) -> Result<(), DataError> {
        self.conn
            .execute("DELETE FROM transactions WHERE id=?1;", params![id as i64])
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }
}
