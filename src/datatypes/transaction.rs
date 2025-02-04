//! Implementation of basic transaction types
use super::CashFlow;
use super::{DataError, DataItem};
use serde::{Deserialize, Serialize};

/// Type of transaction
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Cash,
    Asset { asset_id: i32, position: f64 },
    Dividend { asset_id: i32 },
    Interest { asset_id: i32 },
    Tax { transaction_ref: Option<i32> },
    Fee { transaction_ref: Option<i32> },
}

/// Basic transaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    // Before a transaction is stored to a database, the id maybe None
    pub id: Option<i32>,
    pub transaction_type: TransactionType,
    pub cash_flow: CashFlow,
    pub note: Option<String>,
}

impl Transaction {
    /// Assign or change transaction's asset_id, if possible
    /// This is often required for transactions on new assets
    pub fn set_asset_id(&mut self, asset_id: i32) {
        self.transaction_type = match self.transaction_type {
            TransactionType::Asset {
                asset_id: _,
                position,
            } => TransactionType::Asset { asset_id, position },
            TransactionType::Dividend { asset_id: _ } => TransactionType::Dividend { asset_id },
            TransactionType::Interest { asset_id: _ } => TransactionType::Interest { asset_id },
            _ => self.transaction_type,
        }
    }

    /// Assign new transaction reference, if applicable
    pub fn set_transaction_ref(&mut self, trans_ref: i32) {
        self.transaction_type = match self.transaction_type {
            TransactionType::Tax { transaction_ref: _ } => TransactionType::Tax {
                transaction_ref: Some(trans_ref),
            },
            TransactionType::Fee { transaction_ref: _ } => TransactionType::Fee {
                transaction_ref: Some(trans_ref),
            },
            _ => self.transaction_type,
        }
    }
}

impl DataItem for Transaction {
    // get id or return error if id hasn't been set yet
    fn get_id(&self) -> Result<i32, DataError> {
        match self.id {
            Some(id) => Ok(id),
            None => Err(DataError::DataAccessFailure(
                "tried to get id of temporary transaction".to_string(),
            )),
        }
    }
    // set id or return error if id has already been set
    fn set_id(&mut self, id: i32) -> Result<(), DataError> {
        match self.id {
            Some(_) => Err(DataError::DataAccessFailure(
                "tried to change valid transaction id".to_string(),
            )),
            None => {
                self.id = Some(id);
                Ok(())
            }
        }
    }
}
