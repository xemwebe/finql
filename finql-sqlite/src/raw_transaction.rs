use std::str::FromStr;

use chrono::NaiveDate;

use finql_data::DataError;
use finql_data::transaction::{Transaction, TransactionType};
use finql_data::cash_flow::CashFlow;
use finql_data::currency::Currency;

pub struct RawTransaction {
    pub id: Option<i64>,
    pub trans_type: String,
    pub asset: Option<i64>,
    pub cash_amount: f64,
    pub cash_currency: String,
    pub cash_date: String,
    pub related_trans: Option<i64>,
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


/// Construct cash flow from raw strings
pub fn raw_to_cash_flow(amount: f64, currency: &str, date: &str) -> Result<CashFlow, DataError> {
    let currency = Currency::from_str(currency).map_err(|e| DataError::NotFound(e.to_string()))?;
    let date = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|e| DataError::NotFound(e.to_string()))?;
    Ok(CashFlow::new(amount, currency, date))
}


impl RawTransaction {
    pub fn to_transaction(&self) -> Result<Transaction, DataError> {
        let id = self.id.map(|x| x as usize);
        let cash_flow = raw_to_cash_flow(self.cash_amount, &self.cash_currency, &self.cash_date)?;
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
                transaction_ref: self.id.map(|x| x as usize),
            },
            FEE => TransactionType::Fee {
                transaction_ref: self.id.map(|x| x as usize),
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
        let id = transaction.id.map(|x| x as i64);
        let cash_amount = transaction.cash_flow.amount.amount;
        let cash_currency = transaction.cash_flow.amount.currency.to_string();
        let cash_date = transaction.cash_flow.date.format("%Y-%m-%d").to_string();
        let note = transaction.note.clone();
        let mut raw_transaction = RawTransaction {
            id,
            trans_type: String::new(),
            asset: None,
            cash_amount,
            cash_currency,
            cash_date,
            related_trans: None,
            position: None,
            note,
        };
        match transaction.transaction_type {
            TransactionType::Cash => raw_transaction.trans_type = CASH.to_string(),
            TransactionType::Asset { asset_id, position } => {
                raw_transaction.trans_type = ASSET.to_string();
                raw_transaction.asset = Some(asset_id as i64);
                raw_transaction.position = Some(position);
            }
            TransactionType::Dividend { asset_id } => {
                raw_transaction.trans_type = DIVIDEND.to_string();
                raw_transaction.asset = Some(asset_id as i64);
            }
            TransactionType::Interest { asset_id } => {
                raw_transaction.trans_type = INTEREST.to_string();
                raw_transaction.asset = Some(asset_id as i64);
            }
            TransactionType::Tax { transaction_ref } => {
                raw_transaction.trans_type = TAX.to_string();
                raw_transaction.related_trans = transaction_ref.map(|x| x as i64);
            }
            TransactionType::Fee { transaction_ref } => {
                raw_transaction.trans_type = FEE.to_string();
                raw_transaction.related_trans = transaction_ref.map(|x| x as i64);
            }
        };
        raw_transaction
    }
}
