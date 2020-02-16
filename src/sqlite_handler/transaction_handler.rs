///! Implementation of sqlite3 data handler
use crate::data_handler::{DataError, TransactionHandler};
use crate::transaction::Transaction;
use rusqlite::{params, NO_PARAMS};

use super::raw_transaction::RawTransaction;
use super::SqliteDB;

/// Handler for globally available data
impl TransactionHandler for SqliteDB {
    // insert, get, update and delete for transactions
    fn insert_transaction(&mut self, transaction: &Transaction) -> Result<usize, DataError> {
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
        let id = self
            .conn
            .query_row("SELECT last_insert_rowid();", NO_PARAMS, |row| {
                let id: i64 = row.get(0)?;
                Ok(id as usize)
            })
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok(id)
    }

    fn get_transaction_by_id(&mut self, id: usize) -> Result<Transaction, DataError> {
        let transaction = self
            .conn
            .query_row(
                "SELECT trans_type, asset_id, 
        cash_amount, cash_currency, cash_date, related_trans, position, note 
        FROM transactions
        WHERE id=?;",
                params![id as i64],
                |row| {
                    Ok(RawTransaction {
                        id: Some(id as i64),
                        trans_type: row.get(0)?,
                        asset: row.get(1)?,
                        cash_amount: row.get(2)?,
                        cash_currency: row.get(3)?,
                        cash_date: row.get(4)?,
                        related_trans: row.get(5)?,
                        position: row.get(6)?,
                        note: row.get(7)?,
                    })
                },
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let transaction = transaction.to_transaction()?;
        Ok(transaction)
    }

    fn get_all_transactions(&mut self) -> Result<Vec<Transaction>, DataError> {
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

    fn update_transaction(&mut self, transaction: &Transaction) -> Result<(), DataError> {
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
    fn delete_transaction(&mut self, id: usize) -> Result<(), DataError> {
        self.conn
            .execute("DELETE FROM transactions WHERE id=?1;", params![id as i64])
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }
}
