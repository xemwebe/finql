use std::str::FromStr;
use chrono::NaiveDate;
use async_trait::async_trait;

use finql_data::currency::{Currency, CurrencyISOCode};
use finql_data::{DataError, TransactionHandler};
use finql_data::cash_flow::{CashAmount, CashFlow};
use finql_data::transaction::{Transaction, TransactionType};

use super::{SqliteDB, SQLiteError};
use deadpool_sqlite::rusqlite::params;


#[derive(Clone, Debug)]
pub struct RawTransaction {
    pub id: Option<usize>,
    pub trans_type: String,
    pub asset: Option<usize>,
    pub cash_amount: f64,
    pub cash_currency: Currency,
    pub cash_date: NaiveDate,
    pub related_trans: Option<usize>,
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
        let id = self.id.map(|x| x as usize);
        let cash_flow = CashFlow {
            amount: CashAmount {
                amount: self.cash_amount,
                currency: self.cash_currency,
            },
            date: self.cash_date,
        };
        let note = self.note.clone();
        let transaction_type = match self.trans_type.as_str() {
            CASH => TransactionType::Cash,
            ASSET => TransactionType::Asset {
                asset_id: self.asset.ok_or_else(|| DataError::InvalidTransaction(
                    "missing asset id".to_string()
                ))? as usize,
                position: self.position.ok_or_else(|| DataError::InvalidTransaction(
                    "missing position value".to_string(),
                ))?,
            },
            DIVIDEND => TransactionType::Dividend {
                asset_id: self.asset.ok_or_else(|| DataError::InvalidTransaction(
                    "missing asset id".to_string(),
                ))? as usize,
            },
            INTEREST => TransactionType::Interest {
                asset_id: self.asset.ok_or_else(|| DataError::InvalidTransaction(
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
        let id = transaction.id;
        let cash_amount = transaction.cash_flow.amount.amount;
        let cash_currency = transaction.cash_flow.amount.currency;
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
impl TransactionHandler for SqliteDB {
    // insert, get, update and delete for transactions
    async fn insert_transaction(&self, transaction: &Transaction) -> Result<usize, DataError> {
        let transaction = RawTransaction::from_transaction(transaction);
        let time_stamp = chrono::offset::Utc::now().timestamp_nanos();
        let transaction2 = transaction.clone();

        self.conn.interact(move |conn| -> Result<usize, SQLiteError> {
            let r = conn.execute(
                "INSERT INTO transactions (trans_type, asset_id, cash_amount, cash_currency_id, cash_date, related_trans, position,
                    note, time_stamp)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    &transaction.trans_type,
                    &transaction.asset,
                    &transaction.cash_amount,
                    &transaction.cash_currency.id.expect("cash currency asset id must be in db"),
                    &transaction.cash_date,
                    &transaction.related_trans,
                    &transaction.position,
                    &transaction.note,
                    &time_stamp
                ])?;

            let mut sql = "SELECT id FROM transactions WHERE trans_type = ?1".to_string();
            if transaction.asset.is_some() {
                sql.push_str(" AND asset_id = ?2");
            }
            else {
                sql.push_str(" AND asset_id IS NULL");
            }
            sql.push_str(" AND cash_amount = ?3 AND cash_currency_id = ?4 AND cash_date = ?5");

            let id = conn.query_row(
                &sql,
                params![
                    &transaction.trans_type,
                    &transaction.asset,
                    &transaction.cash_amount,
                    &transaction.cash_currency.id.expect("cash currency asset id must be in db"),
                    &transaction.cash_date,
                ],
                |row| {
                    let id:usize = row.get(0)?;
                    Ok(id)
                })?;

            Ok(id)
        }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))
    }

    async fn get_transaction_by_id(&self, id: usize) -> Result<Transaction, DataError> {
        Ok(self.conn.interact(move |conn| -> Result<RawTransaction, SQLiteError> {
            Ok(conn.query_row(
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
                WHERE t.id = ?1",
                params![&id],
                |row| {
                    let cic:String = row.get(5)?;

                    Ok(RawTransaction {
                        id: row.get(0)?,
                        trans_type: row.get(1)?,
                        asset: row.get(2)?,
                        cash_amount: row.get(3)?,
                        cash_currency: Currency::new(
                            row.get(4)?,
                            CurrencyISOCode::from_str(&cic).expect("expected good code from the database"),
                            row.get(6)?,
                        ),
                        cash_date: row.get(7)?,
                        related_trans: row.get(8)?,
                        position: row.get(9)?,
                        note: row.get(10)?
                    })
                })?)
        }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .to_transaction()?)
    }

    async fn get_all_transactions(&self) -> Result<Vec<Transaction>, DataError> {
        self.conn.interact(|conn| -> Result<Vec<Transaction>, SQLiteError> {
            let mut stmt = conn.prepare("
                SELECT
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
                JOIN currencies c ON c.id = t.cash_currency_id")?;

            let assets: Vec<Transaction> = stmt.query_map([], |row| {
                let cic:String = row.get(5)?;

                Ok(RawTransaction {
                    id: row.get(0)?,
                    trans_type: row.get(1)?,
                    asset: row.get(2)?,
                    cash_amount: row.get(3)?,
                    cash_currency: Currency::new(
                        row.get(4)?,
                        CurrencyISOCode::from_str(&cic).expect("expected good code from the database"),
                        row.get(6)?,
                    ),
                    cash_date: row.get(7)?,
                    related_trans: row.get(8)?,
                    position: row.get(9)?,
                    note: row.get(10)?
                })
            })?.filter_map(|raw_transaction| raw_transaction.ok())
            .map(|raw_transaction| raw_transaction.to_transaction())
            .filter_map(|transaction| transaction.ok()).collect();
            Ok(assets)
        })
        .await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))
    }

    async fn update_transaction(&self, transaction: &Transaction) -> Result<(), DataError> {
        if transaction.id.is_none() {
            return Err(DataError::NotFound(
                "not yet stored to database".to_string(),
            ));
        }

        let transaction = RawTransaction::from_transaction(transaction);

        if transaction.cash_currency.id.is_none() {
            return Err(DataError::NotFound(
                "transaction currency not yet stored to database".to_string(),
            ));
        }

        let time_stamp = chrono::offset::Utc::now().timestamp_nanos();
        self.conn.interact(move |conn| -> Result<(), SQLiteError> {
            conn.execute(
                "UPDATE transactions SET 
                trans_type = ?2,
                asset_id = ?3,
                cash_amount = ?4,
                cash_currency_id = ?5,
                cash_date = ?6,
                related_trans = ?7,
                position = ?8,
                note = ?9,
                time_stamp = ?10
                WHERE id = ?1",
                params![
                    &transaction.id,                
                    &transaction.trans_type,
                    &transaction.asset,
                    &transaction.cash_amount,
                    &transaction.cash_currency.id.unwrap(),
                    &transaction.cash_date,
                    &transaction.related_trans,
                    &transaction.position,
                    &transaction.note,
                    &time_stamp])?;
            Ok(())
        }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))
    }

    async fn delete_transaction(&self, id: usize) -> Result<(), DataError> {
        self.conn.interact(move |conn| -> Result<(), SQLiteError> {
            conn.execute("DELETE FROM transactions WHERE id = ?;", params![&id])?;
            Ok(())
        }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use std::sync::Arc;
    use super::super::SqliteDBPool;
    use finql_data::{Asset, AssetHandler, DataItem};
    
    #[tokio::test]
    async fn transaction_handler_test() {
        let db_pool = Arc::new(SqliteDBPool::in_memory().await.unwrap());
        let db = db_pool.get_conection().await.unwrap();
        assert!(db.clean().await.is_ok());

        let mut asset = Asset::new_stock(
            None,
            "A asset".to_string(),
            Some("Just a simple asset for testing".to_string()),
            "123456789012".to_string(),
            Some("A1B2C3".to_string()),
        );

        let asset_id = db.insert_asset(&mut asset).await.unwrap();
        assert_eq!(asset_id, 1);

        let mut eur_asset = Asset::new_currency(
            None,
            "euro".to_string(),
            Some("Currency of EU".to_string()),
            CurrencyISOCode::from_str("EUR").unwrap(),
            2
        );

        let eur_id = db.insert_asset(&mut eur_asset).await.unwrap();
        eur_asset.set_id(eur_id);
        let eur = eur_asset.currency().unwrap();

        let asset_buy = Transaction {
            id: None,
            transaction_type: TransactionType::Asset{ asset_id, position: 100.0 },
            cash_flow: CashFlow::new(-100.0, eur, NaiveDate::from_ymd(2020, 12, 02)),
            note: Some("First buy".to_string()),
        };
        let buy_id = db.insert_transaction(&asset_buy).await.unwrap();
        assert_eq!(buy_id, 1);

        let dividend = Transaction {
            id: None,
            transaction_type: TransactionType::Dividend{ asset_id },
            cash_flow: CashFlow::new(6.0, eur, NaiveDate::from_ymd(2020, 12, 02)),
            note: None,
        };
        let dividend_id = db.insert_transaction(&dividend).await.unwrap();
        assert_eq!(dividend_id, 2);


        let interest = Transaction {
            id: None,
            transaction_type: TransactionType::Interest{ asset_id },
            cash_flow: CashFlow::new(3.0, eur, NaiveDate::from_ymd(2020, 12, 02)),
            note: None,
        };
        let interest_id = db.insert_transaction(&interest).await.unwrap();
        assert_eq!(interest_id, 3);

        let tax = Transaction {
            id: None,
            transaction_type: TransactionType::Tax{ transaction_ref: Some(dividend_id) },
            cash_flow: CashFlow::new(-4.0, eur, NaiveDate::from_ymd(2020, 12, 02)),
            note: None,
        };
        let tax_id = db.insert_transaction(&tax).await.unwrap();
        assert_eq!(tax_id, 4);

        let fee = Transaction {
            id: None,
            transaction_type: TransactionType::Fee{ transaction_ref: Some(buy_id) },
            cash_flow: CashFlow::new(-0.5, eur, NaiveDate::from_ymd(2020, 12, 02)),
            note: None,
        };
        let fee_id = db.insert_transaction(&fee).await.unwrap();
        assert_eq!(fee_id, 5);

        let cash = Transaction {
            id: None,
            transaction_type: TransactionType::Cash,
            cash_flow: CashFlow::new(100.0, eur, NaiveDate::from_ymd(2020, 12, 02)),
            note: None,
        };
        let cash_id = db.insert_transaction(&cash).await.unwrap();
        assert_eq!(cash_id, 6);

        let mut cash2 = db.get_transaction_by_id(6).await.unwrap();
        assert_eq!(cash2.id, Some(6));
        cash2.cash_flow = CashFlow::new(200.0, eur, NaiveDate::from_ymd(2020, 12, 01));
        assert!(db.update_transaction(&cash2).await.is_ok());

        assert!(db.delete_transaction(interest_id).await.is_ok());
        assert_eq!(db.get_all_transactions().await.unwrap().len(), 5, "get_all_transactions");
    }
}
