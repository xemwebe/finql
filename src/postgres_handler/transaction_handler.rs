use super::PostgresDB;
use crate::asset::Asset;
use crate::currency::Currency;
use crate::data_handler::{DataError, DataHandler};
use crate::fixed_income::{Amount, CashFlow};
use crate::helpers::{i32_to_usize, usize_to_i32};
use crate::transaction::{Transaction, TransactionType};
use chrono::NaiveDate;
use std::str::FromStr;

pub struct RawTransaction {
    pub id: Option<i32>,
    pub trans_type: String,
    pub asset: Option<i32>,
    pub cash_amount: f64,
    pub cash_currency: String,
    pub cash_date: NaiveDate,
    pub related_trans: Option<i32>,
    pub position: Option<f64>,
    pub note: Option<String>,
}

/// Raw transaction type constants
const CASH: &str = "c";
const ASSET: &str = "a";
const DIVIDEND: &str = "d";
const INTEREST: &str = "i";
const TAX: &str = "t";
const FEE: &str = "f";

impl RawTransaction {
    pub fn to_transaction(&self) -> Result<Transaction, DataError> {
        let currency = Currency::from_str(&self.cash_currency)
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        let id = i32_to_usize(self.id);
        let cash_flow = CashFlow {
            amount: Amount {
                amount: self.cash_amount,
                currency,
            },
            date: self.cash_date,
        };
        let note = self.note.clone();
        let transaction_type = match self.trans_type.as_str() {
            CASH => TransactionType::Cash,
            ASSET => TransactionType::Asset {
                asset_id: self.asset.ok_or(DataError::InvalidTransaction(
                    "missing asset id".to_string(),
                ))? as usize,
                position: self.position.ok_or(DataError::InvalidTransaction(
                    "missing position value".to_string(),
                ))?,
            },
            DIVIDEND => TransactionType::Dividend {
                asset_id: self.asset.ok_or(DataError::InvalidTransaction(
                    "missing asset id".to_string(),
                ))? as usize,
            },
            INTEREST => TransactionType::Interest {
                asset_id: self.asset.ok_or(DataError::InvalidTransaction(
                    "missing asset id".to_string(),
                ))? as usize,
            },
            TAX => TransactionType::Tax {
                transaction_ref: i32_to_usize(self.id),
            },
            FEE => TransactionType::Fee {
                transaction_ref: i32_to_usize(self.id),
            },
            unknown => {
                return Err(DataError::InvalidTransaction(unknown.to_string()));
            }
        };
        Ok(Transaction {
            id,
            transaction_type,
            cash_flow,
            note,
        })
    }

    pub fn from_transaction(transaction: &Transaction) -> RawTransaction {
        let id = usize_to_i32(transaction.id);
        let cash_amount = transaction.cash_flow.amount.amount;
        let cash_currency = transaction.cash_flow.amount.currency.to_string();
        let note = transaction.note.clone();
        let mut raw_transaction = RawTransaction {
            id,
            trans_type: String::new(),
            asset: None,
            cash_amount,
            cash_currency,
            cash_date: transaction.cash_flow.date,
            related_trans: None,
            position: None,
            note,
        };
        match transaction.transaction_type {
            TransactionType::Cash => raw_transaction.trans_type = CASH.to_string(),
            TransactionType::Asset { asset_id, position } => {
                raw_transaction.trans_type = ASSET.to_string();
                raw_transaction.asset = Some(asset_id as i32);
                raw_transaction.position = Some(position);
            }
            TransactionType::Dividend { asset_id } => {
                raw_transaction.trans_type = DIVIDEND.to_string();
                raw_transaction.asset = Some(asset_id as i32);
            }
            TransactionType::Interest { asset_id } => {
                raw_transaction.trans_type = INTEREST.to_string();
                raw_transaction.asset = Some(asset_id as i32);
            }
            TransactionType::Tax { transaction_ref } => {
                raw_transaction.trans_type = TAX.to_string();
                raw_transaction.related_trans = usize_to_i32(transaction_ref);
            }
            TransactionType::Fee { transaction_ref } => {
                raw_transaction.trans_type = FEE.to_string();
                raw_transaction.related_trans = usize_to_i32(transaction_ref);
            }
        };
        raw_transaction
    }
}

/// Handler for globally available data
impl DataHandler for PostgresDB {
    fn insert_asset(&mut self, asset: &Asset) -> Result<usize, DataError> {
        let row = self
            .conn
            .query_one(
                "INSERT INTO assets (name, wkn, isin, note) VALUES ($1, $2, $3, $4) RETURNING id",
                &[&asset.name, &asset.wkn, &asset.isin, &asset.note],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        let id: i32 = row.get(0);
        Ok(id as usize)
    }

    fn get_asset_by_id(&mut self, id: usize) -> Result<Asset, DataError> {
        let row = self
            .conn
            .query_one(
                "SELECT name, wkn, isin, note FROM assets WHERE id=$1",
                &[&(id as i32)],
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok(Asset {
            id: Some(id),
            name: row.get(0),
            wkn: row.get(1),
            isin: row.get(2),
            note: row.get(3),
        })
    }

    fn get_all_assets(&mut self) -> Result<Vec<Asset>, DataError> {
        let mut assets = Vec::new();
        for row in self
            .conn
            .query("SELECT id, name, wkn, isin, note FROM assets", &[])
            .map_err(|e| DataError::NotFound(e.to_string()))?
        {
            let id: i32 = row.get(0);
            let id = Some(id as usize);
            assets.push(Asset {
                id,
                name: row.get(1),
                wkn: row.get(2),
                isin: row.get(3),
                note: row.get(4),
            });
        }
        Ok(assets)
    }

    fn update_asset(&mut self, asset: &Asset) -> Result<(), DataError> {
        if asset.id.is_none() {
            return Err(DataError::NotFound(
                "not yet stored to database".to_string(),
            ));
        }
        let id = asset.id.unwrap() as i32;
        self.conn
            .execute(
                "UPDATE assets SET name=$2, wkn=$3, isin=$4, note=$5 
                WHERE id=$1;",
                &[&id, &asset.name, &asset.wkn, &asset.isin, &asset.note],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    fn delete_asset(&mut self, id: usize) -> Result<(), DataError> {
        self.conn
            .execute("DELETE FROM assets WHERE id=$1;", &[&(id as i32)])
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    // insert, get, update and delete for transactions
    fn insert_transaction(&mut self, transaction: &Transaction) -> Result<usize, DataError> {
        let transaction = RawTransaction::from_transaction(transaction);
        let row = self
            .conn
            .query_one(
                "INSERT INTO transactions (trans_type, asset_id, cash_amount, 
                cash_currency, cash_date, related_trans, position,
                note) 
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
                &[
                    &transaction.trans_type,
                    &transaction.asset,
                    &transaction.cash_amount,
                    &transaction.cash_currency,
                    &transaction.cash_date,
                    &transaction.related_trans,
                    &transaction.position,
                    &transaction.note,
                ],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        let id: i32 = row.get(0);
        Ok(id as usize)
    }

    fn get_transaction_by_id(&mut self, id: usize) -> Result<Transaction, DataError> {
        let row = self
            .conn
            .query_one(
                "SELECT trans_type, asset_id, 
        cash_amount, cash_currency, cash_date, related_trans, position, note 
        FROM transactions
        WHERE id=$1",
                &[&(id as i32)],
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let transaction = RawTransaction {
            id: Some(id as i32),
            trans_type: row.get(0),
            asset: row.get(1),
            cash_amount: row.get(2),
            cash_currency: row.get(3),
            cash_date: row.get(4),
            related_trans: row.get(5),
            position: row.get(6),
            note: row.get(7),
        };
        Ok(transaction.to_transaction()?)
    }

    fn get_all_transactions(&mut self) -> Result<Vec<Transaction>, DataError> {
        let mut transactions = Vec::new();
        for row in self
            .conn
            .query(
                "SELECT id, trans_type, asset_id, 
        cash_amount, cash_currency, cash_date, related_trans, position, note 
        FROM transactions",
                &[],
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?
        {
            let transaction = RawTransaction {
                id: row.get(0),
                trans_type: row.get(1),
                asset: row.get(2),
                cash_amount: row.get(3),
                cash_currency: row.get(4),
                cash_date: row.get(5),
                related_trans: row.get(6),
                position: row.get(7),
                note: row.get(8),
            };
            transactions.push(transaction.to_transaction()?);
        }
        Ok(transactions)
    }

    fn update_transaction(&mut self, transaction: &Transaction) -> Result<(), DataError> {
        if transaction.id.is_none() {
            return Err(DataError::NotFound(
                "not yet stored to database".to_string(),
            ));
        }
        let id = transaction.id.unwrap() as i32;
        let transaction = RawTransaction::from_transaction(transaction);
        self.conn
            .execute(
                "UPDATE transactions SET 
                trans_type=$2, 
                asset_id=$3, 
                cash_value=$4, 
                cash_currency=$5,
                cash_date=$6,
                related_trans=$7,
                position=$8,
                note=$9
            WHERE id=$1",
                &[
                    &id,
                    &transaction.trans_type,
                    &transaction.asset,
                    &transaction.cash_amount,
                    &transaction.cash_currency,
                    &transaction.cash_date,
                    &transaction.related_trans,
                    &transaction.position,
                    &transaction.note,
                ],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    fn delete_transaction(&mut self, id: usize) -> Result<(), DataError> {
        self.conn
            .execute("DELETE FROM transactions WHERE id=$1;", &[&(id as i32)])
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }
}
