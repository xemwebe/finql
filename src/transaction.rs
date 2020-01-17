///! Implementation of basic transaction types

use crate::fixed_income::CashFlow;
use serde::{Deserialize, Serialize};
use crate::data_handler::{DataError, DataItem};

/// Type of transaction
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Cash,
    Asset { asset_id: usize, position: f64 },
    Dividend { asset_id: usize },
    Interest { asset_id: usize },
    Tax { transaction_ref: Option<usize> },
    Fee { transaction_ref: Option<usize> },
}

/// Basic transaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    // Before a transaction is stored to a database, the id maybe None
    pub id: Option<usize>,
    pub transaction_type: TransactionType,
    pub cash_flow: CashFlow,
    pub note: Option<String>,
}

impl DataItem for Transaction {
    // get id or return error if id hasn't been set yet
    fn get_id(&self) -> Result<usize, DataError> {
        match self.id {
            Some(id) => Ok(id),
            None => Err(DataError::DataAccessFailure("tried to get id of temporary transaction".to_string())),
        }
    }
    // set id or return error if id has already been set
    fn set_id(&mut self, id: usize) -> Result<(), DataError> {
        match self.id {
            Some(_) => Err(DataError::DataAccessFailure("tried to change valid transaction id".to_string())),
            None => {
                self.id = Some(id);
                Ok(())
            }
        }
    }
}