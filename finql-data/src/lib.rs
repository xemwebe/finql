///! Implementation of a data handler trait to deal with global data
use std::fmt;

pub mod asset_handler;
pub mod quote_handler;
pub mod transaction_handler;
pub mod asset;
pub mod transaction;
pub mod currency;
pub mod cash_flow;
pub mod quote;

pub use asset::Asset;
pub use asset_handler::AssetHandler;
pub use quote::{Quote, Ticker};
pub use quote_handler::QuoteHandler;
pub use transaction::{Transaction, TransactionType};
pub use transaction_handler::TransactionHandler;
pub use currency::{Currency, CurrencyConverter, CurrencyError};
pub use cash_flow::{CashAmount, CashFlow};

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
            Self::NotFound(err) => write!(f, "could not found request object in database: {}", err),
            Self::UpdateFailed(err) => write!(f, "update of object in database failed: {}", err),
            Self::DeleteFailed(err) => write!(f, "removing object from database failed: {}", err),
            Self::InsertFailed(err) => write!(f, "inserting object to database failed: {}", err),
            Self::InvalidTransaction(err) => write!(f, "invalid transaction type: {}", err),
        }
    }
}

pub trait DataItem {
    // get id or return error if id hasn't been set yet
    fn get_id(&self) -> Result<usize, DataError>;
    // set id or return error if id has already been set
    fn set_id(&mut self, id: usize) -> Result<(), DataError>;
}
