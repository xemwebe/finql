use async_trait::async_trait;
use std::str::FromStr;
use time::Date;

use crate::datatypes::cash_flow::{CashAmount, CashFlow};
use crate::datatypes::currency::Currency;
use crate::datatypes::transaction::{Transaction, TransactionType};
use crate::datatypes::{CurrencyISOCode, DataError, TransactionHandler};

use super::PostgresDB;

pub struct RawTransaction {
    pub id: Option<i32>,
    pub trans_type: String,
    pub asset: Option<i32>,
    pub cash_amount: f64,
    pub cash_currency: Currency,
    pub cash_date: Date,
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
        let currency = self.cash_currency;

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
                asset_id: self
                    .asset
                    .ok_or_else(|| DataError::InvalidTransaction("missing asset id".to_string()))?,
                position: self.position.ok_or_else(|| {
                    DataError::InvalidTransaction("missing position value".to_string())
                })?,
            },
            DIVIDEND => TransactionType::Dividend {
                asset_id: self
                    .asset
                    .ok_or_else(|| DataError::InvalidTransaction("missing asset id".to_string()))?,
            },
            INTEREST => TransactionType::Interest {
                asset_id: self
                    .asset
                    .ok_or_else(|| DataError::InvalidTransaction("missing asset id".to_string()))?,
            },
            TAX => TransactionType::Tax {
                transaction_ref: self.related_trans,
            },
            FEE => TransactionType::Fee {
                transaction_ref: self.related_trans,
            },
            unknown => {
                return Err(DataError::InvalidTransaction(unknown.to_string()));
            }
        };
        Ok(Transaction {
            id: self.id,
            transaction_type,
            cash_flow,
            note,
        })
    }

    pub fn from_transaction(transaction: &Transaction) -> RawTransaction {
        let cash_amount = transaction.cash_flow.amount.amount;
        let cash_currency = transaction.cash_flow.amount.currency;
        let note = transaction.note.clone();
        let mut raw_transaction = RawTransaction {
            id: transaction.id,
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
                raw_transaction.asset = Some(asset_id);
                raw_transaction.position = Some(position);
            }
            TransactionType::Dividend { asset_id } => {
                raw_transaction.trans_type = DIVIDEND.to_string();
                raw_transaction.asset = Some(asset_id);
            }
            TransactionType::Interest { asset_id } => {
                raw_transaction.trans_type = INTEREST.to_string();
                raw_transaction.asset = Some(asset_id);
            }
            TransactionType::Tax { transaction_ref } => {
                raw_transaction.trans_type = TAX.to_string();
                raw_transaction.related_trans = transaction_ref;
            }
            TransactionType::Fee { transaction_ref } => {
                raw_transaction.trans_type = FEE.to_string();
                raw_transaction.related_trans = transaction_ref;
            }
        };
        raw_transaction
    }
}

/// Handler for globally available data
#[async_trait]
impl TransactionHandler for PostgresDB {
    // insert, get, update and delete for transactions
    async fn insert_transaction(&self, transaction: &Transaction) -> Result<i32, DataError> {
        let transaction = RawTransaction::from_transaction(transaction);
        let row = sqlx::query!(
            "INSERT INTO transactions (trans_type, asset_id, cash_amount,
                cash_currency_id, cash_date, related_trans, position,
                note)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
            transaction.trans_type,
            transaction.asset,
            transaction.cash_amount,
            transaction.cash_currency.id,
            transaction.cash_date,
            transaction.related_trans,
            transaction.position,
            transaction.note,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row.id)
    }

    async fn get_transaction_by_id(&self, id: i32) -> Result<Transaction, DataError> {
        let row = sqlx::query!(
            "SELECT
                t.id,
                t.trans_type,
                t.asset_id,
                t.cash_amount,
                c.id AS cash_currency_id,
                c.iso_code AS cash_iso_code,
                c.rounding_digits AS cash_rounding_digits,
                t.cash_date,
                t.related_trans,
                t.position,
                t.note
                FROM transactions t
                JOIN currencies c ON c.id = t.cash_currency_id
                WHERE t.id = $1",
            id,
        )
        .fetch_one(&self.pool)
        .await?;
        let transaction = RawTransaction {
            id: Some(id),
            trans_type: row.trans_type,
            asset: row.asset_id,
            cash_amount: row.cash_amount,
            cash_currency: Currency::new(
                Some(row.cash_currency_id),
                CurrencyISOCode::from_str(&row.cash_iso_code)
                    .expect("Expected a good currency code from db"),
                Some(row.cash_rounding_digits),
            ),
            cash_date: row.cash_date,
            related_trans: row.related_trans,
            position: row.position,
            note: row.note,
        };
        Ok(transaction.to_transaction()?)
    }

    async fn get_all_transactions(&self) -> Result<Vec<Transaction>, DataError> {
        let mut transactions = Vec::new();
        for row in sqlx::query!(
            r#"SELECT
                t.id AS "id!",
                t.trans_type AS "trans_type!",
                t.asset_id,
                t.cash_amount AS "cash_amount!",
                c.id AS "cash_currency_id!",
                c.iso_code AS "cash_iso_code!",
                c.rounding_digits AS "cash_rounding_digits!",
                t.cash_date AS "cash_date!",
                t.related_trans,
                t.position,
                t.note
                FROM transactions t
                JOIN currencies c ON c.id = t.cash_currency_id"#
        )
        .fetch_all(&self.pool)
        .await?
        {
            let transaction = RawTransaction {
                id: Some(row.id),
                trans_type: row.trans_type,
                asset: row.asset_id,
                cash_amount: row.cash_amount,
                cash_currency: Currency::new(
                    Some(row.cash_currency_id),
                    CurrencyISOCode::from_str(&row.cash_iso_code)
                        .expect("unknown currency asset referenced in db"),
                    Some(row.cash_rounding_digits),
                ),
                cash_date: row.cash_date,
                related_trans: row.related_trans,
                position: row.position,
                note: row.note,
            };
            transactions.push(transaction.to_transaction()?);
        }
        Ok(transactions)
    }

    async fn update_transaction(&self, transaction: &Transaction) -> Result<(), DataError> {
        if transaction.id.is_none() {
            return Err(DataError::NotFound(
                "not yet stored to database".to_string(),
            ));
        }
        let transaction = RawTransaction::from_transaction(transaction);
        sqlx::query!(
            "UPDATE transactions SET
                trans_type=$2,
                asset_id=$3,
                cash_amount=$4,
                cash_currency_id=$5,
                cash_date=$6,
                related_trans=$7,
                position=$8,
                note=$9
            WHERE id=$1",
            transaction.id,
            transaction.trans_type,
            transaction.asset,
            transaction.cash_amount,
            transaction.cash_currency.id,
            transaction.cash_date,
            transaction.related_trans,
            transaction.position,
            transaction.note,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete_transaction(&self, id: i32) -> Result<(), DataError> {
        sqlx::query!("DELETE FROM transactions WHERE id=$1;", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
