use crate::asset::Asset;
use crate::transaction::Transaction;
///! Implementation of a data handler trait to deal with global data
use std::fmt;

#[derive(Debug)]
pub enum DataError {
    DataAccessFailure(String),
    NotFound(String),
    UpdateFailed(String),
    DeleteFailed(String),
    InsertFailed(String),
    InvalidTransaction(String),
}

impl std::error::Error for DataError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        Some(self)
    }
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DataAccessFailure(err) => write!(f, "connection to database failed: {}", err),
            Self::NotFound(err) => write!(f, "could found request object in database: {}", err),
            Self::UpdateFailed(err) => write!(f, "update of object in database failed: {}", err),
            Self::DeleteFailed(err) => write!(f, "removing object from database failed: {}", err),
            Self::InsertFailed(err) => write!(f, "inserting object to database failed: {}", err),
            Self::InvalidTransaction(err) => write!(f, "invalid transaction type: {}", err),
        }
    }
}

/// Handler for globally available data
pub trait DataHandler {
    // insert, get, update and delete for assets
    fn insert_asset(&self, asset: &Asset) -> Result<usize, DataError>;
    fn get_asset_by_id(&self, id: usize) -> Result<Asset, DataError>;
    fn get_all_assets(&self) -> Result<Vec<Asset>, DataError>;
    fn update_asset(&self, asset: &Asset) -> Result<(), DataError>;
    fn delete_asset(&self, id: usize) -> Result<(), DataError>;

    // insert, get, update and delete for transactions
    fn insert_transaction(&self, transaction: &Transaction) -> Result<usize, DataError>;
    fn get_transaction_by_id(&self, id: usize) -> Result<Transaction, DataError>;
    fn get_all_transactions(&self) -> Result<Vec<Transaction>, DataError>;
    fn update_transaction(&self, transaction: &Transaction) -> Result<(), DataError>;
    fn delete_transaction(&self, id: usize) -> Result<(), DataError>;
}
