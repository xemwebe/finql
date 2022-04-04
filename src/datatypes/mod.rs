///! Implementation of a data handler trait to deal with global data
use thiserror::Error;

pub mod asset;
pub mod asset_handler;
pub mod cash_flow;
pub mod currency;
pub mod date_time_helper;
pub mod quote;
pub mod quote_handler;
pub mod stock;
pub mod transaction_handler;
pub mod transaction;
pub mod object_handler;

pub use asset::Asset;
pub use asset_handler::AssetHandler;
pub use quote::{Quote, Ticker};
pub use quote_handler::QuoteHandler;
pub use stock::Stock;
pub use transaction::{Transaction, TransactionType};
pub use transaction_handler::TransactionHandler;
pub use currency::{Currency, CurrencyConverter, CurrencyError, CurrencyISOCode};
pub use cash_flow::{CashAmount, CashFlow};
pub use object_handler::ObjectHandler;


#[derive(Error, Debug)]
pub enum DataError {
    #[error("Connection to database failed: {0}")]
    DataAccessFailure(String),
    #[error("could not found request object in database: {0}")]
    NotFound(String),
    #[error("update of object in database failed: {0}")]
    UpdateFailed(String),
    #[error("removing object from database failed: {0}")]
    DeleteFailed(String),
    #[error("inserting object to database failed: {0}")]
    InsertFailed(String),
    #[error("invalid asset data: {0}")]
    InvalidAsset(String),
    #[error("invalid transaction type: {0}")]
    InvalidTransaction(String),
    #[error("Invalid currency")]
    InvalidCurrency(#[from] CurrencyError),
}

pub trait DataItem {
    // get id or return error if id hasn't been set yet
    fn get_id(&self) -> Result<usize, DataError>;
    // set id or return error if id has already been set
    fn set_id(&mut self, id: usize) -> Result<(), DataError>;
}
