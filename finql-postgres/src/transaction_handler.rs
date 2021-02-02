use std::str::FromStr;
use chrono::NaiveDate;
use async_trait::async_trait;

use finql_data::currency::Currency;
use finql_data::{DataError, TransactionHandler};
use finql_data::cash_flow::{CashAmount, CashFlow};
use finql_data::transaction::{Transaction, TransactionType};

use super::PostgresDB;


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
    pub async fn to_transaction(&self) -> Result<Transaction, DataError> {
        let currency = Currency::from_str(&self.cash_currency)
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        let id = self.id.map(|x| x as usize);
        let cash_flow = CashFlow {
            amount: CashAmount {
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
                transaction_ref: self.related_trans.map(|x| x as usize),
            },
            FEE => TransactionType::Fee {
                transaction_ref: self.related_trans.map(|x| x as usize),
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
        let id = transaction.id.map(|x| x as i32);
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
                raw_transaction.related_trans = transaction_ref.map(|x| x as i32);
            }
            TransactionType::Fee { transaction_ref } => {
                raw_transaction.trans_type = FEE.to_string();
                raw_transaction.related_trans = transaction_ref.map(|x| x as i32);
            }
        };
        raw_transaction
    }
}

/// Handler for globally available data
#[async_trait]
impl TransactionHandler for PostgresDB {
    // insert, get, update and delete for transactions
    async fn insert_transaction(&mut self, transaction: &Transaction) -> Result<usize, DataError> {
        let transaction = RawTransaction::from_transaction(transaction);
        let row = sqlx::query!(
                "INSERT INTO transactions (trans_type, asset_id, cash_amount, 
                cash_currency, cash_date, related_trans, position,
                note) 
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
                transaction.trans_type,
                transaction.asset,
                transaction.cash_amount,
                transaction.cash_currency,
                transaction.cash_date,
                transaction.related_trans,
                transaction.position,
                transaction.note,
            ).fetch_one(&self.pool).await
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        let id= row.id;
        Ok(id as usize)
    }

    async fn get_transaction_by_id(&mut self, id: usize) -> Result<Transaction, DataError> {
        let row = sqlx::query!(
                "SELECT trans_type, asset_id, 
        cash_amount, cash_currency, cash_date, related_trans, position, note 
        FROM transactions
        WHERE id=$1", (id as i32),
            ).fetch_one(&self.pool).await
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let transaction = RawTransaction {
            id: Some(id as i32),
            trans_type: row.trans_type,
            asset: row.asset_id,
            cash_amount: row.cash_amount,
            cash_currency: row.cash_currency,
            cash_date: row.cash_date,
            related_trans: row.related_trans,
            position: row.position,
            note: row.note,
        };
        Ok(transaction.to_transaction().await?)
    }

    async fn get_all_transactions(&mut self) -> Result<Vec<Transaction>, DataError> {
        let mut transactions = Vec::new();
        for row in sqlx::query!(
                "SELECT id, trans_type, asset_id, 
        cash_amount, cash_currency, cash_date, related_trans, position, note 
        FROM transactions",
            ).fetch_all(&self.pool).await
            .map_err(|e| DataError::NotFound(e.to_string()))?
        {
            let transaction = RawTransaction {
                id: Some(row.id),
                trans_type: row.trans_type,
                asset: row.asset_id,
                cash_amount: row.cash_amount,
                cash_currency: row.cash_currency,
                cash_date: row.cash_date,
                related_trans: row.related_trans,
                position: row.position,
                note: row.note,
            };
            transactions.push(transaction.to_transaction().await?);
        }
        Ok(transactions)
    }

    async fn update_transaction(&mut self, transaction: &Transaction) -> Result<(), DataError> {
        if transaction.id.is_none() {
            return Err(DataError::NotFound(
                "not yet stored to database".to_string(),
            ));
        }
        let id = transaction.id.unwrap() as i32;
        let transaction = RawTransaction::from_transaction(transaction);
        sqlx::query!(
                "UPDATE transactions SET 
                trans_type=$2, 
                asset_id=$3, 
                cash_amount=$4, 
                cash_currency=$5,
                cash_date=$6,
                related_trans=$7,
                position=$8,
                note=$9
            WHERE id=$1",
                id,
                transaction.trans_type,
                transaction.asset,
                transaction.cash_amount,
                transaction.cash_currency,
                transaction.cash_date,
                transaction.related_trans,
                transaction.position,
                transaction.note,
            ).execute(&self.pool).await
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    async fn delete_transaction(&mut self, id: usize) -> Result<(), DataError> {
        sqlx::query!("DELETE FROM transactions WHERE id=$1;", (id as i32))
            .execute(&self.pool).await
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }
}
